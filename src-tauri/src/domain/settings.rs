use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct AppSettings {
    pub(crate) theme: Theme,
    pub(crate) show_audit_log: bool,
    pub(crate) auto_scan_repos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum Theme {
    Dark,
    Light,
    System,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            show_audit_log: true,
            auto_scan_repos: false,
        }
    }
}
