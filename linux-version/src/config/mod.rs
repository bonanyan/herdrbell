use std::path::PathBuf;
use std::sync::Mutex;

const DEFAULT_ICON_SCHEME: &str = "custom";
const DEFAULT_LANGUAGE: &str = "en";

pub struct Settings {
    path: PathBuf,
    data: Mutex<SettingsData>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SettingsData {
    icon_scheme_id: String,
    notifications_enabled: bool,
    language: String,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            icon_scheme_id: DEFAULT_ICON_SCHEME.to_string(),
            notifications_enabled: true,
            language: DEFAULT_LANGUAGE.to_string(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/home"))
            .join("herdrbell")
            .join("settings.json");

        let data = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            SettingsData::default()
        };

        Self {
            path,
            data: Mutex::new(data),
        }
    }

    pub fn icon_scheme_id(&self) -> String {
        self.data.lock().unwrap().icon_scheme_id.clone()
    }

    pub fn set_icon_scheme_id(&self, id: &str) {
        self.data.lock().unwrap().icon_scheme_id = id.to_string();
        self.save();
    }

    pub fn notifications_enabled(&self) -> bool {
        self.data.lock().unwrap().notifications_enabled
    }

    pub fn set_notifications_enabled(&self, enabled: bool) {
        self.data.lock().unwrap().notifications_enabled = enabled;
        self.save();
    }

    pub fn language(&self) -> String {
        self.data.lock().unwrap().language.clone()
    }

    pub fn set_language(&self, lang: &str) {
        self.data.lock().unwrap().language = lang.to_string();
        self.save();
    }

    fn save(&self) {
        let data = self.data.lock().unwrap().clone();
        let path = self.path.clone();
        std::thread::spawn(move || {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&data) {
                let _ = std::fs::write(&path, json);
            }
        });
    }
}
