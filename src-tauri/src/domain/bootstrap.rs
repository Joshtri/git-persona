use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Client state fetched from the `GitPersona` server once at startup.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct BootstrapData {
    pub(crate) update: UpdateInfo,
    pub(crate) announcements: Vec<AnnouncementInfo>,
    /// Enabled feature flags keyed by their kebab-case identifier.
    pub(crate) feature_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct UpdateInfo {
    pub(crate) update_available: bool,
    pub(crate) force_update: bool,
    pub(crate) latest: Option<ReleaseInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ReleaseInfo {
    pub(crate) version: String,
    pub(crate) channel: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) release_notes: Vec<ReleaseNote>,
    pub(crate) minimum_version: Option<String>,
    pub(crate) download_url: String,
    pub(crate) published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ReleaseNote {
    // `type` is a Rust keyword; renamed for both JSON and TypeScript output.
    #[serde(rename = "type")]
    pub(crate) note_type: String,
    pub(crate) title: String,
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
