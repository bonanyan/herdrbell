import Foundation
import Observation

enum AppLanguage: String, CaseIterable, Identifiable, Sendable {
    case english = "en"
    case korean = "ko"
    case chinese = "zh-Hans"
    case vietnamese = "vi"
    case german = "de"
    case hindi = "hi"
    case french = "fr"
    case japanese = "ja"

    var id: String { rawValue }

    /// Name of the language written in the language itself (picker label).
    var nativeName: String {
        switch self {
        case .english: "English"
        case .korean: "한국어"
        case .chinese: "简体中文"
        case .vietnamese: "Tiếng Việt"
        case .german: "Deutsch"
        case .hindi: "हिन्दी"
        case .french: "Français"
        case .japanese: "日本語"
        }
    }
}

/// App-level language override. The picker in the Configure window switches
/// the language live (no relaunch) by resolving a localized sub-bundle and
/// looking every string up through it.
@MainActor
@Observable
final class LocalizationManager {
    static let shared = LocalizationManager()

    private(set) var language: AppLanguage
    /// Tracked by observation so views re-resolve strings when it changes.
    private(set) var bundle: Bundle

    init() {
        let stored = UserDefaults.standard.string(forKey: SettingsKeys.appLanguage)
        let language = stored.flatMap(AppLanguage.init(rawValue:)) ?? .english
        self.language = language
        self.bundle = Self.bundle(for: language)
    }

    func setLanguage(_ newLanguage: AppLanguage) {
        guard newLanguage != language else { return }
        language = newLanguage
        bundle = Self.bundle(for: newLanguage)
        UserDefaults.standard.set(newLanguage.rawValue, forKey: SettingsKeys.appLanguage)
        // Applies to system-provided UI (e.g. permission alerts) on next launch.
        UserDefaults.standard.set([newLanguage.rawValue, "en"], forKey: "AppleLanguages")
    }

    func string(_ key: String) -> String {
        NSLocalizedString(key, bundle: bundle, comment: "")
    }

    func string(_ key: String, _ args: CVarArg...) -> String {
        String(format: string(key), locale: Locale(identifier: language.rawValue), arguments: args)
    }

    private static func bundle(for language: AppLanguage) -> Bundle {
        guard let path = Bundle.main.path(forResource: language.rawValue, ofType: "lproj"),
              let localized = Bundle(path: path)
        else { return .main }
        return localized
    }
}
