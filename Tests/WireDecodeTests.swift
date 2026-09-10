import Foundation
import Testing

@testable import HerdrBell

private func fixtureData(_ name: String) throws -> Data {
    let url = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()
        .appendingPathComponent("Fixtures")
        .appendingPathComponent(name)
    return try Data(contentsOf: url)
}

private func fixtureResult<T: Decodable>(_ name: String, as type: T.Type) throws -> T {
    guard case .response(let response) = try Wire.parseFrame(fixtureData(name)) else {
        throw HerdrSocketError.unexpectedFrame
    }
    guard let result = response.result else {
        throw HerdrSocketError.missingResult
    }
    return try Wire.decodeResult(result, as: T.self)
}

private func fixtureEvent(_ name: String) throws -> HerdrEventEnvelope {
    guard case .event(let envelope) = try Wire.parseFrame(fixtureData(name)) else {
        throw HerdrSocketError.unexpectedFrame
    }
    return envelope
}

@Test func decodePing() throws {
    let pong = try fixtureResult("ping.json", as: PingResult.self)
    #expect(pong.version == "0.8.2")
    #expect(pong.protocolVersion == 20)
    #expect(pong.capabilities?["live_handoff"] == .bool(true))
}

@Test func decodeAgentList() throws {
    let result = try fixtureResult("agent_list.json", as: AgentListResult.self)
    #expect(result.agents.count == 1)
    let agent = try #require(result.agents.first)
    #expect(agent.agent == "opencode")
    #expect(agent.agentStatus == .working)
    #expect(agent.paneId == "w8:p6")
    #expect(agent.workspaceId == "w8")
    #expect(agent.agentSession?.value == "ses_f94b3cc58ffeFBiO2AR2DitNxo")
}

@Test func decodeSessionSnapshot() throws {
    let snapshot = try fixtureResult("session_snapshot.json", as: SnapshotResult.self).snapshot
    #expect(snapshot.version == "0.8.2")
    #expect(snapshot.protocolVersion == 20)
    #expect(snapshot.focusedPaneId == "w8:p6")
    #expect(snapshot.workspaces.count == 1)
    #expect(snapshot.tabs.count == 1)
    #expect(snapshot.panes.count == 1)
    let pane = try #require(snapshot.panes.first)
    #expect(pane.paneId == "w8:p6")
    #expect(pane.agentStatus == .working)
    let agent = try #require(snapshot.agents.first)
    #expect(agent.agentStatus == .working)
}

@Test func parseResponseFrame() throws {
    guard case .response(let response) = try Wire.parseFrame(fixtureData("ping.json")) else {
        Issue.record("expected response frame")
        return
    }
    #expect(response.id == "req_1")
    #expect(response.error == nil)
}

@Test func parseSubscriptionStarted() throws {
    guard case .response(let response) = try Wire.parseFrame(fixtureData("subscription_started.json")) else {
        Issue.record("expected response frame")
        return
    }
    let started = try Wire.decodeResult(response.result ?? .null, as: SubscriptionStartedResult.self)
    #expect(started.type == "subscription_started")
}

@Test func decodePaneCreatedEvent() throws {
    let envelope = try fixtureEvent("event_pane_created.json")
    #expect(envelope.event == "pane_created")
    let data = try Wire.decodeResult(envelope.data, as: PaneEventData.self)
    #expect(data.pane.paneId == "w8:p6")
    #expect(data.pane.workspaceId == "w8")
}

@Test func decodePaneUpdatedEvent() throws {
    let envelope = try fixtureEvent("event_pane_updated.json")
    #expect(envelope.event == "pane_updated")
    let data = try Wire.decodeResult(envelope.data, as: PaneEventData.self)
    #expect(data.pane.paneId == "w8:p5")
    #expect(data.pane.agentStatus != nil)
}

@Test func decodeAgentStatusChangedEvent() throws {
    let envelope = try fixtureEvent("event_pane_agent_status_changed.json")
    #expect(envelope.event == "pane_agent_status_changed")
    let data = try Wire.decodeResult(envelope.data, as: AgentStatusChangedEventData.self)
    #expect(data.paneId == "w8:p6")
    #expect(data.agentStatus == .blocked)
    #expect(data.agent == "opencode")
}

@Test func decodeServerErrorFrame() throws {
    let line = Data(#"{"id":"req_9","error":{"code":"pane_not_found","message":"pane w8:p5 not found"}}"#.utf8)
    guard case .response(let response) = try Wire.parseFrame(line) else {
        Issue.record("expected response frame")
        return
    }
    #expect(response.error?.code == "pane_not_found")
    #expect(response.result == nil)
}

@Test func decodeUnknownStatusTolerated() throws {
    let line = Data(#"{"agent_status":"some_future_status","pane_id":"w1:p1","workspace_id":"w1","type":"pane_agent_status_changed"}"#.utf8)
    let data = try Wire.decoder.decode(AgentStatusChangedEventData.self, from: line)
    #expect(data.agentStatus == .unknown)
}

@Test
func decodeLiveShapedAgentStatusChangedEvent() throws {
    let line = Data(#"{"event":"pane_agent_status_changed","data":{"type":"pane_agent_status_changed","pane_id":"vt:p1","workspace_id":"vt","agent_status":"blocked","agent":"fakeagent","title":"FAKE | verification pane","state_labels":{}}}"#.utf8)
    guard case .event(let envelope) = try Wire.parseFrame(line) else {
        Issue.record("expected an event frame")
        return
    }
    #expect(envelope.event == "pane_agent_status_changed")
    let data = try Wire.decodeResult(envelope.data, as: AgentStatusChangedEventData.self)
    #expect(data.paneId == "vt:p1")
    #expect(data.workspaceId == "vt")
    #expect(data.agentStatus == .blocked)
}
