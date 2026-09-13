import Foundation

enum AgentStatus: String, Codable, Sendable, CaseIterable {
    case idle
    case working
    case blocked
    case done
    case unknown

    init(from decoder: Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        self = AgentStatus(rawValue: raw) ?? .unknown
    }
}

struct AgentSession: Codable, Sendable {
    let source: String?
    let agent: String?
    let kind: String?
    let value: String?
}

struct PaneInfo: Codable, Sendable {
    let paneId: String
    let terminalId: String?
    let workspaceId: String?
    let tabId: String?
    let focused: Bool?
    let cwd: String?
    let foregroundCwd: String?
    let agent: String?
    let terminalTitle: String?
    let terminalTitleStripped: String?
    let agentStatus: AgentStatus?
    let agentSession: AgentSession?
    let revision: Int?
}

struct WorkspaceInfo: Codable, Sendable {
    let workspaceId: String
    let number: Int?
    let label: String?
    let focused: Bool?
    let paneCount: Int?
    let tabCount: Int?
    let activeTabId: String?
    let agentStatus: AgentStatus?
}

struct TabInfo: Codable, Sendable {
    let tabId: String
    let workspaceId: String?
    let number: Int?
    let label: String?
    let focused: Bool?
    let paneCount: Int?
    let agentStatus: AgentStatus?
}

struct AgentInfo: Codable, Sendable {
    let terminalId: String?
    let agent: String?
    let terminalTitle: String?
    let terminalTitleStripped: String?
    let agentStatus: AgentStatus
    let screenDetectionSkipped: Bool?
    let agentSession: AgentSession?
    let workspaceId: String?
    let tabId: String?
    let paneId: String?
    let focused: Bool?
    let stateChangeSeq: Int?
    let cwd: String?
    let foregroundCwd: String?
    let revision: Int?
}

struct PingResult: Decodable, Sendable {
    let type: String?
    let version: String
    let protocolVersion: Int
    let capabilities: [String: JSONValue]?

    enum CodingKeys: String, CodingKey {
        case type
        case version
        case capabilities
        case protocolVersion = "protocol"
    }
}

struct SessionSnapshot: Decodable, Sendable {
    let version: String?
    let protocolVersion: Int?
    let focusedWorkspaceId: String?
    let focusedTabId: String?
    let focusedPaneId: String?
    let workspaces: [WorkspaceInfo]
    let tabs: [TabInfo]
    let panes: [PaneInfo]
    let layouts: [JSONValue]?
    let agents: [AgentInfo]

    enum CodingKeys: String, CodingKey {
        case version
        case focusedWorkspaceId
        case focusedTabId
        case focusedPaneId
        case workspaces
        case tabs
        case panes
        case layouts
        case agents
        case protocolVersion = "protocol"
    }
}

struct AgentListResult: Decodable, Sendable {
    let type: String?
    let agents: [AgentInfo]
}

struct SnapshotResult: Decodable, Sendable {
    let type: String?
    let snapshot: SessionSnapshot
}

struct SubscriptionStartedResult: Decodable, Sendable {
    let type: String?
}

struct HerdrRequest: Encodable, Sendable {
    let id: String
    let method: String
    let params: JSONValue
}

struct HerdrErrorInfo: Codable, Sendable {
    let code: String
    let message: String
}

struct HerdrResponse: Decodable, Sendable {
    let id: String?
    let result: JSONValue?
    let error: HerdrErrorInfo?
}

struct HerdrEventEnvelope: Decodable, Sendable {
    let event: String
    let data: JSONValue
}

struct PaneEventData: Decodable, Sendable {
    let type: String
    let pane: PaneInfo
}

struct PaneIdEventData: Decodable, Sendable {
    let type: String
    let paneId: String
    let workspaceId: String
}

struct AgentStatusChangedEventData: Decodable, Sendable {
    let type: String
    let paneId: String
    let workspaceId: String
    let agentStatus: AgentStatus
    let agent: String?
    let displayAgent: String?
    let title: String?
    let stateLabels: [String: String]?
}

struct AgentDetectedEventData: Decodable, Sendable {
    let type: String
    let paneId: String
    let workspaceId: String
    let agent: String?
    let finalStatus: Bool?
    let released: Bool?
}

struct WorkspaceClosedEventData: Decodable, Sendable {
    let type: String
    let workspaceId: String
    let workspace: JSONValue?
}

enum HerdrFrame: Sendable {
    case response(HerdrResponse)
    case event(HerdrEventEnvelope)
}

enum Wire {
    static let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }()

    static let encoder = JSONEncoder()

    static func encodeRequest(id: String, method: String, params: JSONValue = .object([:])) throws -> Data {
        var data = try encoder.encode(HerdrRequest(id: id, method: method, params: params))
        data.append(0x0A)
        return data
    }

    static func parseFrame(_ line: Data) throws -> HerdrFrame {
        struct Probe: Decodable {
            let id: String?
            let event: String?
        }
        let probe = try decoder.decode(Probe.self, from: line)
        if probe.event != nil {
            return .event(try decoder.decode(HerdrEventEnvelope.self, from: line))
        }
        return .response(try decoder.decode(HerdrResponse.self, from: line))
    }

    static func decodeResult<T: Decodable>(_ result: JSONValue, as type: T.Type) throws -> T {
        try decoder.decode(T.self, from: encoder.encode(result))
    }
}
