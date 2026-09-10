import Foundation
import Testing

@testable import HerdrBell

private func makeAgent(_ status: AgentStatus) -> AgentItem {
    AgentItem(
        paneId: "w1:p1",
        workspaceId: "w1",
        agent: "opencode",
        title: "test",
        status: status,
        focused: false,
        cwd: nil
    )
}

private let scheme = IconSchemeRegistry.default

@MainActor
private func makeConnectedStore(settleDelay: TimeInterval) async -> HerdrStore {
    UserDefaults.standard.set(IconSchemeRegistry.default.id, forKey: SettingsKeys.iconSchemeID)
    let store = HerdrStore(idleSettleDelay: settleDelay)
    store.enableForTesting()
    store.addSessionForTesting(name: "default")
    await store.handle(.connectionChanged(sessionName: "default", connected: true))
    return store
}

@Test @MainActor
func aggregateIconIgnoresIdleBlipsWhileWorking() async throws {
    let store = await makeConnectedStore(settleDelay: 0.4)
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.idle)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    try await Task.sleep(for: .milliseconds(700))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))
}

@Test @MainActor
func aggregateIconSettlesToIdleAfterSustainedIdle() async throws {
    let store = await makeConnectedStore(settleDelay: 0.3)
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.idle)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    try await Task.sleep(for: .milliseconds(600))
    #expect(store.aggregateIcon == scheme.idleAggregateIcon)
}

@Test @MainActor
func shippedIdleSettleDelayKeepsIconNearRealTime() async throws {
    UserDefaults.standard.set(IconSchemeRegistry.default.id, forKey: SettingsKeys.iconSchemeID)
    let store = HerdrStore()
    store.enableForTesting()
    store.addSessionForTesting(name: "default")
    await store.handle(.connectionChanged(sessionName: "default", connected: true))
    #expect(store.idleSettleDelay == 0.5)

    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.idle)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    try await Task.sleep(for: .milliseconds(1_200))
    #expect(store.aggregateIcon == scheme.idleAggregateIcon)
}

@Test @MainActor
func aggregateIconShowsBlockedAndDoneImmediately() async throws {
    let store = await makeConnectedStore(settleDelay: 5)
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))

    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.blocked)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.blocked]))

    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.done)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.done]))
}

@Test @MainActor
func aggregateIconDisconnectIsImmediate() async throws {
    let store = await makeConnectedStore(settleDelay: 5)
    await store.handle(.agentsChanged(sessionName: "default", agents: [makeAgent(.working)]))
    #expect(store.aggregateIcon == scheme.aggregateIcon(for: [AgentStatus.working]))

    await store.handle(.connectionChanged(sessionName: "default", connected: false))
    #expect(store.aggregateIcon == scheme.disconnectedIcon)
}
