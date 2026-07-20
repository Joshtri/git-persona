use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// The result of evaluating one repository activation against the smart-switching
/// rules. Drives the toast/notification and dashboard state on the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum SwitchStatus {
    /// A different profile was applied automatically.
    Switched,
    /// The repository's assigned profile was already the active one — no-op.
    AlreadyActive,
    /// The repository is tracked but has no assigned profile.
    NoAssignment,
    /// A switch is warranted but is awaiting user confirmation.
    PendingConfirmation,
}

/// A single smart-switching event, emitted to the frontend and stored as the
/// "last automatic switch" for the dashboard. Carries only display metadata —
/// never secrets, tokens, or key material.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct SwitchEvent {
    pub(crate) status: SwitchStatus,
    pub(crate) git_root: String,
    pub(crate) repo_id: Option<Uuid>,
    pub(crate) repo_name: Option<String>,
    pub(crate) profile_id: Option<Uuid>,
    pub(crate) profile_label: Option<String>,
    pub(crate) at: DateTime<Utc>,
}

/// A snapshot of the watcher/orchestrator state for the dashboard widget.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct SmartSwitchStatus {
    pub(crate) enabled: bool,
    pub(crate) watching: bool,
    pub(crate) paused: bool,
    pub(crate) repos_monitored: u32,
    pub(crate) last_switch: Option<SwitchEvent>,
    pub(crate) started_at: Option<DateTime<Utc>>,
}

/// The outcome of resolving a repository's assigned identity. Domain-internal
/// (not serialized): the resolver port returns this to the orchestrator.
pub(crate) struct ResolvedRepo {
    pub(crate) repo_id: Uuid,
    pub(crate) repo_name: String,
    pub(crate) profile_id: Option<Uuid>,
}
