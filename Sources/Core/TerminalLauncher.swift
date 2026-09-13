import AppKit
import CoreServices

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
        let contentType = "com.apple.terminal.shell-script" as CFString
        if let cfURL = LSCopyDefaultApplicationURLForContentType(contentType, .all, nil)?.takeRetainedValue() {
            let url = cfURL as URL
            if let bundleID = Bundle(url: url)?.bundleIdentifier {
                if let name = terminalName(for: bundleID) {
                    return name
                }
            }
        }
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

    private static func terminalName(for bundleID: String) -> String? {
        switch bundleID {
        case "com.apple.Terminal": return "Terminal"
        case "com.googlecode.iterm2": return "iTerm"
        case "dev.warp.Warp-Stable": return "Warp"
        case "com.mitchellh.ghostty": return "Ghostty"
        case "net.kovidgoyal.kitty": return "Kitty"
        case "org.alacritty": return "Alacritty"
        default: return nil
        }
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
