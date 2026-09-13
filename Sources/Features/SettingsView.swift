import SwiftUI

struct SettingsView: View {
    @Environment(LocalizationManager.self) private var l10n
    @AppStorage(SettingsKeys.notificationsEnabled) private var notificationsEnabled = true
    @AppStorage(SettingsKeys.iconSchemeID) private var iconSchemeID = IconSchemeRegistry.default.id
    @State private var launchAtLogin = LoginItem.isEnabled

    var body: some View {
        Form {
            Picker(l10n.string("settings.language"), selection: Binding(
                get: { l10n.language },
                set: { l10n.setLanguage($0) }
            )) {
                ForEach(AppLanguage.allCases) { language in
                    Text(verbatim: language.nativeName).tag(language)
                }
            }
            Text(l10n.string("settings.language.help"))
                .font(.caption)
                .foregroundStyle(.secondary)

            Picker(l10n.string("settings.iconStyle"), selection: $iconSchemeID) {
                ForEach(IconSchemeRegistry.all, id: \.id) { scheme in
                    Text(l10n.string(scheme.displayNameKey)).tag(scheme.id)
                }
            }
            Text(l10n.string("settings.iconStyle.help"))
                .font(.caption)
                .foregroundStyle(.secondary)

            Toggle(l10n.string("settings.launchAtLogin"), isOn: Binding(
                get: { launchAtLogin },
                set: { newValue in
                    if LoginItem.setEnabled(newValue) {
                        launchAtLogin = newValue
                    } else {
                        launchAtLogin = LoginItem.isEnabled
                    }
                }
            ))

            Toggle(l10n.string("settings.notifications"), isOn: $notificationsEnabled)

            Divider()
                .padding(.vertical, 4)

            HStack {
                Spacer()
                Text(verbatim: "HerdrBell v\(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "?")")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
            }
        }
        .formStyle(.grouped)
        .frame(width: 420)
        .padding(.vertical, 8)
    }
}
