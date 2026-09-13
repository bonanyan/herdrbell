use herdrbell::wire::AgentStatus;
use herdrbell::ui::icons::*;

#[test]
fn test_classic_appearance_blocked() {
    let scheme = ClassicIconScheme;
    let app = scheme.appearance(AgentStatus::Blocked);
    assert_eq!(app.color, (0.9, 0.2, 0.2));
}

#[test]
fn test_classic_appearance_working() {
    let scheme = ClassicIconScheme;
    let app = scheme.appearance(AgentStatus::Working);
    assert_eq!(app.color, (0.2, 0.4, 0.9));
}

#[test]
fn test_classic_appearance_done() {
    let scheme = ClassicIconScheme;
    let app = scheme.appearance(AgentStatus::Done);
    assert_eq!(app.color, (0.2, 0.7, 0.3));
}

#[test]
fn test_aggregate_priority_blocked() {
    let scheme = ClassicIconScheme;
    let statuses = vec![AgentStatus::Working, AgentStatus::Blocked, AgentStatus::Done];
    let icon = scheme.aggregate_icon(&statuses);
    assert!(matches!(icon, StatusIcon::Symbol(ref s) if s == "dialog-warning"));
}

#[test]
fn test_aggregate_priority_done_over_working() {
    let scheme = ClassicIconScheme;
    let statuses = vec![AgentStatus::Working, AgentStatus::Done];
    let icon = scheme.aggregate_icon(&statuses);
    assert!(matches!(icon, StatusIcon::Symbol(ref s) if s == "emblem-ok"));
}

#[test]
fn test_aggregate_all_working() {
    let scheme = ClassicIconScheme;
    let statuses = vec![AgentStatus::Working, AgentStatus::Working];
    let icon = scheme.aggregate_icon(&statuses);
    assert!(matches!(icon, StatusIcon::Symbol(ref s) if s == "view-refresh"));
}

#[test]
fn test_aggregate_all_idle_returns_idle_icon() {
    let scheme = ClassicIconScheme;
    let statuses = vec![AgentStatus::Idle, AgentStatus::Idle];
    let icon = scheme.aggregate_icon(&statuses);
    let idle_icon = scheme.idle_aggregate_icon();
    assert_eq!(icon, idle_icon);
}

#[test]
fn test_aggregate_empty_returns_idle() {
    let scheme = ClassicIconScheme;
    let statuses: Vec<AgentStatus> = vec![];
    let icon = scheme.aggregate_icon(&statuses);
    let idle_icon = scheme.idle_aggregate_icon();
    assert_eq!(icon, idle_icon);
}

#[test]
fn test_aggregate_unknown_priority() {
    let scheme = ClassicIconScheme;
    let statuses = vec![AgentStatus::Working, AgentStatus::Unknown];
    let icon = scheme.aggregate_icon(&statuses);
    assert!(matches!(icon, StatusIcon::Symbol(ref s) if s == "help-questionmark"));
}

#[test]
fn test_disconnected_icon() {
    let scheme = ClassicIconScheme;
    let icon = scheme.disconnected_icon();
    assert!(matches!(icon, StatusIcon::Symbol(ref s) if s == "window-close"));
}

#[test]
fn test_custom_scheme_uses_assets() {
    let scheme = CustomIconScheme;
    let app = scheme.appearance(AgentStatus::Blocked);
    assert!(matches!(app.icon, StatusIcon::Asset { ref name, .. } if name == "status-blocked"));
}

#[test]
fn test_registry_default_is_custom() {
    let scheme = IconSchemeRegistry::default_scheme();
    assert_eq!(scheme.id(), "custom");
}

#[test]
fn test_registry_lookup_by_id() {
    let scheme = IconSchemeRegistry::scheme("classic");
    assert_eq!(scheme.id(), "classic");
}

#[test]
fn test_registry_unknown_falls_back_to_default() {
    let scheme = IconSchemeRegistry::scheme("nonexistent");
    assert_eq!(scheme.id(), "custom");
}

#[test]
fn test_notification_policy_blocked() {
    assert!(NotificationPolicy::should_notify(Some(AgentStatus::Working), AgentStatus::Blocked));
}

#[test]
fn test_notification_policy_done() {
    assert!(NotificationPolicy::should_notify(Some(AgentStatus::Working), AgentStatus::Done));
}

#[test]
fn test_notification_policy_idle_no_notify() {
    assert!(!NotificationPolicy::should_notify(Some(AgentStatus::Working), AgentStatus::Idle));
}

#[test]
fn test_notification_policy_same_status_no_notify() {
    assert!(!NotificationPolicy::should_notify(Some(AgentStatus::Blocked), AgentStatus::Blocked));
}

#[test]
fn test_notification_policy_from_none() {
    assert!(NotificationPolicy::should_notify(None, AgentStatus::Blocked));
}
