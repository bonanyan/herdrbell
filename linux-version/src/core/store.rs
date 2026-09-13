use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::Settings;
use crate::core::agent_item::AgentItem;
use crate::core::discovery::{DiscoveredSession, SessionDiscovery};
use crate::core::session_client::{ClientEvent, HerdrSessionClient};
use crate::i18n::LocalizationManager;
use crate::platform::notifications::Notifier;
use crate::ui::icons::{IconScheme, IconSchemeRegistry, NotificationPolicy, StatusIcon};

#[derive(Debug, Clone)]
pub struct SessionView {
    pub name: String,
    pub agents: Vec<AgentItem>,
    pub connected: bool,
}

pub struct HerdrStore {
    sessions: Arc<Mutex<Vec<SessionView>>>,
    aggregate_icon: Arc<Mutex<StatusIcon>>,
    clients: Arc<Mutex<HashMap<String, HerdrSessionClient>>>,
    discovery: Arc<Mutex<SessionDiscovery>>,
    notifier: Arc<Notifier>,
    settings: Arc<Settings>,
    l10n: Arc<LocalizationManager>,
    started: Arc<Mutex<bool>>,
    _event_tx: tokio::sync::mpsc::UnboundedSender<ClientEvent>,
    idle_settle_delay: std::time::Duration,
    on_update: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl HerdrStore {
    pub fn new(settings: Arc<Settings>, l10n: Arc<LocalizationManager>) -> Self {
        let (event_tx, _) = tokio::sync::mpsc::unbounded_channel();
        let scheme = IconSchemeRegistry::scheme(&settings.icon_scheme_id());
        Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
            aggregate_icon: Arc::new(Mutex::new(scheme.disconnected_icon())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            discovery: Arc::new(Mutex::new(SessionDiscovery::new())),
            notifier: Arc::new(Notifier::new()),
            settings,
            l10n,
            started: Arc::new(Mutex::new(false)),
            _event_tx: event_tx,
            idle_settle_delay: std::time::Duration::from_millis(300),
            on_update: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_on_update<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let on_update = self.on_update.clone();
        let handle = tokio::runtime::Handle::current();
        handle.spawn(async move {
            *on_update.lock().await = Some(Box::new(callback));
        });
    }

    pub async fn start(&self) {
        let mut started = self.started.lock().await;
        if *started {
            return;
        }
        *started = true;
        drop(started);

        let sessions = self.sessions.clone();
        let aggregate_icon = self.aggregate_icon.clone();
        let clients = self.clients.clone();
        let notifier = self.notifier.clone();
        let settings = self.settings.clone();
        let l10n = self.l10n.clone();
        let started_flag = self.started.clone();
        let on_update = self.on_update.clone();
        let _idle_settle_delay = self.idle_settle_delay;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ClientEvent>();

        let mut discovery = self.discovery.lock().await;
        discovery.set_on_change(move |found| {
            let sessions = sessions.clone();
            let aggregate_icon = aggregate_icon.clone();
            let clients = clients.clone();
            let _notifier = notifier.clone();
            let settings = settings.clone();
            let _l10n = l10n.clone();
            let started_flag = started_flag.clone();
            let on_update = on_update.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                if !*started_flag.lock().await {
                    return;
                }
                sync_clients(&found, &sessions, &clients, &tx).await;
                refresh_aggregate(&sessions, &aggregate_icon, &settings).await;
                notify_update(&on_update).await;
            });
        });
        discovery.start();
        drop(discovery);

        let sessions_for_events = self.sessions.clone();
        let aggregate_icon_for_events = self.aggregate_icon.clone();
        let settings_for_events = self.settings.clone();
        let notifier_for_events = self.notifier.clone();
        let l10n_for_events = self.l10n.clone();
        let on_update_for_events = self.on_update.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                handle_client_event(
                    event,
                    &sessions_for_events,
                    &aggregate_icon_for_events,
                    &settings_for_events,
                    &notifier_for_events,
                    &l10n_for_events,
                    &on_update_for_events,
                )
                .await;
            }
        });
    }

    pub async fn stop(&self) {
        let mut started = self.started.lock().await;
        if !*started {
            return;
        }
        *started = false;
        drop(started);

        let mut discovery = self.discovery.lock().await;
        discovery.stop();
        drop(discovery);

        let clients = self.clients.lock().await;
        for client in clients.values() {
            client.stop().await;
        }
        drop(clients);

        self.sessions.lock().await.clear();
        let scheme = IconSchemeRegistry::scheme(&self.settings.icon_scheme_id());
        *self.aggregate_icon.lock().await = scheme.disconnected_icon();
    }

    pub async fn visible_sessions(&self) -> Vec<SessionView> {
        let sessions = self.sessions.lock().await;
        let mut visible: Vec<SessionView> = sessions
            .iter()
            .filter(|s| s.connected)
            .cloned()
            .collect();
        visible.sort_by(|a, b| a.name.cmp(&b.name));
        visible
    }

    pub async fn has_connections(&self) -> bool {
        !self.visible_sessions().await.is_empty()
    }

    pub async fn aggregate_icon(&self) -> StatusIcon {
        self.aggregate_icon.lock().await.clone()
    }

    pub async fn current_scheme(&self) -> Box<dyn IconScheme> {
        IconSchemeRegistry::scheme(&self.settings.icon_scheme_id())
    }

    pub async fn refresh_now(&self) {
        let started = *self.started.lock().await;
        if !started {
            return;
        }
        let clients = self.clients.lock().await;
        for client in clients.values() {
            client.refresh_now().await;
        }
    }

    pub async fn focus(&self, agent: &AgentItem, session_name: &str) {
        let clients = self.clients.lock().await;
        if let Some(client) = clients.get(session_name) {
            let _ = client.focus(&agent.pane_id).await;
        }
    }

    pub async fn all_sessions(&self) -> Vec<SessionView> {
        self.sessions.lock().await.clone()
    }
}

async fn sync_clients(
    found: &[DiscoveredSession],
    sessions: &Arc<Mutex<Vec<SessionView>>>,
    clients: &Arc<Mutex<HashMap<String, HerdrSessionClient>>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<ClientEvent>,
) {
    let found_names: std::collections::HashSet<String> =
        found.iter().map(|s| s.name.clone()).collect();

    let mut clients_guard = clients.lock().await;
    let mut sessions_guard = sessions.lock().await;

    let to_remove: Vec<String> = clients_guard
        .keys()
        .filter(|name| !found_names.contains(*name))
        .cloned()
        .collect();
    for name in to_remove {
        if let Some(client) = clients_guard.remove(&name) {
            client.stop().await;
        }
        sessions_guard.retain(|s| s.name != name);
    }

    for session in found {
        if !clients_guard.contains_key(&session.name) {
            let tx = event_tx.clone();
            let client = HerdrSessionClient::new(
                session.name.clone(),
                session.socket_path.clone(),
                tx,
            );
            clients_guard.insert(session.name.clone(), client);
            sessions_guard.push(SessionView {
                name: session.name.clone(),
                agents: Vec::new(),
                connected: false,
            });
            if let Some(client) = clients_guard.get(&session.name) {
                client.start().await;
            }
        }
    }
}

async fn refresh_aggregate(
    sessions: &Arc<Mutex<Vec<SessionView>>>,
    aggregate_icon: &Arc<Mutex<StatusIcon>>,
    settings: &Arc<Settings>,
) {
    let sessions_guard = sessions.lock().await;
    let visible: Vec<&SessionView> = sessions_guard.iter().filter(|s| s.connected).collect();
    let scheme = IconSchemeRegistry::scheme(&settings.icon_scheme_id());

    let icon = if visible.is_empty() {
        scheme.disconnected_icon()
    } else {
        let statuses: Vec<crate::wire::AgentStatus> =
            visible.iter().flat_map(|s| s.agents.iter().map(|a| a.status)).collect();
        scheme.aggregate_icon(&statuses)
    };

    *aggregate_icon.lock().await = icon;
}

async fn handle_client_event(
    event: ClientEvent,
    sessions: &Arc<Mutex<Vec<SessionView>>>,
    aggregate_icon: &Arc<Mutex<StatusIcon>>,
    settings: &Arc<Settings>,
    notifier: &Arc<Notifier>,
    l10n: &Arc<LocalizationManager>,
    on_update: &Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
) {
    match event {
        ClientEvent::ConnectionChanged {
            session_name,
            connected,
        } => {
            let mut sessions_guard = sessions.lock().await;
            if let Some(session) = sessions_guard.iter_mut().find(|s| s.name == session_name) {
                session.connected = connected;
            }
            drop(sessions_guard);
            refresh_aggregate(sessions, aggregate_icon, settings).await;
            notify_update(on_update).await;
        }
        ClientEvent::AgentsChanged {
            session_name,
            agents,
        } => {
            let mut sessions_guard = sessions.lock().await;
            if let Some(session) = sessions_guard.iter_mut().find(|s| s.name == session_name) {
                session.agents = agents;
                session.connected = true;
            }
            drop(sessions_guard);
            refresh_aggregate(sessions, aggregate_icon, settings).await;
            notify_update(on_update).await;
        }
        ClientEvent::StatusChanged {
            session_name,
            pane_id,
            from,
            to,
        } => {
            if NotificationPolicy::should_notify(from, to) {
                let sessions_guard = sessions.lock().await;
                if let Some(session) = sessions_guard.iter().find(|s| s.name == session_name) {
                    if let Some(agent) = session.agents.iter().find(|a| a.pane_id == pane_id) {
                        let scheme = IconSchemeRegistry::scheme(&settings.icon_scheme_id());
                        let _appearance = scheme.appearance(to);
                        let title_key = if to == crate::wire::AgentStatus::Blocked {
                            "notification.blocked.title"
                        } else {
                            "notification.done.title"
                        };
                        let body = if agent.title.is_empty() {
                            l10n.string("notification.body.session", &[&session_name])
                        } else {
                            agent.title.clone()
                        };
                        notifier.post(
                            &l10n.string(title_key, &[&agent.agent]),
                            &body,
                        );
                    }
                }
            }
            refresh_aggregate(sessions, aggregate_icon, settings).await;
            notify_update(on_update).await;
        }
    }
}

async fn notify_update(on_update: &Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>) {
    let cb = on_update.lock().await;
    if let Some(ref callback) = *cb {
        callback();
    }
}
