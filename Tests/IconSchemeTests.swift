import Foundation
import Testing

@testable import HerdrBell

@Test
func registryHasUniqueIdsAndResolvesDefaults() {
    let ids = IconSchemeRegistry.all.map(\.id)
    #expect(Set(ids).count == ids.count)
    #expect(IconSchemeRegistry.scheme(id: "does-not-exist").id == IconSchemeRegistry.default.id)
    #expect(IconSchemeRegistry.scheme(id: "classic").id == "classic")
    #expect(IconSchemeRegistry.scheme(id: "custom").id == "custom")
}

@Test
func classicSchemeCoversEveryStatus() {
    let scheme = ClassicIconScheme()
    let statuses: [AgentStatus] = [.idle, .working, .blocked, .done, .unknown]
    for status in statuses {
        guard case .systemSymbol(let name) = scheme.appearance(for: status).icon else {
            Issue.record("classic scheme should use SF Symbols")
            return
        }
        #expect(!name.isEmpty)
    }
    #expect(scheme.aggregateIcon(for: [.idle, .working, .blocked, .done])
        == scheme.aggregateIcon(for: [.blocked]))
    #expect(scheme.aggregateIcon(for: [.idle, .working, .done])
        == scheme.aggregateIcon(for: [.done]))
    #expect(scheme.aggregateIcon(for: [.idle, .done])
        == scheme.aggregateIcon(for: [.done]))
    #expect(scheme.aggregateIcon(for: [.idle, .unknown])
        == scheme.aggregateIcon(for: [.unknown]))
    #expect(scheme.aggregateIcon(for: []) == scheme.idleAggregateIcon)
}

@Test
func customSchemeCoversEveryStatusWithFallbacks() {
    let scheme = CustomIconScheme()
    let expectedAssets: [AgentStatus: String] = [
        .blocked: "status-blocked",
        .working: "status-working",
        .done: "status-done",
        .idle: "status-idle",
        .unknown: "status-unknown",
    ]
    for (status, assetName) in expectedAssets {
        guard case .asset(let name, let fallback) = scheme.appearance(for: status).icon else {
            Issue.record("custom scheme should use assets for \(status)")
            return
        }
        #expect(name == assetName)
        #expect(!fallback.isEmpty)
    }
    #expect(scheme.aggregateIcon(for: [.blocked]) == scheme.appearance(for: .blocked).icon)
    #expect(scheme.aggregateIcon(for: [.working]) == scheme.appearance(for: .working).icon)
    #expect(scheme.aggregateIcon(for: [.done]) == scheme.appearance(for: .done).icon)
    #expect(scheme.aggregateIcon(for: [.idle]) == scheme.idleAggregateIcon)
    #expect(scheme.aggregateIcon(for: [.unknown]) == scheme.appearance(for: .unknown).icon)
    #expect(scheme.aggregateIcon(for: [.working, .idle]) == scheme.idleAggregateIcon)
    #expect(scheme.emptyStateIcon == scheme.disconnectedIcon)
}

@Test
func aggregatePriorityIsBlockedUnknownDoneThenAllWorking() {
    for scheme in IconSchemeRegistry.all {
        #expect(scheme.aggregateIcon(for: [.working, .working]) == scheme.aggregateIcon(for: [.working]))
        #expect(scheme.aggregateIcon(for: [.working, .idle]) == scheme.idleAggregateIcon)
        #expect(scheme.aggregateIcon(for: [.idle]) == scheme.idleAggregateIcon)
        #expect(scheme.aggregateIcon(for: [.unknown]) != scheme.idleAggregateIcon)
        #expect(scheme.aggregateIcon(for: [.done, .unknown]) == scheme.aggregateIcon(for: [.unknown]))
        #expect(scheme.aggregateIcon(for: [.working, .done]) == scheme.aggregateIcon(for: [.done]))
        #expect(scheme.aggregateIcon(for: [.blocked, .unknown, .done]) == scheme.aggregateIcon(for: [.blocked]))
    }
}

@Test
func everySchemeDisplayNameKeyIsLocalizable() {
    for scheme in IconSchemeRegistry.all {
        #expect(!scheme.displayNameKey.isEmpty)
    }
}
