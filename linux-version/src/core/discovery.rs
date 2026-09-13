use std::path::PathBuf;
use std::sync::Arc;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscoveredSession {
    pub name: String,
    pub socket_path: String,
}

pub struct SessionDiscovery {
    config_dir: PathBuf,
    sessions_dir: PathBuf,
    watcher: Option<RecommendedWatcher>,
    last_reported: Arc<Mutex<Vec<DiscoveredSession>>>,
    on_change: Arc<Mutex<Option<Box<dyn Fn(Vec<DiscoveredSession>) + Send + Sync>>>>,
}

impl SessionDiscovery {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let config_dir = home.join(".config/herdr");
        let sessions_dir = config_dir.join("sessions");
        Self {
            config_dir,
            sessions_dir,
            watcher: None,
            last_reported: Arc::new(Mutex::new(Vec::new())),
            on_change: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_on_change<F>(&self, callback: F)
    where
        F: Fn(Vec<DiscoveredSession>) + Send + Sync + 'static,
    {
        let on_change = self.on_change.clone();
        tokio::spawn(async move {
            *on_change.lock().await = Some(Box::new(callback));
        });
    }

    pub fn start(&mut self) {
        let last_reported = self.last_reported.clone();
        let on_change = self.on_change.clone();
        let config_dir = self.config_dir.clone();
        let sessions_dir = self.sessions_dir.clone();

        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(16);

        let tx_clone = tx.clone();
        let mut watcher = match RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if res.is_ok() {
                    let _ = tx_clone.try_send(());
                }
            },
            notify::Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                log::error!("failed to create file watcher: {}", e);
                return;
            }
        };

        let _ = watcher.watch(&config_dir, RecursiveMode::NonRecursive);
        let _ = watcher.watch(&sessions_dir, RecursiveMode::NonRecursive);
        self.watcher = Some(watcher);

        let scan_now = tx.clone();
        tokio::spawn(async move {
            let _ = scan_now.send(()).await;
        });

        tokio::spawn(async move {
            let mut pending = false;
            loop {
                match rx.recv().await {
                    Some(()) => {
                        if pending {
                            continue;
                        }
                        #[allow(unused_assignments)]
                        {
                            pending = true;
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            pending = false;
                        }

                        let found = scan_sessions(&config_dir, &sessions_dir);
                        let mut last = last_reported.lock().await;
                        if found != *last {
                            *last = found.clone();
                            let cb = on_change.lock().await;
                            if let Some(ref callback) = *cb {
                                callback(found);
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        let rescan_last = self.last_reported.clone();
        let rescan_on_change = self.on_change.clone();
        let rescan_config = self.config_dir.clone();
        let rescan_sessions = self.sessions_dir.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let found = scan_sessions(&rescan_config, &rescan_sessions);
                let mut last = rescan_last.lock().await;
                if found != *last {
                    *last = found.clone();
                    let cb = rescan_on_change.lock().await;
                    if let Some(ref callback) = *cb {
                        callback(found);
                    }
                }
            }
        });
    }

    pub fn stop(&mut self) {
        self.watcher.take();
    }
}

fn scan_sessions(config_dir: &PathBuf, sessions_dir: &PathBuf) -> Vec<DiscoveredSession> {
    let mut found = Vec::new();

    let default_socket = config_dir.join("herdr.sock");
    if default_socket.exists() {
        found.push(DiscoveredSession {
            name: "default".into(),
            socket_path: default_socket.to_string_lossy().into_owned(),
        });
    }

    if let Ok(entries) = std::fs::read_dir(sessions_dir) {
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        names.sort();

        for name in names {
            let socket_path = sessions_dir.join(&name).join("herdr.sock");
            if socket_path.exists() {
                found.push(DiscoveredSession {
                    name,
                    socket_path: socket_path.to_string_lossy().into_owned(),
                });
            }
        }
    }

    found
}
