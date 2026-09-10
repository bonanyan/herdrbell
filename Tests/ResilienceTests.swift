import Foundation
import Testing

@testable import HerdrBell

@Test func notificationPolicyOnlyBlockedAndDoneTransitions() {
    #expect(NotificationPolicy.shouldNotify(from: .working, to: .blocked))
    #expect(NotificationPolicy.shouldNotify(from: .working, to: .done))
    #expect(NotificationPolicy.shouldNotify(from: .idle, to: .blocked))
    #expect(!NotificationPolicy.shouldNotify(from: .blocked, to: .blocked))
    #expect(!NotificationPolicy.shouldNotify(from: .done, to: .done))
    #expect(!NotificationPolicy.shouldNotify(from: .working, to: .idle))
    #expect(!NotificationPolicy.shouldNotify(from: .blocked, to: .working))
    #expect(!NotificationPolicy.shouldNotify(from: nil, to: .working))
}

@Test func clientSurvivesServerRestart() async throws {
    let path = "/tmp/herdrbell_fake_\(ProcessInfo.processInfo.processIdentifier).sock"
    let server = FakeHerdrServer(path: path)
    try server.start()

    let collector = EventCollector()
    let client = HerdrSessionClient(sessionName: "fake", socketPath: path) { event in
        collector.add(event)
    }
    await client.start()

    #expect(await waitUntil(5) {
        collector.connectedCount >= 1 && collector.latestAgents.contains { $0.agent == "fake" }
    }, "client should connect to fake server")

    server.stop()

    #expect(await waitUntil(10) {
        collector.disconnectedCount >= 1
    }, "client should notice the server went away")

    let revived = FakeHerdrServer(path: path)
    try revived.start()

    #expect(await waitUntil(15) {
        collector.connectedCount >= 2
    }, "client should reconnect after the server comes back")

    await client.stop()
    revived.stop()
}

final class EventCollector: @unchecked Sendable {
    private let lock = NSLock()
    private var stored: [HerdrSessionClient.ClientEvent] = []

    func add(_ event: HerdrSessionClient.ClientEvent) {
        lock.lock()
        stored.append(event)
        lock.unlock()
    }

    var events: [HerdrSessionClient.ClientEvent] {
        lock.lock()
        defer { lock.unlock() }
        return stored
    }

    var connectedCount: Int {
        events.countConnections(connected: true)
    }

    var disconnectedCount: Int {
        events.countConnections(connected: false)
    }

    var latestAgents: [AgentItem] {
        var agents: [AgentItem] = []
        for event in events {
            if case .agentsChanged(_, let list) = event {
                agents = list
            }
        }
        return agents
    }
}

private extension [HerdrSessionClient.ClientEvent] {
    func countConnections(connected: Bool) -> Int {
        var count = 0
        for event in self {
            if case .connectionChanged(_, let value) = event, value == connected {
                count += 1
            }
        }
        return count
    }
}

private func waitUntil(_ seconds: TimeInterval, _ condition: @escaping @Sendable () -> Bool) async -> Bool {
    let deadline = Date().addingTimeInterval(seconds)
    while Date() < deadline {
        if condition() { return true }
        try? await Task.sleep(for: .milliseconds(100))
    }
    return condition()
}

@Test
func clientEmitsStatusChangedForPushedEvent() async throws {
    let path = "/tmp/herdrbell_push_\(ProcessInfo.processInfo.processIdentifier).sock"
    let server = FakeHerdrServer(path: path)
    try server.start()
    defer { server.stop() }

    let collector = EventCollector()
    let client = HerdrSessionClient(sessionName: "fake", socketPath: path) { event in
        collector.add(event)
    }
    await client.start()

    #expect(await waitUntil(5) { collector.latestAgents.contains { $0.paneId == "w1:p1" } },
              "client should publish the snapshot agent")

    await server.pushStatusChange(paneId: "w1:p1", status: "blocked")

    #expect(await waitUntil(5) {
        collector.events.contains { event in
            if case .statusChanged(_, let paneId, _, let to) = event {
                return paneId == "w1:p1" && to == .blocked
            }
            return false
        }
    }, "client should emit statusChanged for a pushed event")

    await client.stop()
}
