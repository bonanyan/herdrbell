use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    #[serde(other)]
    Unknown,
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSession {
    pub source: Option<String>,
    pub agent: Option<String>,
    pub kind: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaneInfo {
    pub pane_id: String,
    pub terminal_id: Option<String>,
    pub workspace_id: Option<String>,
    pub tab_id: Option<String>,
    pub focused: Option<bool>,
    pub cwd: Option<String>,
    pub foreground_cwd: Option<String>,
    pub agent: Option<String>,
    pub terminal_title: Option<String>,
    pub terminal_title_stripped: Option<String>,
    pub agent_status: Option<AgentStatus>,
    pub agent_session: Option<AgentSession>,
    pub revision: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceInfo {
    pub workspace_id: String,
    pub number: Option<i32>,
    pub label: Option<String>,
    pub focused: Option<bool>,
    pub pane_count: Option<i32>,
    pub tab_count: Option<i32>,
    pub active_tab_id: Option<String>,
    pub agent_status: Option<AgentStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TabInfo {
    pub tab_id: String,
    pub workspace_id: Option<String>,
    pub number: Option<i32>,
    pub label: Option<String>,
    pub focused: Option<bool>,
    pub pane_count: Option<i32>,
    pub agent_status: Option<AgentStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentInfo {
    pub terminal_id: Option<String>,
    pub agent: Option<String>,
    pub terminal_title: Option<String>,
    pub terminal_title_stripped: Option<String>,
    pub agent_status: AgentStatus,
    pub screen_detection_skipped: Option<bool>,
    pub agent_session: Option<AgentSession>,
    pub workspace_id: Option<String>,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
    pub focused: Option<bool>,
    pub state_change_seq: Option<i64>,
    pub cwd: Option<String>,
    pub foreground_cwd: Option<String>,
    pub revision: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PingResult {
    pub r#type: Option<String>,
    pub version: String,
    #[serde(alias = "protocol")]
    pub protocol_version: i64,
    pub capabilities: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionSnapshot {
    pub version: Option<String>,
    #[serde(alias = "protocol")]
    pub protocol_version: Option<i64>,
    pub focused_workspace_id: Option<String>,
    pub focused_tab_id: Option<String>,
    pub focused_pane_id: Option<String>,
    pub workspaces: Vec<WorkspaceInfo>,
    pub tabs: Vec<TabInfo>,
    pub panes: Vec<PaneInfo>,
    pub layouts: Option<Vec<Value>>,
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotResult {
    pub r#type: Option<String>,
    pub snapshot: SessionSnapshot,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentListResult {
    pub r#type: Option<String>,
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Serialize)]
pub struct HerdrRequest {
    pub id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HerdrErrorInfo {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HerdrResponse {
    pub id: Option<String>,
    pub result: Option<Value>,
    pub error: Option<HerdrErrorInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HerdrEventEnvelope {
    pub event: String,
    pub data: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaneEventData {
    pub r#type: String,
    pub pane: PaneInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaneIdEventData {
    pub r#type: String,
    pub pane_id: String,
    pub workspace_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentStatusChangedEventData {
    pub r#type: String,
    pub pane_id: String,
    pub workspace_id: String,
    pub agent_status: AgentStatus,
    pub agent: Option<String>,
    pub display_agent: Option<String>,
    pub title: Option<String>,
    pub state_labels: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentDetectedEventData {
    pub r#type: String,
    pub pane_id: String,
    pub workspace_id: String,
    pub agent: Option<String>,
    pub final_status: Option<bool>,
    pub released: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceClosedEventData {
    pub r#type: String,
    pub workspace_id: String,
    pub workspace: Option<Value>,
}

pub enum HerdrFrame {
    Response(HerdrResponse),
    Event(HerdrEventEnvelope),
}

pub fn encode_request(id: &str, method: &str, params: Value) -> Vec<u8> {
    let req = HerdrRequest {
        id: id.to_string(),
        method: method.to_string(),
        params,
    };
    let mut data = serde_json::to_vec(&req).expect("serialize request");
    data.push(b'\n');
    data
}

pub fn parse_frame(line: &[u8]) -> Result<HerdrFrame, serde_json::Error> {
    #[derive(Deserialize)]
    struct Probe {
        #[allow(dead_code)]
        id: Option<String>,
        event: Option<String>,
    }
    let probe: Probe = serde_json::from_slice(line)?;
    if probe.event.is_some() {
        let event: HerdrEventEnvelope = serde_json::from_slice(line)?;
        Ok(HerdrFrame::Event(event))
    } else {
        let response: HerdrResponse = serde_json::from_slice(line)?;
        Ok(HerdrFrame::Response(response))
    }
}

pub fn decode_result<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, serde_json::Error> {
    serde_json::from_value(value.clone())
}

pub fn lifecycle_subscriptions() -> Vec<Value> {
    vec![
        serde_json::json!({"type": "pane.created"}),
        serde_json::json!({"type": "pane.closed"}),
        serde_json::json!({"type": "pane.exited"}),
        serde_json::json!({"type": "pane.updated"}),
        serde_json::json!({"type": "pane.agent_detected"}),
        serde_json::json!({"type": "workspace.closed"}),
    ]
}

pub fn current_subscriptions(pane_ids: &[String]) -> Vec<Value> {
    let mut subs = lifecycle_subscriptions();
    let mut sorted_ids = pane_ids.to_vec();
    sorted_ids.sort();
    for pane_id in sorted_ids {
        subs.push(serde_json::json!({
            "type": "pane.agent_status_changed",
            "pane_id": pane_id,
        }));
    }
    subs
}
