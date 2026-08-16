use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Client state fetched from the `GitPersona` server once at startup.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct BootstrapData {
    pub(crate) announcements: Vec<AnnouncementInfo>,
    /// Enabled feature flags keyed by their kebab-case identifier.
    pub(crate) feature_flags: HashMap<String, bool>,
    /// Whether the server considers the current update mandatory.
    /// `None` when the server is unreachable or returns no update info.
    pub(crate) force_update: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct AnnouncementInfo {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) message: String,
    #[serde(rename = "type")]
    pub(crate) announcement_type: String,
    pub(crate) priority: String,
    pub(crate) dismissible: bool,
    pub(crate) start_at: String,
    pub(crate) end_at: String,
    pub(crate) action: Option<AnnouncementAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct AnnouncementAction {
    pub(crate) label: String,
    pub(crate) url: String,
}
