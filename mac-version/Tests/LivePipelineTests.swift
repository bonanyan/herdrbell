import Foundation
import Testing

@testable import HerdrBell

private final class SessionCollector: @unchecked Sendable {
    private let lock = NSLock()
    private var stored: [DiscoveredSession] = []

    func set(_ sessions: [DiscoveredSession]) {
        lock.lock()
        stored = sessions
        lock.unlock()
    }

    var sessions: [DiscoveredSession] {
        lock.lock()
        defer { lock.unlock() }
        return stored
    }
}

private func defaultSocketPath() throws -> String {
    let path = ("~/.config/herdr/herdr.sock" as NSString).expandingTildeInPath
    try #require(
        FileManager.default.fileExists(atPath: path),
        "herdr server not running; skipping live test"
    )
    return path
}

@Test func liveClientPublishesAgents() async throws {
    let collector = EventCollector()
    let client = HerdrSessionClient(
        sessionName: "default",
        socketPath: try defaultSocketPath()
    ) { event in
        collector.add(event)
    }
    await client.start()
    let deadline = Date().addingTimeInterval(5)
    var agents: [AgentItem] = []
    while Date() < deadline && agents.isEmpty {
        try await Task.sleep(for: .milliseconds(200))
        for event in collector.events {
            if case .agentsChanged(_, let list) = event {
                agents = list
            }
        }
    }
    await client.stop()
    #expect(!agents.isEmpty)
}

@Test func liveDiscoveryFindsDefaultSession() async throws {
    _ = try defaultSocketPath()
    let collector = SessionCollector()
    let discovery = SessionDiscovery()
    discovery.onChange = { found in
        collector.set(found)
    }
    discovery.start()
    let deadline = Date().addingTimeInterval(5)
    while Date() < deadline && collector.sessions.isEmpty {
        try await Task.sleep(for: .milliseconds(200))
    }
    discovery.stop()
    let names = collector.sessions.map(\.name)
    #expect(names.contains("default"))
}

@Test func liveStoreShowsAgents() async throws {
    _ = try defaultSocketPath()
    let store = await HerdrStore()
    await store.start()
    let deadline = Date().addingTimeInterval(10)
    var snapshot: [HerdrStore.SessionView] = []
    while Date() < deadline {
        snapshot = await store.visibleSessions
        if snapshot.contains(where: { !$0.agents.isEmpty }) {
            break
        }
        try await Task.sleep(for: .milliseconds(200))
    }
    await store.stop()
    let session = try #require(snapshot.first(where: { !$0.agents.isEmpty }))
    #expect(!session.agents.isEmpty)
}
