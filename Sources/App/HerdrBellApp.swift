// HerdrBell v2.0
import SwiftUI

@main
struct HerdrBellApp: App {
    @State private var store: HerdrStore
    @State private var localization = LocalizationManager.shared

    init() {
        let store = HerdrStore()
        _store = State(initialValue: store)
        store.start()
        if CommandLine.arguments.contains("--herdrbell-test-notification") {
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(2))
                let l10n = LocalizationManager.shared
                let appearance = IconSchemeRegistry.current.appearance(for: .blocked)
                Notifier().post(
                    title: l10n.string("notification.blocked.title", "opencode"),
                    body: l10n.string("notification.body.session", "default"),
                    icon: appearance.icon,
                    tint: appearance.color
                )
            }
        }
    }

    var body: some Scene {
        MenuBarExtra {
            MenuView()
        } label: {
            MenuBarLabel(icon: store.aggregateIcon)
        }
        .menuBarExtraStyle(.window)
        .environment(store)
        .environment(localization)
    }
}
