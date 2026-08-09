use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Repo {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) git_root: String,
    pub(crate) active_branch: Option<String>,
    pub(crate) active_profile_id: Option<Uuid>,
    pub(crate) remote_origin: Option<String>,
    pub(crate) last_opened: Option<DateTime<Utc>>,
    pub(crate) favorite: bool,
    pub(crate) detected_at: DateTime<Utc>,
    /// The user-defined group ("ecosystem") this repo belongs to, if any.
    /// `#[serde(default)]` keeps pre-grouping records deserializable.
    #[serde(default)]
    pub(crate) group_id: Option<Uuid>,
    /// Whether `path` still exists on disk. Derived at read time and recomputed
    /// on every load, so the UI can flag repositories whose folder was moved or
    /// deleted. `#[serde(default)]` keeps pre-existing records deserializable.
    #[serde(default)]
    pub(crate) path_exists: bool,
}

/// A user-defined repository group — an "ecosystem" such as "Dolphin Modules"
/// that repositories can be organized under. Ordering in the UI follows the
/// stored order.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RepoGroup {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) color: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

/// Read-only Git metadata for a repository, resolved from the filesystem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RepoMetadata {
    pub(crate) active_branch: Option<String>,
    pub(crate) remote_origin: Option<String>,
}

/// Progress emitted to the frontend while a scan traverses the filesystem.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct ScanProgress {
    pub(crate) scanned_dirs: u32,
    pub(crate) found_repos: u32,
}
