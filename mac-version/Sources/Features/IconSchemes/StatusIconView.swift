import AppKit
import SwiftUI

/// Renders a `StatusIcon`: SF Symbol or template asset image (with the
/// scheme's fallback symbol while the artwork is missing).
/// Asset artwork is sized explicitly — compiled vector assets must not be
/// trusted to carry an intrinsic size.
/// Renders an agent row's status icon with idle hysteresis: herdr reports
/// brief `idle` blips between tool calls, so a transition to `idle` is only
/// shown after it persists for `settleDelay`; any other status applies
/// immediately and cancels a pending idle. Keeps menu rows from blinking.
struct StabilizedStatusIcon: View {
    let status: AgentStatus
    let scheme: any IconScheme
    let settleDelay: TimeInterval
    var assetSize: CGFloat = 16

    @State private var displayed: AgentStatus

    init(status: AgentStatus, scheme: any IconScheme, settleDelay: TimeInterval, assetSize: CGFloat = 16) {
        self.status = status
        self.scheme = scheme
        self.settleDelay = settleDelay
        self.assetSize = assetSize
        _displayed = State(initialValue: status)
    }

    var body: some View {
        let appearance = scheme.appearance(for: displayed)
        StatusIconView(icon: appearance.icon, assetSize: assetSize)
            .foregroundStyle(appearance.color)
            .frame(width: assetSize)
            .task(id: status) {
                guard status == .idle, displayed != .idle else {
                    displayed = status
                    return
                }
                try? await Task.sleep(for: .seconds(settleDelay))
                guard !Task.isCancelled else { return }
                displayed = .idle
            }
    }
}

struct StatusIconView: View {
    let icon: StatusIcon
    var assetSize: CGFloat = 16

    var body: some View {
        switch icon {
        case .systemSymbol(let name):
            Image(systemName: name)
        case .asset(let name, let fallback):
            if NSImage(named: NSImage.Name(name)) != nil {
                Image(name)
                    .renderingMode(.template)
                    .resizable()
                    .scaledToFit()
                    .frame(width: assetSize, height: assetSize)
            } else {
                Image(systemName: fallback)
            }
        }
    }
}
