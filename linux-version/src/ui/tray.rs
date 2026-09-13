use std::sync::Arc;

use ksni::{menu::StandardItem, Handle, MenuItem, Tray, TrayService};

use crate::core::store::HerdrStore;
use crate::i18n::LocalizationManager;
use crate::platform::terminal::TerminalLauncher;
use crate::ui::icons::StatusIcon;

pub struct HerdrBellTray {
    pub icon_name: String,
    pub store: Arc<HerdrStore>,
    pub l10n: Arc<LocalizationManager>,
    pub on_configure: Arc<dyn Fn() + Send + Sync>,
    pub on_quit: Arc<dyn Fn() + Send + Sync>,
    pub rt_handle: tokio::runtime::Handle,
}

impl Tray for HerdrBellTray {
    fn id(&self) -> String {
        "herdrbell".into()
    }

    fn title(&self) -> String {
        "HerdrBell".into()
    }

    fn icon_name(&self) -> String {
        self.icon_name.clone()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let sessions = self.rt_handle.block_on(self.store.visible_sessions());

        let mut items = Vec::new();

        if sessions.is_empty() {
            let all_sessions = self.rt_handle.block_on(self.store.all_sessions());
            let label = if all_sessions.is_empty() {
                self.l10n.string("menu.noSessions", &[])
            } else {
                self.l10n.string("menu.connecting", &[])
            };
            items.push(MenuItem::Standard(StandardItem {
                label,
                enabled: false,
                ..Default::default()
            }));
        } else {
            for session in &sessions {
                items.push(MenuItem::Standard(StandardItem {
                    label: session.name.clone(),
                    enabled: false,
                    ..Default::default()
                }));

                if session.agents.is_empty() {
                    items.push(MenuItem::Standard(StandardItem {
                        label: self.l10n.string("menu.noAgents", &[]),
                        enabled: false,
                        ..Default::default()
                    }));
                } else {
                    for agent in &session.agents {
                        let label = format!("{} — {}", agent.agent, agent.title);
                        let store = self.store.clone();
                        let session_name = session.name.clone();
                        let agent_clone = agent.clone();
                        items.push(MenuItem::Standard(StandardItem {
                            label,
                            activate: Box::new(move |_tray: &mut HerdrBellTray| {
                                let store = store.clone();
                                let session_name = session_name.clone();
                                let agent = agent_clone.clone();
                                tokio::spawn(async move {
                                    store.focus(&agent, &session_name).await;
                                    TerminalLauncher::launch(&session_name);
                                });
                            }),
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        items.push(MenuItem::Separator);

        let on_configure = self.on_configure.clone();
        items.push(MenuItem::Standard(StandardItem {
            label: self.l10n.string("menu.configure", &[]),
            activate: Box::new(move |_tray: &mut HerdrBellTray| {
                on_configure();
            }),
            ..Default::default()
        }));

        let on_quit = self.on_quit.clone();
        items.push(MenuItem::Standard(StandardItem {
            label: self.l10n.string("menu.quit", &[]),
            activate: Box::new(move |_tray: &mut HerdrBellTray| {
                on_quit();
            }),
            ..Default::default()
        }));

        items
    }
}

pub fn status_icon_to_name(icon: &StatusIcon) -> String {
    match icon {
        StatusIcon::Symbol(name) => name.clone(),
        StatusIcon::Asset { fallback, .. } => fallback.clone(),
    }
}

pub fn start_tray(
    store: Arc<HerdrStore>,
    l10n: Arc<LocalizationManager>,
    on_configure: Arc<dyn Fn() + Send + Sync>,
    on_quit: Arc<dyn Fn() + Send + Sync>,
) -> Handle<HerdrBellTray> {
    let tray = HerdrBellTray {
        icon_name: "view-grid-symbolic".into(),
        store: store.clone(),
        l10n: l10n.clone(),
        on_configure,
        on_quit,
        rt_handle: tokio::runtime::Handle::current(),
    };

    let service = TrayService::new(tray);
    let handle = service.handle();

    let store_for_update = store.clone();
    let handle_for_update = handle.clone();
    store.set_on_update(move || {
        let store = store_for_update.clone();
        let handle = handle_for_update.clone();
        tokio::spawn(async move {
            let icon = store.aggregate_icon().await;
            let icon_name = status_icon_to_name(&icon);
            handle.update(|t: &mut HerdrBellTray| {
                t.icon_name = icon_name;
            });
        });
    });

    service.spawn();

    handle
}
