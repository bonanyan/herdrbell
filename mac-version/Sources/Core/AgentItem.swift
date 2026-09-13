import Foundation

struct AgentItem: Identifiable, Hashable, Sendable {
    let paneId: String
    var workspaceId: String?
    var agent: String
    var title: String
    var status: AgentStatus
    var focused: Bool
    var cwd: String?

    var id: String { paneId }
}

extension AgentItem {
    init(info: AgentInfo) {
        paneId = info.paneId ?? ""
        workspaceId = info.workspaceId
        agent = info.agent ?? "agent"
        title = info.terminalTitleStripped ?? info.terminalTitle ?? info.cwd ?? info.paneId ?? ""
        status = info.agentStatus
        focused = info.focused ?? false
        cwd = info.cwd
    }

    mutating func apply(pane: PaneInfo) -> AgentStatus? {
        let oldStatus = status
        if let agentStatus = pane.agentStatus {
            status = agentStatus
        }
        if let agent = pane.agent {
            self.agent = agent
        }
        if let title = pane.terminalTitleStripped ?? pane.terminalTitle {
            self.title = title
        }
        if let focused = pane.focused {
            self.focused = focused
        }
        if let cwd = pane.cwd {
            self.cwd = cwd
        }
        if let workspaceId = pane.workspaceId {
            self.workspaceId = workspaceId
        }
        return oldStatus == status ? nil : oldStatus
    }
}
