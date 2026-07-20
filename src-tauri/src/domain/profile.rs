use crate::domain::identity::Identity;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Profile {
    pub(crate) id: Uuid,
    pub(crate) label: String,
    pub(crate) identity: Identity,
    pub(crate) color: Option<String>,
}
