use herdrbell::wire::*;

#[test]
fn test_decode_ping() {
    let json = std::fs::read_to_string("tests/fixtures/ping.json").unwrap();
    let response: HerdrResponse = serde_json::from_str(&json).unwrap();
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let ping: PingResult = serde_json::from_value(result).unwrap();
    assert_eq!(ping.version, "0.8.2");
    assert_eq!(ping.protocol_version, 20);
}

#[test]
fn test_decode_agent_list() {
    let json = std::fs::read_to_string("tests/fixtures/agent_list.json").unwrap();
    let response: HerdrResponse = serde_json::from_str(&json).unwrap();
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let agent_list: AgentListResult = serde_json::from_value(result).unwrap();
    assert_eq!(agent_list.agents.len(), 1);
    let agent = &agent_list.agents[0];
    assert_eq!(agent.agent.as_deref(), Some("opencode"));
    assert_eq!(agent.agent_status, AgentStatus::Working);
    assert_eq!(agent.pane_id.as_deref(), Some("w8:p6"));
    assert!(agent.focused.unwrap_or(false));
}

#[test]
fn test_decode_session_snapshot() {
    let json = std::fs::read_to_string("tests/fixtures/session_snapshot.json").unwrap();
    let response: HerdrResponse = serde_json::from_str(&json).unwrap();
    let result = response.result.unwrap();
    let snapshot_result: SnapshotResult = serde_json::from_value(result).unwrap();
    let snapshot = snapshot_result.snapshot;
    assert_eq!(snapshot.agents.len(), 1);
    assert_eq!(snapshot.workspaces.len(), 1);
    assert_eq!(snapshot.tabs.len(), 1);
    assert_eq!(snapshot.panes.len(), 1);
    assert_eq!(snapshot.focused_pane_id.as_deref(), Some("w8:p6"));
}

#[test]
fn test_decode_subscription_started() {
    let json = std::fs::read_to_string("tests/fixtures/subscription_started.json").unwrap();
    let response: HerdrResponse = serde_json::from_str(&json).unwrap();
    assert!(response.error.is_none());
    assert_eq!(response.id.as_deref(), Some("req_1"));
}

#[test]
fn test_parse_event_frame() {
    let json = std::fs::read_to_string("tests/fixtures/event_pane_agent_status_changed.json").unwrap();
    let frame = parse_frame(json.as_bytes()).unwrap();
    match frame {
        HerdrFrame::Event(envelope) => {
            assert_eq!(envelope.event, "pane_agent_status_changed");
            let data: AgentStatusChangedEventData = serde_json::from_value(envelope.data).unwrap();
            assert_eq!(data.agent_status, AgentStatus::Blocked);
            assert_eq!(data.pane_id, "w8:p6");
            assert_eq!(data.agent.as_deref(), Some("opencode"));
        }
        _ => panic!("expected event frame"),
    }
}

#[test]
fn test_parse_pane_created_event() {
    let json = std::fs::read_to_string("tests/fixtures/event_pane_created.json").unwrap();
    let frame = parse_frame(json.as_bytes()).unwrap();
    match frame {
        HerdrFrame::Event(envelope) => {
            assert_eq!(envelope.event, "pane_created");
            let data: PaneEventData = serde_json::from_value(envelope.data).unwrap();
            assert_eq!(data.pane.pane_id, "w8:p6");
            assert_eq!(data.pane.agent_status, Some(AgentStatus::Unknown));
        }
        _ => panic!("expected event frame"),
    }
}

#[test]
fn test_parse_pane_updated_event() {
    let json = std::fs::read_to_string("tests/fixtures/event_pane_updated.json").unwrap();
    let frame = parse_frame(json.as_bytes()).unwrap();
    match frame {
        HerdrFrame::Event(envelope) => {
            assert_eq!(envelope.event, "pane_updated");
            let data: PaneEventData = serde_json::from_value(envelope.data).unwrap();
            assert_eq!(data.pane.pane_id, "w8:p5");
            assert!(data.pane.agent_session.is_some());
        }
        _ => panic!("expected event frame"),
    }
}

#[test]
fn test_parse_response_frame() {
    let json = std::fs::read_to_string("tests/fixtures/ping.json").unwrap();
    let frame = parse_frame(json.as_bytes()).unwrap();
    match frame {
        HerdrFrame::Response(resp) => {
            assert_eq!(resp.id.as_deref(), Some("req_1"));
            assert!(resp.error.is_none());
        }
        _ => panic!("expected response frame"),
    }
}

#[test]
fn test_agent_status_unknown_fallback() {
    let json = r#""something_new""#;
    let status: AgentStatus = serde_json::from_str(json).unwrap();
    assert_eq!(status, AgentStatus::Unknown);
}

#[test]
fn test_encode_request() {
    let data = encode_request("req_1", "ping", serde_json::json!({}));
    assert!(data.ends_with(b"\n"));
    let line = &data[..data.len() - 1];
    let v: serde_json::Value = serde_json::from_slice(line).unwrap();
    assert_eq!(v["id"], "req_1");
    assert_eq!(v["method"], "ping");
}

#[test]
fn test_lifecycle_subscriptions() {
    let subs = lifecycle_subscriptions();
    assert_eq!(subs.len(), 6);
}

#[test]
fn test_current_subscriptions_includes_pane_ids() {
    let pane_ids = vec!["w8:p6".to_string(), "w8:p5".to_string()];
    let subs = current_subscriptions(&pane_ids);
    assert_eq!(subs.len(), 6 + 2);
}
