use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;
use zeroize::Zeroizing;

/// A secret (token / PAT / password) in transit through the backend.
///
/// `Zeroizing` overwrites the heap buffer on drop. Secrets are only ever held in
/// values of this type — never in [`Credential`], never serialized, never logged.
pub(crate) type Secret = Zeroizing<String>;

/// Transport protocol for a Git credential. Sprint 5 handles HTTPS only; the
/// enum exists so SSH/other schemes can be added without a data migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum Protocol {
    Https,
}

impl Protocol {
    /// The URL scheme used when building a Git credential target
    /// (`git:<scheme>://<host>`).
    pub(crate) fn scheme(self) -> &'static str {
        match self {
            Self::Https => "https",
        }
    }
}

/// Hosts `GitPersona` knows how to manage credentials for. The list is a starting
/// allow-list, not a hard limit — [`is_supported_host`] is the single check so a
/// future setting can widen it without touching call sites.
pub(crate) const SUPPORTED_HOSTS: &[&str] = &[
    "github.com",
    "gitlab.com",
    "bitbucket.org",
    "dev.azure.com",
];

pub(crate) fn is_supported_host(host: &str) -> bool {
    SUPPORTED_HOSTS.contains(&host)
}

/// Metadata for a stored HTTPS Git credential.
///
/// Holds **references and metadata only**. The secret lives exclusively in the
/// OS credential vault (Windows Credential Manager) and is never a field here.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Credential {
    pub(crate) id: Uuid,
    pub(crate) profile_id: Option<Uuid>,
    pub(crate) host: String,
    pub(crate) protocol: Protocol,
    pub(crate) username: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) last_used: Option<DateTime<Utc>>,
}

/// The full prior contents of a vault slot (identified by its target string),
/// captured so a failed multi-stage switch can be rolled back. The secret is
/// zeroized when the snapshot drops.
pub(crate) struct VaultSnapshot {
    pub(crate) target: String,
    pub(crate) prior: Option<VaultSecret>,
}

/// A username + secret pair read back from the vault for rollback purposes.
pub(crate) struct VaultSecret {
    pub(crate) username: String,
    pub(crate) secret: Secret,
}

/// Opaque handle returned by a credential switch, holding the rollback
/// snapshots. The profile-apply pipeline carries it and hands it back to the
/// credential service to undo the switch if a later stage fails.
pub(crate) struct CredentialTxn {
    pub(crate) snapshots: Vec<VaultSnapshot>,
}
