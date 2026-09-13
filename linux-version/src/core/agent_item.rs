use crate::wire::AgentStatus;

#[derive(Debug, Clone)]
pub struct AgentItem {
    pub pane_id: String,
    pub workspace_id: Option<String>,
    pub agent: String,
    pub title: String,
    pub status: AgentStatus,
    pub focused: bool,
    pub cwd: Option<String>,
}

impl AgentItem {
    pub fn from_info(info: &crate::wire::AgentInfo) -> Self {
        let pane_id = info.pane_id.clone().unwrap_or_default();
        let agent = info.agent.clone().unwrap_or_else(|| "agent".into());
        let title = info
            .terminal_title_stripped
            .clone()
            .or_else(|| info.terminal_title.clone())
            .or_else(|| info.cwd.clone())
            .or_else(|| info.pane_id.clone())
            .unwrap_or_default();
        Self {
            pane_id,
            workspace_id: info.workspace_id.clone(),
            agent,
            title,
            status: info.agent_status,
            focused: info.focused.unwrap_or(false),
            cwd: info.cwd.clone(),
        }
    }

    pub fn apply_pane(&mut self, pane: &crate::wire::PaneInfo) -> Option<AgentStatus> {
        let old_status = self.status;
        if let Some(status) = pane.agent_status {
            self.status = status;
        }
        if let Some(agent) = &pane.agent {
            self.agent = agent.clone();
        }
        if let Some(title) = pane
            .terminal_title_stripped
            .as_ref()
            .or(pane.terminal_title.as_ref())
        {
            self.title = title.clone();
        }
        if let Some(focused) = pane.focused {
            self.focused = focused;
        }
        if let Some(cwd) = &pane.cwd {
            self.cwd = Some(cwd.clone());
        }
        if let Some(workspace_id) = &pane.workspace_id {
            self.workspace_id = Some(workspace_id.clone());
        }
        if old_status == self.status {
            None
        } else {
            Some(old_status)
        }
    }
}

impl PartialEq for AgentItem {
    fn eq(&self, other: &Self) -> bool {
        self.pane_id == other.pane_id
            && self.agent == other.agent
            && self.title == other.title
            && self.status == other.status
            && self.focused == other.focused
    }
}
