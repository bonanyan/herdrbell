import Foundation

actor HerdrSessionClient {
    enum ClientEvent: Sendable {
        case connectionChanged(sessionName: String, connected: Bool)
        case agentsChanged(sessionName: String, agents: [AgentItem])
        case statusChanged(sessionName: String, paneId: String, from: AgentStatus?, to: AgentStatus)
    }

    let sessionName: String

    private let socket: HerdrSocket
    private let onEvent: @Sendable (ClientEvent) -> Void
    private var agentsByPane: [String: AgentItem] = [:]
    private var lastEmitted: [AgentItem] = []
    private var needsResubscribe = false
    private var running = false
    private var runTask: Task<Void, Never>?

    /// Safety net for missed events: the stream is the primary source of truth,
    /// this only guarantees stale state self-heals within a few seconds.
    private static let refreshInterval: TimeInterval = 5

    private static let lifecycleSubscriptions: [JSONValue] = [
        .object(["type": .string("pane.created")]),
        .object(["type": .string("pane.closed")]),
        .object(["type": .string("pane.exited")]),
        .object(["type": .string("pane.updated")]),
        .object(["type": .string("pane.agent_detected")]),
        .object(["type": .string("workspace.closed")]),
    ]

    init(sessionName: String, socketPath: String, onEvent: @escaping @Sendable (ClientEvent) -> Void) {
        self.sessionName = sessionName
        self.socket = HerdrSocket(path: socketPath)
        self.onEvent = onEvent
    }

    func start() {
        guard !running else { return }
        running = true
        runTask = Task { [weak self] in
            await self?.runLoop()
        }
    }

    func stop() {
        running = false
        runTask?.cancel()
        runTask = nil
    }

    func focus(paneId: String) async throws {
        try await socket.agentFocus(target: paneId)
    }

    /// Re-reads agent state on demand — called when the menu opens, so rows can
    /// never be staler than the moment the user looks at them.
    func refreshNow() async {
        guard running else { return }
        try? await refreshAgentList()
    }

    private func runLoop() async {
        var delay: TimeInterval = 1
        while running && !Task.isCancelled {
            var connected = false
            do {
                _ = try await socket.ping()
                let snapshot = try await socket.sessionSnapshot()
                applyAgents(snapshot.agents)
                emitAgentsIfChanged()
                connected = true
                delay = 1
                onEvent(.connectionChanged(sessionName: sessionName, connected: true))
                while running && !Task.isCancelled {
                    needsResubscribe = false
                    do {
                        try await liveOnce()
                    } catch {
                        break
                    }
                    guard needsResubscribe else { break }
                    try await refreshAgentList()
                }
            } catch {
                // fall through to reconnect
            }
            if connected {
                onEvent(.connectionChanged(sessionName: sessionName, connected: false))
            }
            guard running && !Task.isCancelled else { break }
            try? await Task.sleep(for: .seconds(delay))
            delay = min(delay * 2, 30)
        }
    }

    private func liveOnce() async throws {
        let subscription = await socket.subscribe(currentSubscriptions())
        defer { subscription.cancel() }
        try await withThrowingTaskGroup(of: Void.self) { group in
            group.addTask { [weak self] in
                guard let self else { return }
                try await self.consume(subscription.events)
            }
            group.addTask { [weak self] in
                guard let self else { return }
                try await self.poll()
            }
            try await group.next()
            group.cancelAll()
        }
    }

    private func consume(_ stream: AsyncThrowingStream<HerdrEventEnvelope, Error>) async throws {
        for try await envelope in stream {
            guard running else { return }
            handle(envelope)
            if needsResubscribe { return }
        }
        throw HerdrSocketError.closed
    }

    private func poll() async throws {
        while running && !Task.isCancelled && !needsResubscribe {
            try await Task.sleep(for: .seconds(Self.refreshInterval))
            guard running, !needsResubscribe else { return }
            try await refreshAgentList()
        }
    }

    private func currentSubscriptions() -> [JSONValue] {
        var subscriptions = Self.lifecycleSubscriptions
        for paneId in agentsByPane.keys.sorted() {
            subscriptions.append(.object([
                "type": .string("pane.agent_status_changed"),
                "pane_id": .string(paneId),
            ]))
        }
        return subscriptions
    }

    private func handle(_ envelope: HerdrEventEnvelope) {
        switch envelope.event {
        case "pane_created", "pane_updated":
            guard let data = try? Wire.decodeResult(envelope.data, as: PaneEventData.self) else { return }
            applyPaneUpdate(data.pane)
        case "pane_agent_status_changed":
            guard let data = try? Wire.decodeResult(envelope.data, as: AgentStatusChangedEventData.self) else { return }
            applyStatusChange(data)
        case "pane_closed", "pane_exited":
            guard let data = try? Wire.decodeResult(envelope.data, as: PaneIdEventData.self) else { return }
            if agentsByPane.removeValue(forKey: data.paneId) != nil {
                emitAgentsIfChanged()
            }
            needsResubscribe = true
        case "pane_agent_detected", "workspace_closed":
            needsResubscribe = true
        default:
            break
        }
    }

    private func applyPaneUpdate(_ pane: PaneInfo) {
        if var item = agentsByPane[pane.paneId] {
            if let oldStatus = item.apply(pane: pane) {
                onEvent(.statusChanged(sessionName: sessionName, paneId: pane.paneId, from: oldStatus, to: item.status))
            }
            agentsByPane[pane.paneId] = item
            emitAgentsIfChanged()
        } else if let agent = pane.agent, !agent.isEmpty, pane.agentStatus != nil {
            var item = AgentItem(info: AgentInfo(
                terminalId: pane.terminalId,
                agent: pane.agent,
                terminalTitle: pane.terminalTitle,
                terminalTitleStripped: pane.terminalTitleStripped,
                agentStatus: pane.agentStatus ?? .unknown,
                screenDetectionSkipped: nil,
                agentSession: pane.agentSession,
                workspaceId: pane.workspaceId,
                tabId: pane.tabId,
                paneId: pane.paneId,
                focused: pane.focused,
                stateChangeSeq: nil,
                cwd: pane.cwd,
                foregroundCwd: pane.foregroundCwd,
                revision: pane.revision
            ))
            item.status = pane.agentStatus ?? item.status
            agentsByPane[pane.paneId] = item
            needsResubscribe = true
            emitAgentsIfChanged()
        }
    }

    private func applyStatusChange(_ data: AgentStatusChangedEventData) {
        if var item = agentsByPane[data.paneId] {
            let oldStatus = item.status
            item.status = data.agentStatus
            if let agent = data.agent {
                item.agent = agent
            }
            if let title = data.title, !title.isEmpty {
                item.title = title
            }
            agentsByPane[data.paneId] = item
            if oldStatus != data.agentStatus {
                onEvent(.statusChanged(sessionName: sessionName, paneId: data.paneId, from: oldStatus, to: data.agentStatus))
            }
            emitAgentsIfChanged()
        } else {
            needsResubscribe = true
        }
    }

    private func refreshAgentList() async throws {
        let agents = try await socket.agentList()
        applyAgents(agents)
        emitAgentsIfChanged()
    }

    private func applyAgents(_ agents: [AgentInfo]) {
        var rebuilt: [String: AgentItem] = [:]
        for info in agents {
            guard let paneId = info.paneId, !paneId.isEmpty else { continue }
            rebuilt[paneId] = AgentItem(info: info)
        }
        agentsByPane = rebuilt
    }

    private func emitAgentsIfChanged() {
        let current = agentsByPane.values.sorted { $0.paneId < $1.paneId }
        guard current != lastEmitted else { return }
        lastEmitted = current
        onEvent(.agentsChanged(sessionName: sessionName, agents: current))
    }
}
