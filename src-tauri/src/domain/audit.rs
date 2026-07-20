use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct AuditEntry {
    pub(crate) id: Uuid,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) action: String,
    pub(crate) profile_id: Option<Uuid>,
    pub(crate) repo_path: Option<String>,
}
