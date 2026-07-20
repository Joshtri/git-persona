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
