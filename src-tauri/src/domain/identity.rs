use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Identity {
    pub(crate) name: String,
    pub(crate) email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) signing_key: Option<String>,
}
