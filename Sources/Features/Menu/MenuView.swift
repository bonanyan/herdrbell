import AppKit
import SwiftUI

struct MenuView: View {
    @Environment(HerdrStore.self) private var store
    @Environment(LocalizationManager.self) private var l10n
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            if store.visibleSessions.isEmpty {
                emptyState
                    .padding(.vertical, 12)
            } else {
                ForEach(store.visibleSessions) { session in
                    sessionSection(session)
                }
            }

            Divider().padding(.vertical, 4)

            Button {
                openConfigure()
            } label: {
                Label(l10n.string("menu.configure"), systemImage: "gear")
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .contentShape(Rectangle())
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
            }
            .buttonStyle(.plain)

            Button {
                NSApplication.shared.terminate(nil)
            } label: {
                Label(l10n.string("menu.quit"), systemImage: "power")
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .contentShape(Rectangle())
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
            }
            .buttonStyle(.plain)

            Text(verbatim: "HerdrBell v\(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "?")")
                .font(.caption2)
                .foregroundStyle(.tertiary)
                .frame(maxWidth: .infinity)
                .padding(.bottom, 6)
        }
        .frame(width: 280)
        .task {
            await store.refreshNow()
        }
    }

    /// Opens the Configure window and reliably grabs focus. This is an
    /// LSUIElement menu bar app, so the window is owned and shown directly
    /// by `ConfigureWindow` — `makeKeyAndOrderFront` plus app activation
    /// brings it forward even when it was already open behind other apps.
    private func openConfigure() {
        dismiss()
        ConfigureWindow.show()
    }

    @ViewBuilder
    private var emptyState: some View {
        HStack(spacing: 8) {
            StatusIconView(icon: store.scheme.emptyStateIcon, assetSize: 16)
                .foregroundStyle(.secondary)
            Text(store.sessions.isEmpty ? l10n.string("menu.noSessions") : l10n.string("menu.connecting"))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 8)
    }

    @ViewBuilder
    private func sessionSection(_ session: HerdrStore.SessionView) -> some View {
        Text(verbatim: session.name)
            .font(.caption)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 8)
            .padding(.top, 6)

        if session.agents.isEmpty {
            Text(l10n.string("menu.noAgents"))
                .font(.caption)
                .foregroundStyle(.tertiary)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
        } else {
            ForEach(session.agents) { agent in
                Button {
                    store.focus(agent, in: session.name)
                    dismiss()
                } label: {
                    HStack(spacing: 8) {
                        StabilizedStatusIcon(
                            status: agent.status,
                            scheme: store.scheme,
                            settleDelay: store.idleSettleDelay
                        )
                        VStack(alignment: .leading, spacing: 0) {
                            Text(verbatim: agent.agent)
                                .lineLimit(1)
                            Text(verbatim: agent.title)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                        Spacer()
                        if agent.focused {
                            StatusIconView(icon: store.scheme.focusedMarkerIcon, assetSize: 10)
                                .font(.caption)
                                .foregroundStyle(.tertiary)
                        }
                    }
                    .contentShape(Rectangle())
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                }
                .buttonStyle(.plain)
                .help(agent.title)
            }
        }
    }
}
