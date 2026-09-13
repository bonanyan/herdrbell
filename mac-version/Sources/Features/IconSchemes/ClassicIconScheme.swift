import SwiftUI

struct ClassicIconScheme: IconScheme {
    let id = "classic"
    let displayNameKey = "icon.scheme.classic"

    func appearance(for status: AgentStatus) -> StatusAppearance {
        switch status {
        case .blocked: StatusAppearance(icon: .systemSymbol("hand.raised.fill"), color: .red)
        case .working: StatusAppearance(icon: .systemSymbol("arrow.triangle.2.circlepath"), color: .blue)
        case .done: StatusAppearance(icon: .systemSymbol("checkmark.circle.fill"), color: .green)
        case .idle: StatusAppearance(icon: .systemSymbol("circle.fill"), color: .gray)
        case .unknown: StatusAppearance(icon: .systemSymbol("questionmark.circle"), color: .gray)
        }
    }

    func aggregateIcon(for statuses: some Sequence<AgentStatus>) -> StatusIcon {
        switch aggregateStatus(for: statuses) {
        case .blocked: .systemSymbol("exclamationmark.octagon.fill")
        case .unknown: .systemSymbol("questionmark.circle")
        case .done: .systemSymbol("checkmark.circle.fill")
        case .working: .systemSymbol("arrow.triangle.2.circlepath")
        case .idle: idleAggregateIcon
        }
    }

    var idleAggregateIcon: StatusIcon { .systemSymbol("circle.grid.2x2") }
    var disconnectedIcon: StatusIcon { .systemSymbol("circle.slash") }
    var focusedMarkerIcon: StatusIcon { .systemSymbol("cursorarrow.rays") }
    var emptyStateIcon: StatusIcon { .systemSymbol("circle.slash") }
}
