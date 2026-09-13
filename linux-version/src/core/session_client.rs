use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::core::agent_item::AgentItem;
use crate::core::socket::HerdrSocket;
use crate::wire::{self, AgentStatus, HerdrEventEnvelope};

pub enum ClientEvent {
    ConnectionChanged {
        session_name: String,
        connected: bool,
    },
    AgentsChanged {
        session_name: String,
        agents: Vec<AgentItem>,
    },
    StatusChanged {
        session_name: String,
        pane_id: String,
        from: Option<AgentStatus>,
        to: AgentStatus,
    },
}

pub struct HerdrSessionClient {
    session_name: String,
    socket: HerdrSocket,
    event_tx: tokio::sync::mpsc::UnboundedSender<ClientEvent>,
    agents_by_pane: Arc<Mutex<HashMap<String, AgentItem>>>,
    last_emitted: Arc<Mutex<Vec<AgentItem>>>,
    needs_resubscribe: Arc<Mutex<bool>>,
    running: Arc<Mutex<bool>>,
}

const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1500);

impl HerdrSessionClient {
    pub fn new(
        session_name: String,
        socket_path: String,
        event_tx: tokio::sync::mpsc::UnboundedSender<ClientEvent>,
    ) -> Self {
        Self {
            session_name,
            socket: HerdrSocket::new(socket_path),
            event_tx,
            agents_by_pane: Arc::new(Mutex::new(HashMap::new())),
            last_emitted: Arc::new(Mutex::new(Vec::new())),
            needs_resubscribe: Arc::new(Mutex::new(false)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let session_name = self.session_name.clone();
        let socket = HerdrSocket::new(self.socket.path().to_string());
        let event_tx = self.event_tx.clone();
        let agents_by_pane = self.agents_by_pane.clone();
        let last_emitted = self.last_emitted.clone();
        let needs_resubscribe = self.needs_resubscribe.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            let mut delay = 1u64;
            loop {
                let running = *running_flag.lock().await;
                if !running {
                    break;
                }

                let mut connected = false;

                if socket.ping().await.is_ok() {
                    if let Ok(snapshot) = socket.session_snapshot().await {
                        {
                            let mut agents = agents_by_pane.lock().await;
                            apply_agents_from_snapshot(&mut agents, &snapshot.agents);
                        }
                        emit_if_changed(
                            &session_name,
                            &agents_by_pane,
                            &last_emitted,
                            &event_tx,
                        )
                        .await;
                        connected = true;
                        delay = 1;
                        let _ = event_tx.send(ClientEvent::ConnectionChanged {
                            session_name: session_name.clone(),
                            connected: true,
                        });

                        while *running_flag.lock().await {
                            *needs_resubscribe.lock().await = false;
                            if let Err(_) = live_once(
                                &socket,
                                &session_name,
                                &agents_by_pane,
                                &last_emitted,
                                &needs_resubscribe,
                                &running_flag,
                                &event_tx,
                            )
                            .await
                            {
                                break;
                            }
                            let _ = refresh_agent_list(
                                &socket,
                                &session_name,
                                &agents_by_pane,
                                &last_emitted,
                                &event_tx,
                            )
                            .await;
                            if !*needs_resubscribe.lock().await {
                                break;
                            }
                        }
                    }
                }

                if connected {
                    let _ = event_tx.send(ClientEvent::ConnectionChanged {
                        session_name: session_name.clone(),
                        connected: false,
                    });
                }

                if !*running_flag.lock().await {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                delay = (delay * 2).min(30);
            }
        });
    }

    pub async fn stop(&self) {
        *self.running.lock().await = false;
    }

    pub async fn focus(&self, pane_id: &str) -> Result<(), crate::core::socket::SocketError> {
        self.socket.agent_focus(pane_id).await
    }

    pub async fn refresh_now(&self) {
        let running = *self.running.lock().await;
        if !running {
            return;
        }
        let _ = refresh_agent_list(
            &self.socket,
            &self.session_name,
            &self.agents_by_pane,
            &self.last_emitted,
            &self.event_tx,
        )
        .await;
    }
}

async fn refresh_agent_list(
    socket: &HerdrSocket,
    session_name: &str,
    agents_by_pane: &Arc<Mutex<HashMap<String, AgentItem>>>,
    last_emitted: &Arc<Mutex<Vec<AgentItem>>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<ClientEvent>,
) -> Result<(), crate::core::socket::SocketError> {
    let agents = socket.agent_list().await?;
    {
        let mut map = agents_by_pane.lock().await;
        map.clear();
        for info in &agents {
            if let Some(pane_id) = &info.pane_id {
                if !pane_id.is_empty() {
                    map.insert(pane_id.clone(), AgentItem::from_info(info));
                }
            }
        }
    }
    emit_if_changed(session_name, agents_by_pane, last_emitted, event_tx).await;
    Ok(())
}

fn apply_agents_from_snapshot(
    agents: &mut HashMap<String, AgentItem>,
    infos: &[crate::wire::AgentInfo],
) {
    agents.clear();
    for info in infos {
        if let Some(pane_id) = &info.pane_id {
            if !pane_id.is_empty() {
                agents.insert(pane_id.clone(), AgentItem::from_info(info));
            }
        }
    }
}

async fn emit_if_changed(
    session_name: &str,
    agents_by_pane: &Arc<Mutex<HashMap<String, AgentItem>>>,
    last_emitted: &Arc<Mutex<Vec<AgentItem>>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<ClientEvent>,
) {
    let map = agents_by_pane.lock().await;
    let mut current: Vec<AgentItem> = map.values().cloned().collect();
    current.sort_by(|a, b| a.pane_id.cmp(&b.pane_id));
    drop(map);

    let mut last = last_emitted.lock().await;
    if current != *last {
        *last = current.clone();
        let _ = event_tx.send(ClientEvent::AgentsChanged {
            session_name: session_name.to_string(),
            agents: current,
        });
    }
}

async fn live_once(
    socket: &HerdrSocket,
    session_name: &str,
    agents_by_pane: &Arc<Mutex<HashMap<String, AgentItem>>>,
    last_emitted: &Arc<Mutex<Vec<AgentItem>>>,
    needs_resubscribe: &Arc<Mutex<bool>>,
    running: &Arc<Mutex<bool>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<ClientEvent>,
) -> Result<(), crate::core::socket::SocketError> {
    let pane_ids: Vec<String> = agents_by_pane.lock().await.keys().cloned().collect();
    let subs = wire::current_subscriptions(&pane_ids);
    let mut rx = socket.subscribe(subs).await?;

    let socket_for_poll = HerdrSocket::new(socket.path().to_string());
    let session_name_poll = session_name.to_string();
    let agents_for_poll = agents_by_pane.clone();
    let last_for_poll = last_emitted.clone();
    let event_tx_poll = event_tx.clone();
    let needs_resub_poll = needs_resubscribe.clone();
    let running_poll = running.clone();

    let poll_handle = tokio::spawn(async move {
        while *running_poll.lock().await && !*needs_resub_poll.lock().await {
            tokio::time::sleep(REFRESH_INTERVAL).await;
            if !*running_poll.lock().await || *needs_resub_poll.lock().await {
                break;
            }
            let _ = refresh_agent_list(
                &socket_for_poll,
                &session_name_poll,
                &agents_for_poll,
                &last_for_poll,
                &event_tx_poll,
            )
            .await;
        }
    });

    while *running.lock().await {
        match rx.recv().await {
            Some(envelope) => {
                handle_event(
                    envelope,
                    session_name,
                    agents_by_pane,
                    last_emitted,
                    needs_resubscribe,
                    event_tx,
                )
                .await;
                if *needs_resubscribe.lock().await {
                    break;
                }
            }
            None => break,
        }
    }

    poll_handle.abort();
    Ok(())
}

async fn handle_event(
    envelope: HerdrEventEnvelope,
    session_name: &str,
    agents_by_pane: &Arc<Mutex<HashMap<String, AgentItem>>>,
    last_emitted: &Arc<Mutex<Vec<AgentItem>>>,
    needs_resubscribe: &Arc<Mutex<bool>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<ClientEvent>,
) {
    match envelope.event.as_str() {
        "pane_created" | "pane_updated" => {
            if let Ok(data) = wire::decode_result::<wire::PaneEventData>(&envelope.data) {
                let mut agents = agents_by_pane.lock().await;
                if let Some(item) = agents.get_mut(&data.pane.pane_id) {
                    if let Some(old_status) = item.apply_pane(&data.pane) {
                        let _ = event_tx.send(ClientEvent::StatusChanged {
                            session_name: session_name.to_string(),
                            pane_id: data.pane.pane_id.clone(),
                            from: Some(old_status),
                            to: item.status,
                        });
                    }
                    drop(agents);
                    emit_if_changed(session_name, agents_by_pane, last_emitted, event_tx).await;
                } else if let Some(agent) = &data.pane.agent {
                    if !agent.is_empty() && data.pane.agent_status.is_some() {
                        let mut item = AgentItem::from_info(&crate::wire::AgentInfo {
                            terminal_id: data.pane.terminal_id.clone(),
                            agent: data.pane.agent.clone(),
                            terminal_title: data.pane.terminal_title.clone(),
                            terminal_title_stripped: data.pane.terminal_title_stripped.clone(),
                            agent_status: data.pane.agent_status.unwrap_or(AgentStatus::Unknown),
                            screen_detection_skipped: None,
                            agent_session: data.pane.agent_session.clone(),
                            workspace_id: data.pane.workspace_id.clone(),
                            tab_id: data.pane.tab_id.clone(),
                            pane_id: Some(data.pane.pane_id.clone()),
                            focused: data.pane.focused,
                            state_change_seq: None,
                            cwd: data.pane.cwd.clone(),
                            foreground_cwd: data.pane.foreground_cwd.clone(),
                            revision: data.pane.revision,
                        });
                        if let Some(status) = data.pane.agent_status {
                            item.status = status;
                        }
                        agents.insert(data.pane.pane_id.clone(), item);
                        *needs_resubscribe.lock().await = true;
                        drop(agents);
                        emit_if_changed(session_name, agents_by_pane, last_emitted, event_tx).await;
                    }
                }
            }
        }
        "pane_agent_status_changed" => {
            if let Ok(data) = wire::decode_result::<wire::AgentStatusChangedEventData>(&envelope.data)
            {
                let mut agents = agents_by_pane.lock().await;
                if let Some(item) = agents.get_mut(&data.pane_id) {
                    let old_status = item.status;
                    item.status = data.agent_status;
                    if let Some(agent) = &data.agent {
                        item.agent = agent.clone();
                    }
                    if let Some(title) = &data.title {
                        if !title.is_empty() {
                            item.title = title.clone();
                        }
                    }
                    if old_status != data.agent_status {
                        let _ = event_tx.send(ClientEvent::StatusChanged {
                            session_name: session_name.to_string(),
                            pane_id: data.pane_id.clone(),
                            from: Some(old_status),
                            to: data.agent_status,
                        });
                    }
                    drop(agents);
                    emit_if_changed(session_name, agents_by_pane, last_emitted, event_tx).await;
                } else {
                    *needs_resubscribe.lock().await = true;
                }
            }
        }
        "pane_closed" | "pane_exited" => {
            if let Ok(data) = wire::decode_result::<wire::PaneIdEventData>(&envelope.data) {
                let removed = agents_by_pane.lock().await.remove(&data.pane_id);
                if removed.is_some() {
                    emit_if_changed(session_name, agents_by_pane, last_emitted, event_tx).await;
                }
                *needs_resubscribe.lock().await = true;
            }
        }
        "pane_agent_detected" | "workspace_closed" => {
            *needs_resubscribe.lock().await = true;
        }
        _ => {}
    }
}
