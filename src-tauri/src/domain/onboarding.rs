use crate::domain::ssh::SshAlgorithm;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The Git identity read from the user's global config, offered as a profile to
/// create during first-run onboarding. Any field may be absent.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct DetectedIdentity {
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) signing_key: Option<String>,
}

/// A private key discovered in `~/.ssh` during onboarding. Holds references and
/// metadata only — never key material. Not yet persisted; the user selects which
/// to import.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct DetectedSshKey {
    pub(crate) private_key_path: String,
    pub(crate) algorithm: SshAlgorithm,
    pub(crate) fingerprint: String,
    pub(crate) comment: Option<String>,
}

/// A Git credential the OS vault already holds for a supported host. Only the
/// username is read — the secret is never touched during detection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct DetectedCredential {
    pub(crate) host: String,
    pub(crate) username: String,
}

/// The read-only result of scanning the system for existing Git setup, shown in
/// the first-run onboarding wizard.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct SystemScan {
    pub(crate) identity: Option<DetectedIdentity>,
    pub(crate) ssh_keys: Vec<DetectedSshKey>,
    pub(crate) credentials: Vec<DetectedCredential>,
    /// `true` when onboarding was already completed or a profile already exists —
    /// the frontend uses this to avoid re-prompting.
    pub(crate) already_onboarded: bool,
}
