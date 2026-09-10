import Foundation
import Testing

@testable import HerdrBell

private final class Flag: @unchecked Sendable {
    private let lock = NSLock()
    private var stored = false

    func set() {
        lock.lock()
        stored = true
        lock.unlock()
    }

    var value: Bool {
        lock.lock()
        defer { lock.unlock() }
        return stored
    }
}

private func waitUntil(_ seconds: TimeInterval, _ condition: @escaping @Sendable () -> Bool) async -> Bool {
    let deadline = Date().addingTimeInterval(seconds)
    while Date() < deadline {
        if condition() { return true }
        try? await Task.sleep(for: .milliseconds(50))
    }
    return condition()
}

/// A live subscription holds its socket connection open indefinitely, so it must
/// never block the one-shot requests (`agent.list` refresh, `agent.focus` from a
/// menu row click) that share the same `HerdrSocket`.
@Test func oneShotRequestIsNotStarvedByOpenSubscription() async throws {
    let path = "/tmp/herdrbell_starve_\(ProcessInfo.processInfo.processIdentifier).sock"
    let server = FakeHerdrServer(path: path)
    try server.start()
    defer { server.stop() }

    let socket = HerdrSocket(path: path)
    let subscription = await socket.subscribe([.object(["type": .string("pane.updated")])])

    let completed = Flag()
    let probe = Task {
        _ = try? await socket.agentList()
        completed.set()
    }

    #expect(
        await waitUntil(5) { completed.value },
        "agent.list must complete while a subscription stream is open"
    )

    probe.cancel()
    withExtendedLifetime(subscription) {}
}

/// Abandoning a subscription (resubscribing after a pane lifecycle event) must
/// release the connection deterministically instead of leaking a blocked reader.
@Test func abandonedSubscriptionReleasesSocket() async throws {
    let path = "/tmp/herdrbell_release_\(ProcessInfo.processInfo.processIdentifier).sock"
    let server = FakeHerdrServer(path: path)
    try server.start()
    defer { server.stop() }

    let socket = HerdrSocket(path: path)
    let subscription = await socket.subscribe([.object(["type": .string("pane.updated")])])
    await subscription.cancel()

    let completed = Flag()
    let probe = Task {
        _ = try? await socket.agentList()
        completed.set()
    }

    #expect(
        await waitUntil(5) { completed.value },
        "agent.list must complete after the subscription was cancelled"
    )

    probe.cancel()
}
