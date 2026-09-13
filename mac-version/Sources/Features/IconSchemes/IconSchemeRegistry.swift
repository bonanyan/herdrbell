import Foundation

enum IconSchemeRegistry {
    /// All available icon schemes, in Configure-picker order.
    static let all: [any IconScheme] = [
        CustomIconScheme(),
        ClassicIconScheme(),
    ]

    static var `default`: any IconScheme { all[0] }

    /// The scheme currently selected in Configure.
    static var current: any IconScheme {
        scheme(id: UserDefaults.standard.string(forKey: SettingsKeys.iconSchemeID) ?? `default`.id)
    }

    static func scheme(id: String) -> any IconScheme {
        all.first { $0.id == id } ?? `default`
    }
}

enum SettingsKeys {
    static let notificationsEnabled = "notificationsEnabled"
    static let iconSchemeID = "iconSchemeID"
    static let appLanguage = "appLanguage"
}
