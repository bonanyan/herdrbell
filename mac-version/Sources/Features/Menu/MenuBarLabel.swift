import SwiftUI

/// The menu bar item itself: renders the active scheme's aggregate icon
/// (SF Symbol or template artwork) so custom schemes can own the menu bar.
struct MenuBarLabel: View {
    let icon: StatusIcon

    var body: some View {
        StatusIconView(icon: icon, assetSize: 18)
            .frame(height: 18)
    }
}
