use std::fs;
use std::path::PathBuf;

const DESKTOP_ENTRY: &str = "[Desktop Entry]
Type=Application
Name=HerdrBell
Comment=Menu-bar monitor for herdr coding agents
Exec=herdrbell
Terminal=false
Categories=Utility;Monitor;
";

fn autostart_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/home"));
    path.push("autostart");
    path
}

fn desktop_file_path() -> PathBuf {
    let mut path = autostart_path();
    path.push("herdrbell.desktop");
    path
}

pub fn is_enabled() -> bool {
    desktop_file_path().exists()
}

pub fn set_enabled(enabled: bool) -> Result<(), std::io::Error> {
    if enabled {
        let dir = autostart_path();
        fs::create_dir_all(&dir)?;
        fs::write(desktop_file_path(), DESKTOP_ENTRY)?;
    } else {
        let path = desktop_file_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
