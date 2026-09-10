import SwiftUI

/// Where a status icon's artwork comes from.
enum StatusIcon: Sendable, Equatable {
    /// An SF Symbol name.
    case systemSymbol(String)

    /// An imageset in the asset catalog (template-rendered). Falls back to
    /// the given SF Symbol while the artwork is missing.
    case asset(name: String, fallback: String)
}

struct StatusAppearance: Sendable, Equatable {
    let icon: StatusIcon
    let color: Color
}

/// A pluggable visual theme mapping agent statuses to icons and colors.
///
/// To add a new icon set:
/// 1. Create a struct conforming to `IconScheme`.
/// 2. Append it to `IconSchemeRegistry.all`.
/// 3. If it uses custom artwork, drop the imagesets into
///    `Sources/Resources/Assets.xcassets/StatusIcons/` (see README for the
///    artwork spec); until then the scheme renders its fallback symbols.
/// Nothing else needs to change — the menu, menu bar aggregate icon,
/// and the Configure picker pick it up automatically.
protocol IconScheme: Sendable {
    /// Stable identifier persisted in UserDefaults (`SettingsKeys.iconSchemeID`).
    var id: String { get }

    /// Localization key for the human-readable scheme name.
    var displayNameKey: String { get }

    /// Menu row icon and tint for a single agent status.
    func appearance(for status: AgentStatus) -> StatusAppearance

    /// Menu bar icon summarizing all agents across connected sessions.
    /// Priority: blocked > unknown > done; working only when every agent is
    /// working; otherwise the scheme's all-idle aggregate.
    func aggregateIcon(for statuses: some Sequence<AgentStatus>) -> StatusIcon

    /// Menu bar icon shown when everything is idle (after the settle debounce).
    var idleAggregateIcon: StatusIcon { get }

    /// Menu bar icon shown when no herdr session is connected.
    var disconnectedIcon: StatusIcon { get }

    /// Menu row accessory marking the focused agent.
    var focusedMarkerIcon: StatusIcon { get }

    /// Icon for the menu's empty/offline state row.
    var emptyStateIcon: StatusIcon { get }
}

extension IconScheme {
    /// The single status the menu bar aggregate stands for.
    ///
    /// Priority: blocked > unknown > done. `working` only when *every* agent is
    /// working; anything else — all idle, or working mixed with idle — collapses
    /// to `.idle`, which schemes render as their all-idle aggregate artwork.
    func aggregateStatus(for statuses: some Sequence<AgentStatus>) -> AgentStatus {
        let set = Set(statuses)
        if set.contains(.blocked) { return .blocked }
        if set.contains(.unknown) { return .unknown }
        if set.contains(.done) { return .done }
        if !set.isEmpty, set.isSubset(of: [.working]) { return .working }
        return .idle
    }
}
