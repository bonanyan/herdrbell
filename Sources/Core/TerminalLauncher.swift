import AppKit

enum TerminalLauncher: Sendable {

    static func launch(herdrForSession sessionName: String) {
        let command = herdrCommand(for: sessionName)
        guard let terminal = findTerminal() else { return }
        runInTerminal(terminal, command: command)
    }

    private static func herdrCommand(for sessionName: String) -> String {
        if sessionName == "default" { return "herdr" }
        return "herdr --session \(sessionName)"
    }

    private static func findTerminal() -> String? {
        let candidates = [
            ("com.apple.Terminal", "Terminal"),
            ("com.googlecode.iterm2", "iTerm"),
            ("dev.warp.Warp-Stable", "Warp"),
            ("com.mitchellh.ghostty", "Ghostty"),
            ("net.kovidgoyal.kitty", "Kitty"),
            ("org.alacritty", "Alacritty"),
        ]
        for (bundleID, name) in candidates {
            if NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleID) != nil {
                return name
            }
        }
        return nil
    }

    private static func runInTerminal(_ terminal: String, command: String) {
        let escaped = command.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let script: String
        switch terminal {
        case "Terminal":
            script = """
            tell application "Terminal"
                activate
                do script "\(escaped)"
            end tell
            """
        case "iTerm":
            script = """
            tell application "iTerm"
                activate
                tell current window
                    if it is missing value then
                        create window with default profile
                    else
                        create tab with default profile
                    end if
                    tell current session to write text "\(escaped)"
                end tell
            end tell
            """
        default:
            script = """
            tell application "Terminal"
                activate
                do script "\(escaped)"
            end tell
            """
        }
        if let appleScript = NSAppleScript(source: script) {
            var error: NSDictionary?
            appleScript.executeAndReturnError(&error)
        }
    }
}
