import Foundation
import SwiftUI
import UserNotifications

enum NotificationPolicy: Sendable {
    static func shouldNotify(from: AgentStatus?, to: AgentStatus) -> Bool {
        guard to == .blocked || to == .done else { return false }
        return from != to
    }
}

/// Posts macOS notifications. Callers pass already-localized title/body
/// (localization happens on the main actor in `HerdrStore`).
final class Notifier: @unchecked Sendable {
    private var center: UNUserNotificationCenter {
        UNUserNotificationCenter.current()
    }

    func requestAuthorization() async {
        let settings = await center.notificationSettings()
        guard settings.authorizationStatus == .notDetermined else { return }
        _ = try? await center.requestAuthorization(options: [.alert, .sound])
    }

    func post(title: String, body: String, icon: StatusIcon? = nil, tint: Color? = nil) {
        let enabled = UserDefaults.standard.object(forKey: SettingsKeys.notificationsEnabled) as? Bool ?? true
        guard enabled else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        if let icon, let tint,
           let url = StatusIconImage.pngFileURL(for: icon, tint: tint),
           let attachment = try? UNNotificationAttachment(identifier: "status-icon", url: url, options: nil) {
            content.attachments = [attachment]
        }
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        center.add(request, withCompletionHandler: nil)
    }
}
