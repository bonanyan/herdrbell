import AppKit
import SwiftUI

/// Owns the Configure window as a plain NSWindow.
///
/// The SwiftUI `Settings` scene plus `showSettingsWindow:` proved unreliable
/// for this LSUIElement menu-bar app: the window could silently fail to
/// appear (or to come forward) right after the menu dismissed. Creating and
/// showing the window ourselves makes both first-open and refocus
/// deterministic.
@MainActor
enum ConfigureWindow {
    private static var window: NSWindow?

    static func show() {
        NSApp.activate(ignoringOtherApps: true)
        if let window {
            window.makeKeyAndOrderFront(nil)
            return
        }
        let hosting = NSHostingController(
            rootView: SettingsView()
                .environment(LocalizationManager.shared)
        )
        let window = NSWindow(contentViewController: hosting)
        window.identifier = NSUserInterfaceItemIdentifier("configure")
        window.title = "HerdrBell"
        window.styleMask = [.titled, .closable]
        window.isReleasedWhenClosed = false
        var content = hosting.view.fittingSize
        if content.width <= 0 || content.height <= 0 {
            content = NSSize(width: 420, height: 460)
        }
        window.setContentSize(content)
        window.center()
        Self.window = window
        window.makeKeyAndOrderFront(nil)
    }
}
