use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Supported SSH key algorithms. Anything else is ignored during discovery and
/// rejected on import (Sprint 4 covers ED25519 and RSA only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum SshAlgorithm {
    Ed25519,
    Rsa,
}

/// A managed SSH identity. Stores **references and metadata only** — private key
/// bytes are never held here, serialized, or logged.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct SshKey {
    pub(crate) id: Uuid,
    pub(crate) label: String,
    pub(crate) algorithm: SshAlgorithm,
    pub(crate) fingerprint: String,
    pub(crate) private_key_path: String,
    pub(crate) public_key_path: Option<String>,
    pub(crate) comment: Option<String>,
    pub(crate) assigned_profile_id: Option<Uuid>,
    pub(crate) host_alias: Option<String>,
    pub(crate) host_name: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) imported: bool,
    pub(crate) last_used: Option<DateTime<Utc>>,
}

/// Read-only metadata extracted from a private key file by an [`SshKeyReader`].
///
/// [`SshKeyReader`]: crate::domain::ports::SshKeyReader
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SshKeyMetadata {
    pub(crate) algorithm: SshAlgorithm,
    pub(crate) fingerprint: String,
    pub(crate) comment: Option<String>,
    pub(crate) public_key_path: Option<String>,
}

/// Result of generating a new key pair on disk.
#[derive(Debug, Clone)]
pub(crate) struct GeneratedKey {
    pub(crate) private_key_path: String,
    pub(crate) public_key_path: String,
    pub(crate) fingerprint: String,
    pub(crate) comment: Option<String>,
}

/// Inputs for generating a new key pair (assembled by the command layer).
#[derive(Debug, Clone)]
pub(crate) struct GenerateParams {
    pub(crate) label: String,
    pub(crate) algorithm: SshAlgorithm,
    pub(crate) comment: Option<String>,
    pub(crate) output_dir: String,
    pub(crate) file_name: String,
    pub(crate) host_alias: Option<String>,
    pub(crate) host_name: Option<String>,
}

/// A single `Host` stanza rendered into the managed `~/.ssh/config` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SshConfigEntry {
    pub(crate) host_alias: String,
    pub(crate) host_name: String,
    pub(crate) user: String,
    pub(crate) identity_file: String,
}
