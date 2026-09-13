use crate::wire::AgentStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusIcon {
    Symbol(String),
    Asset { name: String, fallback: String },
}

#[derive(Debug, Clone)]
pub struct StatusAppearance {
    pub icon: StatusIcon,
    pub color: (f64, f64, f64),
}

pub trait IconScheme: Send + Sync {
    fn id(&self) -> &str;
    fn display_name_key(&self) -> &str;
    fn appearance(&self, status: AgentStatus) -> StatusAppearance;
    fn aggregate_icon(&self, statuses: &[AgentStatus]) -> StatusIcon;
    fn idle_aggregate_icon(&self) -> StatusIcon;
    fn disconnected_icon(&self) -> StatusIcon;
    fn focused_marker_icon(&self) -> StatusIcon;
    fn empty_state_icon(&self) -> StatusIcon;
}

fn aggregate_status(statuses: &[AgentStatus]) -> AgentStatus {
    let set: std::collections::HashSet<AgentStatus> = statuses.iter().copied().collect();
    if set.contains(&AgentStatus::Blocked) {
        return AgentStatus::Blocked;
    }
    if set.contains(&AgentStatus::Unknown) {
        return AgentStatus::Unknown;
    }
    if set.contains(&AgentStatus::Done) {
        return AgentStatus::Done;
    }
    if !set.is_empty() && set.is_subset(&[AgentStatus::Working].into()) {
        return AgentStatus::Working;
    }
    AgentStatus::Idle
}

pub struct ClassicIconScheme;

impl IconScheme for ClassicIconScheme {
    fn id(&self) -> &str {
        "classic"
    }

    fn display_name_key(&self) -> &str {
        "icon.scheme.classic"
    }

    fn appearance(&self, status: AgentStatus) -> StatusAppearance {
        match status {
            AgentStatus::Blocked => StatusAppearance {
                icon: StatusIcon::Symbol("dialog-warning".into()),
                color: (0.9, 0.2, 0.2),
            },
            AgentStatus::Working => StatusAppearance {
                icon: StatusIcon::Symbol("view-refresh".into()),
                color: (0.2, 0.4, 0.9),
            },
            AgentStatus::Done => StatusAppearance {
                icon: StatusIcon::Symbol("emblem-ok".into()),
                color: (0.2, 0.7, 0.3),
            },
            AgentStatus::Idle => StatusAppearance {
                icon: StatusIcon::Symbol("bullet".into()),
                color: (0.5, 0.5, 0.5),
            },
            AgentStatus::Unknown => StatusAppearance {
                icon: StatusIcon::Symbol("help-questionmark".into()),
                color: (0.5, 0.5, 0.5),
            },
        }
    }

    fn aggregate_icon(&self, statuses: &[AgentStatus]) -> StatusIcon {
        match aggregate_status(statuses) {
            AgentStatus::Blocked => StatusIcon::Symbol("dialog-warning".into()),
            AgentStatus::Unknown => StatusIcon::Symbol("help-questionmark".into()),
            AgentStatus::Done => StatusIcon::Symbol("emblem-ok".into()),
            AgentStatus::Working => StatusIcon::Symbol("view-refresh".into()),
            AgentStatus::Idle => self.idle_aggregate_icon(),
        }
    }

    fn idle_aggregate_icon(&self) -> StatusIcon {
        StatusIcon::Symbol("view-grid-symbolic".into())
    }

    fn disconnected_icon(&self) -> StatusIcon {
        StatusIcon::Symbol("window-close".into())
    }

    fn focused_marker_icon(&self) -> StatusIcon {
        StatusIcon::Symbol("go-next".into())
    }

    fn empty_state_icon(&self) -> StatusIcon {
        self.disconnected_icon()
    }
}

pub struct CustomIconScheme;

impl IconScheme for CustomIconScheme {
    fn id(&self) -> &str {
        "custom"
    }

    fn display_name_key(&self) -> &str {
        "icon.scheme.custom"
    }

    fn appearance(&self, status: AgentStatus) -> StatusAppearance {
        match status {
            AgentStatus::Blocked => StatusAppearance {
                icon: StatusIcon::Asset {
                    name: "status-blocked".into(),
                    fallback: "dialog-warning".into(),
                },
                color: (0.9, 0.2, 0.2),
            },
            AgentStatus::Working => StatusAppearance {
                icon: StatusIcon::Asset {
                    name: "status-working".into(),
                    fallback: "view-refresh".into(),
                },
                color: (0.2, 0.4, 0.9),
            },
            AgentStatus::Done => StatusAppearance {
                icon: StatusIcon::Asset {
                    name: "status-done".into(),
                    fallback: "emblem-ok".into(),
                },
                color: (0.2, 0.7, 0.3),
            },
            AgentStatus::Idle => StatusAppearance {
                icon: StatusIcon::Asset {
                    name: "status-idle".into(),
                    fallback: "bullet".into(),
                },
                color: (0.5, 0.5, 0.5),
            },
            AgentStatus::Unknown => StatusAppearance {
                icon: StatusIcon::Asset {
                    name: "status-unknown".into(),
                    fallback: "help-questionmark".into(),
                },
                color: (0.5, 0.5, 0.5),
            },
        }
    }

    fn aggregate_icon(&self, statuses: &[AgentStatus]) -> StatusIcon {
        match aggregate_status(statuses) {
            AgentStatus::Idle => self.idle_aggregate_icon(),
            status => self.appearance(status).icon,
        }
    }

    fn idle_aggregate_icon(&self) -> StatusIcon {
        StatusIcon::Asset {
            name: "aggregate-idle".into(),
            fallback: "view-grid-symbolic".into(),
        }
    }

    fn disconnected_icon(&self) -> StatusIcon {
        StatusIcon::Asset {
            name: "aggregate-disconnected".into(),
            fallback: "window-close".into(),
        }
    }

    fn focused_marker_icon(&self) -> StatusIcon {
        StatusIcon::Asset {
            name: "marker-focused".into(),
            fallback: "go-next".into(),
        }
    }

    fn empty_state_icon(&self) -> StatusIcon {
        self.disconnected_icon()
    }
}

pub struct IconSchemeRegistry;

impl IconSchemeRegistry {
    pub fn all() -> Vec<Box<dyn IconScheme>> {
        vec![Box::new(CustomIconScheme), Box::new(ClassicIconScheme)]
    }

    pub fn default_scheme() -> Box<dyn IconScheme> {
        Box::new(CustomIconScheme)
    }

    pub fn scheme(id: &str) -> Box<dyn IconScheme> {
        Self::all().into_iter().find(|s| s.id() == id).unwrap_or_else(Self::default_scheme)
    }

    pub fn scheme_ids() -> Vec<(String, String)> {
        Self::all()
            .iter()
            .map(|s| (s.id().to_string(), s.display_name_key().to_string()))
            .collect()
    }
}

pub struct NotificationPolicy;

impl NotificationPolicy {
    pub fn should_notify(from: Option<AgentStatus>, to: AgentStatus) -> bool {
        if to != AgentStatus::Blocked && to != AgentStatus::Done {
            return false;
        }
        from != Some(to)
    }
}
