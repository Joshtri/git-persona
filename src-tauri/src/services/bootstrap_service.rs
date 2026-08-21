use crate::{
    domain::bootstrap::{AnnouncementAction, AnnouncementInfo, BootstrapData},
    error::AppError,
};
use std::collections::HashMap;

const DEFAULT_SERVER_URL: &str = "https://api-gitpersona.vercel.app";

// --- Wire types: camelCase deserialization from the server's JSON response ---

#[derive(serde::Deserialize)]
struct WireEnvelope {
    data: WireBootstrapData,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireBootstrapData {
    update: Option<WireUpdateData>,
    announcements: Vec<WireAnnouncement>,
    feature_flags: HashMap<String, bool>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireUpdateData {
    force_update: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireAnnouncement {
    id: String,
    title: String,
    message: String,
    #[serde(rename = "type")]
    announcement_type: String,
    priority: String,
    dismissible: bool,
    start_at: String,
    end_at: String,
    action: Option<WireAction>,
}

#[derive(serde::Deserialize)]
struct WireAction {
    label: String,
    url: String,
}

// --- Mapping from wire types to domain types ---

fn from_wire(wire: WireBootstrapData) -> BootstrapData {
    BootstrapData {
        announcements: wire
            .announcements
            .into_iter()
            .map(|a| AnnouncementInfo {
                id: a.id,
                title: a.title,
                message: a.message,
                announcement_type: a.announcement_type,
                priority: a.priority,
                dismissible: a.dismissible,
                start_at: a.start_at,
                end_at: a.end_at,
                action: a.action.map(|act| AnnouncementAction {
                    label: act.label,
                    url: act.url,
                }),
            })
            .collect(),
        feature_flags: wire.feature_flags,
        force_update: wire.update.map(|u| u.force_update),
    }
}

// --- Service ---

/// HTTP client for the `GitPersona` server. Holds a `reqwest` client so the
/// underlying connection pool can be reused across calls.
pub(crate) struct BootstrapService {
    http: reqwest::Client,
}

impl BootstrapService {
    pub(crate) fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }

    /// Fetch the client initialization data from the `GitPersona` server.
    ///
    /// Returns `Err` when the network is unavailable or the server responds
    /// with a non-2xx status. The command layer treats these errors as
    /// non-fatal and the application starts normally regardless.
    pub(crate) async fn fetch(
        &self,
        version: &str,
        channel: &str,
    ) -> Result<BootstrapData, AppError> {
        let base = std::env::var("GITPERSONA_SERVER_URL")
            .unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());
        let url = format!("{base}/client/bootstrap");

        let response = self
            .http
            .get(&url)
            .query(&[("version", version), ("channel", channel)])
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("bootstrap request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(AppError::Internal(format!(
                "bootstrap returned HTTP {status}"
            )));
        }

        let envelope: WireEnvelope = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("bootstrap parse failed: {e}")))?;

        Ok(from_wire(envelope.data))
    }
}
