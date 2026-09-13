use std::process::Command;

pub struct TerminalLauncher;

impl TerminalLauncher {
    pub fn launch(session_name: &str) {
        let command = herdr_command(session_name);
        let terminal = find_terminal();
        run_in_terminal(&terminal, &command);
    }
}

fn herdr_command(session_name: &str) -> String {
    if session_name == "default" {
        "herdr".to_string()
    } else {
        format!("herdr --session {}", session_name)
    }
}

fn find_terminal() -> String {
    let candidates = [
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "mate-terminal",
        "lxterminal",
        "terminator",
        "tilix",
        "alacritty",
        "kitty",
        "ghostty",
        "foot",
        "wezterm",
        "xterm",
    ];

    for candidate in &candidates {
        if which(candidate) {
            return candidate.to_string();
        }
    }

    if let Ok(output) = Command::new("xdg-mime").args(["query", "default", "x-scheme-handler/terminal"]).output() {
        if output.status.success() {
            let desktop = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !desktop.is_empty() {
                return desktop;
            }
        }
    }

    "xterm".to_string()
}

fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_in_terminal(terminal: &str, command: &str) {
    let result = match terminal {
        "gnome-terminal" => Command::new("gnome-terminal").args(["--", "bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        "konsole" => Command::new("konsole").args(["-e", &format!("bash -c '{}; exec bash'", command)]).spawn(),
        "xfce4-terminal" => Command::new("xfce4-terminal").args(["--command", &format!("bash -c '{}; exec bash'", command)]).spawn(),
        "alacritty" => Command::new("alacritty").args(["-e", "bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        "kitty" => Command::new("kitty").args(["bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        "ghostty" => Command::new("ghostty").args(["-e", "bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        "wezterm" => Command::new("wezterm").args(["start", "--", "bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        "foot" => Command::new("foot").args(["bash", "-c", &format!("{}; exec bash", command)]).spawn(),
        _ => Command::new("xdg-terminal").arg(command).spawn(),
    };

    if let Err(e) = result {
        log::error!("failed to launch terminal '{}': {}", terminal, e);
    }
}
