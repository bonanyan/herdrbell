import Foundation
import Testing

@testable import HerdrBell

private func liveSocket() throws -> HerdrSocket {
    let path = ("~/.config/herdr/herdr.sock" as NSString).expandingTildeInPath
    try #require(
        FileManager.default.fileExists(atPath: path),
        "herdr server not running; skipping live test"
    )
    return HerdrSocket(path: path)
}

@Test func livePing() async throws {
    let socket = try liveSocket()
    let pong = try await socket.ping()
    #expect(!pong.version.isEmpty)
    #expect(pong.protocolVersion > 0)
}

@Test func liveSessionSnapshot() async throws {
    let socket = try liveSocket()
    let snapshot = try await socket.sessionSnapshot()
    #expect(!snapshot.panes.isEmpty)
    #expect(snapshot.workspaces.count >= 1)
}

@Test func liveAgentList() async throws {
    let socket = try liveSocket()
    let agents = try await socket.agentList()
    #expect(!agents.isEmpty)
    for agent in agents {
        #expect(AgentStatus.allCases.contains(agent.agentStatus))
    }
}

@Test func liveOneShotIsOneConnectionPerRequest() async throws {
    let socket = try liveSocket()
    _ = try await socket.ping()
    _ = try await socket.ping()
}

@Test func liveAgentFocusAcceptsPaneId() async throws {
    let socket = try liveSocket()
    let agents = try await socket.agentList()
    let target = try #require(agents.first(where: { $0.focused == true }) ?? agents.first)
    let paneId = try #require(target.paneId)
    try await socket.agentFocus(target: paneId)
}
