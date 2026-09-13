use herdrbell::wire::{AgentInfo, AgentStatus, PaneInfo};
use herdrbell::core::agent_item::AgentItem;

fn make_agent_info(pane_id: &str, agent: &str, status: AgentStatus) -> AgentInfo {
    AgentInfo {
        terminal_id: Some("term_1".into()),
        agent: Some(agent.into()),
        terminal_title: Some("title".into()),
        terminal_title_stripped: Some("title_stripped".into()),
        agent_status: status,
        screen_detection_skipped: None,
        agent_session: None,
        workspace_id: Some("w1".into()),
        tab_id: Some("w1:t1".into()),
        pane_id: Some(pane_id.into()),
        focused: Some(false),
        state_change_seq: None,
        cwd: Some("/home/user".into()),
        foreground_cwd: None,
        revision: None,
    }
}

#[test]
fn test_agent_item_from_info() {
    let info = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let item = AgentItem::from_info(&info);
    assert_eq!(item.pane_id, "w1:p1");
    assert_eq!(item.agent, "opencode");
    assert_eq!(item.title, "title_stripped");
    assert_eq!(item.status, AgentStatus::Working);
    assert!(!item.focused);
}

#[test]
fn test_agent_item_apply_pane_updates_status() {
    let info = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let mut item = AgentItem::from_info(&info);

    let pane = PaneInfo {
        pane_id: "w1:p1".into(),
        terminal_id: None,
        workspace_id: None,
        tab_id: None,
        focused: Some(true),
        cwd: None,
        foreground_cwd: None,
        agent: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: Some(AgentStatus::Blocked),
        agent_session: None,
        revision: None,
    };

    let old = item.apply_pane(&pane);
    assert_eq!(old, Some(AgentStatus::Working));
    assert_eq!(item.status, AgentStatus::Blocked);
    assert!(item.focused);
}

#[test]
fn test_agent_item_apply_pane_no_change() {
    let info = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let mut item = AgentItem::from_info(&info);

    let pane = PaneInfo {
        pane_id: "w1:p1".into(),
        terminal_id: None,
        workspace_id: None,
        tab_id: None,
        focused: None,
        cwd: None,
        foreground_cwd: None,
        agent: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: Some(AgentStatus::Working),
        agent_session: None,
        revision: None,
    };

    let old = item.apply_pane(&pane);
    assert_eq!(old, None);
}

#[test]
fn test_agent_item_equality() {
    let info1 = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let info2 = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let item1 = AgentItem::from_info(&info1);
    let item2 = AgentItem::from_info(&info2);
    assert_eq!(item1, item2);
}

#[test]
fn test_agent_item_inequality_on_status() {
    let info1 = make_agent_info("w1:p1", "opencode", AgentStatus::Working);
    let info2 = make_agent_info("w1:p1", "opencode", AgentStatus::Blocked);
    let item1 = AgentItem::from_info(&info1);
    let item2 = AgentItem::from_info(&info2);
    assert_ne!(item1, item2);
}
