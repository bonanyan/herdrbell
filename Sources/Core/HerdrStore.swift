import Foundation
import Observation

@MainActor
@Observable
final class HerdrStore {
    struct SessionView: Identifiable, Sendable {
        let name: String
        var agents: [AgentItem]
        var connected: Bool

        var id: String { name }
    }

    private(set) var sessions: [SessionView] = []
    private(set) var aggregateIcon: StatusIcon
    private(set) var scheme: any IconScheme

    @ObservationIgnored private var clients: [String: HerdrSessionClient] = [:]
    @ObservationIgnored private let discovery = SessionDiscovery()
    @ObservationIgnored private let notifier = Notifier()
    @ObservationIgnored private var started = false
    @ObservationIgnored let idleSettleDelay: TimeInterval
    @ObservationIgnored private var idleSettleTask: Task<Void, Never>?
    @ObservationIgnored private var defaultsObserver: (any NSObjectProtocol)?

    init(idleSettleDelay: TimeInterval = 0.5) {
        self.idleSettleDelay = idleSettleDelay
        let schemeID = UserDefaults.standard.string(forKey: SettingsKeys.iconSchemeID) ?? IconSchemeRegistry.default.id
        let scheme = IconSchemeRegistry.scheme(id: schemeID)
        self.scheme = scheme
        self.aggregateIcon = scheme.disconnectedIcon
    }

    var visibleSessions: [SessionView] {
        sessions.filter(\.connected).sorted { $0.name < $1.name }
    }

    var hasConnections: Bool {
        !visibleSessions.isEmpty
    }

    private var rawAggregateIcon: StatusIcon {
        if visibleSessions.isEmpty { return scheme.disconnectedIcon }
        let statuses = visibleSessions.flatMap { $0.agents.map(\.status) }
        return scheme.aggregateIcon(for: statuses)
    }

    private func refreshAggregateSymbol() {
        let raw = rawAggregateIcon
        if raw == aggregateIcon {
            idleSettleTask?.cancel()
            idleSettleTask = nil
            return
        }
        if raw != scheme.idleAggregateIcon {
            idleSettleTask?.cancel()
            idleSettleTask = nil
            aggregateIcon = raw
            return
        }
        guard idleSettleTask == nil else { return }
        idleSettleTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(self?.idleSettleDelay ?? 0))
            guard !Task.isCancelled else { return }
            self?.settleToIdleIfStillIdle()
        }
    }

    private func settleToIdleIfStillIdle() {
        idleSettleTask = nil
        if rawAggregateIcon == scheme.idleAggregateIcon {
            aggregateIcon = scheme.idleAggregateIcon
        }
    }

    func enableForTesting() {
        started = true
        observeIconSchemeChanges()
    }

    func addSessionForTesting(name: String) {
        guard !sessions.contains(where: { $0.name == name }) else { return }
        sessions.append(SessionView(name: name, agents: [], connected: false))
        refreshAggregateSymbol()
    }

    func start() {
        guard !started else { return }
        started = true
        observeIconSchemeChanges()
        discovery.onChange = { [weak self] found in
            Task { @MainActor [weak self] in
                await self?.syncClients(with: found)
            }
        }
        discovery.start()
        if ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] == nil {
            Task { [notifier] in
                await notifier.requestAuthorization()
            }
        }
    }

    func stop() {
        guard started else { return }
        started = false
        if let defaultsObserver {
            NotificationCenter.default.removeObserver(defaultsObserver)
            self.defaultsObserver = nil
        }
        discovery.stop()
        for client in clients.values {
            Task { await client.stop() }
        }
        clients.removeAll()
        sessions.removeAll()
        idleSettleTask?.cancel()
        idleSettleTask = nil
        aggregateIcon = scheme.disconnectedIcon
    }

    /// Live-switches the icon scheme when the Configure window changes it.
    private func observeIconSchemeChanges() {
        guard defaultsObserver == nil else { return }
        defaultsObserver = NotificationCenter.default.addObserver(
            forName: UserDefaults.didChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor [weak self] in
                self?.syncSchemeWithDefaults()
            }
        }
    }

    private func syncSchemeWithDefaults() {
        let schemeID = UserDefaults.standard.string(forKey: SettingsKeys.iconSchemeID) ?? IconSchemeRegistry.default.id
        guard scheme.id != schemeID else { return }
        scheme = IconSchemeRegistry.scheme(id: schemeID)
        idleSettleTask?.cancel()
        idleSettleTask = nil
        aggregateIcon = rawAggregateIcon
    }

    /// Pulls fresh agent state from every session — invoked when the menu opens
    /// so the rows always match what herdr reports right now.
    func refreshNow() async {
        guard started else { return }
        let clients = Array(self.clients.values)
        for client in clients {
            await client.refreshNow()
        }
    }

    func focus(_ agent: AgentItem, in sessionName: String) {
        guard let client = clients[sessionName] else { return }
        Task {
            try? await client.focus(paneId: agent.paneId)
        }
    }

    private func syncClients(with found: [DiscoveredSession]) async {
        guard started else { return }
        let foundNames = Set(found.map(\.name))
        for (name, client) in clients where !foundNames.contains(name) {
            await client.stop()
            clients.removeValue(forKey: name)
            sessions.removeAll { $0.name == name }
        }
        for session in found where clients[session.name] == nil {
            let client = HerdrSessionClient(sessionName: session.name, socketPath: session.socketPath) { [weak self] event in
                Task { @MainActor [weak self] in
                    await self?.handle(event)
                }
            }
            clients[session.name] = client
            sessions.append(SessionView(name: session.name, agents: [], connected: false))
            await client.start()
        }
        refreshAggregateSymbol()
    }

    func handle(_ event: HerdrSessionClient.ClientEvent) async {
        guard started else { return }
        switch event {
        case .connectionChanged(let name, let connected):
            if let index = sessions.firstIndex(where: { $0.name == name }) {
                sessions[index].connected = connected
                refreshAggregateSymbol()
            }
        case .agentsChanged(let name, let agents):
            if let index = sessions.firstIndex(where: { $0.name == name }) {
                sessions[index].agents = agents
                sessions[index].connected = true
                refreshAggregateSymbol()
            }
        case .statusChanged(let name, let paneId, let from, let to):
            guard NotificationPolicy.shouldNotify(from: from, to: to) else { break }
            guard let session = sessions.first(where: { $0.name == name }),
                  let agent = session.agents.first(where: { $0.paneId == paneId })
            else { break }
            let l10n = LocalizationManager.shared
            let titleKey = to == .blocked ? "notification.blocked.title" : "notification.done.title"
            let body = agent.title.isEmpty
                ? l10n.string("notification.body.session", name)
                : agent.title
            let appearance = scheme.appearance(for: to)
            notifier.post(
                title: l10n.string(titleKey, agent.agent),
                body: body,
                icon: appearance.icon,
                tint: appearance.color
            )
        }
    }
}
