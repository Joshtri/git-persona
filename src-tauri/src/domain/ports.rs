use chrono::{DateTime, Utc};
use crate::domain::{
    audit::AuditEntry,
    credential::{Credential, CredentialTxn, Secret, VaultSnapshot},
    identity::Identity,
    identity_switch::ResolvedRepo,
    profile::Profile,
    repo::{Repo, RepoGroup, RepoMetadata, ScanProgress},
    rule::{EvaluationContext, Rule, RuleMatch},
    settings::AppSettings,
    ssh::{GeneratedKey, SshAlgorithm, SshConfigEntry, SshKey, SshKeyMetadata},
};
use crate::error::AppError;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) trait GitConfigBackend: Send + Sync {
    fn get_global_name(&self) -> Result<Option<String>, AppError>;
    fn get_global_email(&self) -> Result<Option<String>, AppError>;
    fn get_global_signing_key(&self) -> Result<Option<String>, AppError>;
    fn set_global_name(&self, name: &str) -> Result<(), AppError>;
    fn set_global_email(&self, email: &str) -> Result<(), AppError>;
    fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError>;

    /// Write `name`/`email`/`signingkey` into the repository's **local** config
    /// (`<git-dir>/config`). Git reads local config with higher precedence than
    /// the global file and resolves it at commit time, so pinning it here makes
    /// the correct identity present *before* a commit — the piece that removes
    /// the one-commit lag of purely reactive switching.
    fn set_local_identity(&self, git_root: &Path, identity: &Identity) -> Result<(), AppError>;

    /// Enable (`true`) or remove (`false`) path-scoped credential lookups for
    /// `host` in the repository's local config
    /// (`credential.https://<host>.useHttpPath`). With it enabled, Git passes
    /// the full remote URL path to the credential helper, which is what lets a
    /// repository authenticate as its assigned profile over HTTPS even while a
    /// different profile is globally active.
    fn set_local_use_http_path(
        &self,
        git_root: &Path,
        host: &str,
        enabled: bool,
    ) -> Result<(), AppError>;
}

pub(crate) trait ProfileStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<Profile>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<Profile>, AppError>;
    fn save(&self, profile: &Profile) -> Result<(), AppError>;
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
    fn load_settings(&self) -> Result<AppSettings, AppError>;
    fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError>;
    fn get_active_id(&self) -> Result<Option<Uuid>, AppError>;
    fn set_active_id(&self, id: Option<Uuid>) -> Result<(), AppError>;
}

pub(crate) trait AuditSink: Send + Sync {
    fn append(&self, entry: &AuditEntry) -> Result<(), AppError>;
    fn read_all(&self) -> Result<Vec<AuditEntry>, AppError>;
    /// Delete every entry whose timestamp is strictly before `cutoff` and
    /// rewrite the log file. Returns the number of entries removed.
    fn purge_before(&self, cutoff: DateTime<Utc>) -> Result<u32, AppError>;
}

pub(crate) trait RepoStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<Repo>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<Repo>, AppError>;
    fn find_by_git_root(&self, git_root: &str) -> Result<Option<Repo>, AppError>;
    fn save(&self, repo: &Repo) -> Result<(), AppError>;
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
    /// User-defined repository groups ("ecosystems"), in stored order.
    fn load_groups(&self) -> Result<Vec<RepoGroup>, AppError>;
    fn save_group(&self, group: &RepoGroup) -> Result<(), AppError>;
    fn delete_group(&self, id: Uuid) -> Result<(), AppError>;
}

/// Persistence for [`Rule`] definitions. Mirrors the other Tauri-store adapters.
pub(crate) trait RuleStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<Rule>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<Rule>, AppError>;
    fn save(&self, rule: &Rule) -> Result<(), AppError>;
    /// Replace the entire rule set — used by reorder and import.
    fn save_all(&self, rules: &[Rule]) -> Result<(), AppError>;
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

/// The decision-layer port Smart Switching consults before applying an identity.
/// Implemented by the rule service; keeps rule evaluation isolated from the
/// switching orchestrator, which never inspects rule internals.
pub(crate) trait RuleResolver: Send + Sync {
    /// The highest-priority enabled rule matching `ctx`, if any. Never mutates
    /// Git config, credentials, or profiles — it only decides.
    fn resolve(&self, ctx: &EvaluationContext) -> Result<Option<RuleMatch>, AppError>;
}

pub(crate) trait RepoScanner: Send + Sync {
    /// Recursively discover Git repository roots beneath `roots`.
    ///
    /// Implementations must be read-only, must not follow symlinks, and must
    /// invoke `on_progress` periodically so the caller can surface UI progress.
    fn scan(
        &self,
        roots: &[String],
        on_progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<String>, AppError>;
}

pub(crate) trait GitMetadataReader: Send + Sync {
    /// Read branch + remote origin for the repository at `git_root`.
    fn read(&self, git_root: &Path) -> Result<RepoMetadata, AppError>;
}

pub(crate) trait SshStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<SshKey>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<SshKey>, AppError>;
    fn find_by_path(&self, private_key_path: &str) -> Result<Option<SshKey>, AppError>;
    fn save(&self, key: &SshKey) -> Result<(), AppError>;
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub(crate) trait SshScanner: Send + Sync {
    /// Discover candidate private-key file paths inside `ssh_dir`.
    ///
    /// Must be read-only and must ignore `known_hosts`, `authorized_keys`,
    /// `config`, and `*.pub`. Returned paths are not guaranteed parseable —
    /// the caller validates each via an [`SshKeyReader`].
    fn scan(&self, ssh_dir: &Path) -> Result<Vec<String>, AppError>;
}

pub(crate) trait SshKeyReader: Send + Sync {
    /// Parse metadata (algorithm, fingerprint, comment, public path) from a
    /// private key file. Never decrypts and never returns private key material.
    fn read(&self, private_key_path: &Path) -> Result<SshKeyMetadata, AppError>;
}

pub(crate) trait SshKeyGenerator: Send + Sync {
    /// Generate a new key pair into `output_dir` as `file_name` (private) and
    /// `file_name.pub` (public). The private key is written with `0600` on Unix.
    fn generate(
        &self,
        algorithm: SshAlgorithm,
        output_dir: &Path,
        file_name: &str,
        comment: Option<&str>,
    ) -> Result<GeneratedKey, AppError>;
}

pub(crate) trait SshConfigWriter: Send + Sync {
    /// Replace the `GitPersona Managed` block with `entries`, preserving every
    /// byte outside the markers. Creates the file if absent.
    fn write_managed_block(&self, entries: &[SshConfigEntry]) -> Result<(), AppError>;
    /// Read the raw config contents for preview (empty string if absent).
    fn read_raw(&self) -> Result<String, AppError>;
}

/// Persistence for [`Credential`] **metadata** — never secrets. Mirrors the
/// other Tauri-store adapters; the store file holds no token material.
pub(crate) trait CredentialStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<Credential>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<Credential>, AppError>;
    /// The credential a profile owns for a host, if any (enforces one-per-host).
    fn find_by_host(&self, profile_id: Uuid, host: &str) -> Result<Option<Credential>, AppError>;
    fn save(&self, credential: &Credential) -> Result<(), AppError>;
    /// Remove a credential's metadata and any stored reveal-PIN hash.
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
    /// Store the Argon2 hash of a credential's reveal PIN in the side map.
    fn save_pin(&self, id: Uuid, hash: &str) -> Result<(), AppError>;
    /// The stored Argon2 PIN hash for a credential, or `None` if no PIN is set.
    fn load_pin(&self, id: Uuid) -> Result<Option<String>, AppError>;
}

/// The OS secure credential storage (Windows Credential Manager today; macOS
/// Keychain / Linux Secret Service later). The **only** place secrets live.
///
/// Every operation is keyed by an opaque `target` string built by the service.
/// The canonical, Git-readable target is `git:<scheme>://<host>`; per-credential
/// backing secrets are parked under a `gitpersona:<id>:<scheme>://<host>` target.
/// The vault must touch **only** those targets — it never enumerates or modifies
/// unrelated OS credentials.
pub(crate) trait CredentialVault: Send + Sync {
    /// Upsert `target` with `username`/`secret`, verify by reading it back, and
    /// return the prior slot contents so a failed switch can be rolled back.
    fn store(
        &self,
        target: &str,
        username: &str,
        secret: &Secret,
    ) -> Result<VaultSnapshot, AppError>;

    /// The username currently stored at `target`, or `None`. Never the secret.
    /// Used by the frontend phase to surface the active credential per host.
    #[allow(dead_code)]
    fn username(&self, target: &str) -> Result<Option<String>, AppError>;

    /// Read back the secret stored at `target`, or `None` if absent. Unlike every
    /// other read path this returns the secret to the caller — it exists solely
    /// for the PIN-gated reveal flow and must never be called without a prior
    /// successful PIN check.
    fn reveal(&self, target: &str) -> Result<Option<Secret>, AppError>;

    /// Copy the secret at `from` into `to` under `username`, keeping the secret
    /// inside the vault. Used to promote a profile's backing credential onto the
    /// canonical Git target. Returns `to`'s prior contents for rollback.
    fn promote(&self, from: &str, to: &str, username: &str) -> Result<VaultSnapshot, AppError>;

    /// Delete `target`. `Ok` even if it was already absent. Returns the prior
    /// slot contents for rollback.
    fn delete(&self, target: &str) -> Result<VaultSnapshot, AppError>;

    /// Restore a previously captured slot: rewrite it if the snapshot held a
    /// secret, otherwise delete it.
    fn restore(&self, snapshot: &VaultSnapshot) -> Result<(), AppError>;
}

/// Narrow port the profile-apply pipeline uses to switch a profile's HTTPS
/// credentials as one stage, with rollback. Implemented by the credential
/// service; keeps all vault access inside that service.
pub(crate) trait ProfileCredentialSync: Send + Sync {
    /// Promote every credential owned by `profile_id` onto its canonical Git
    /// target. Returns a transaction handle carrying rollback snapshots. A
    /// profile with no credentials yields an empty, successful transaction.
    fn switch_profile(&self, profile_id: Uuid) -> Result<CredentialTxn, AppError>;

    /// Undo a previously returned switch by restoring its snapshots.
    fn rollback(&self, txn: CredentialTxn) -> Result<(), AppError>;

    /// Promote the profile's credential for `host` onto the **path-scoped** Git
    /// target (`git:<scheme>://<host>/<path>`), so a single repository can
    /// authenticate as that profile regardless of which profile is globally
    /// active. Returns whether a credential was actually pinned (`false` when
    /// the profile owns no credential for `host`).
    fn pin_path(&self, profile_id: Uuid, host: &str, path: &str) -> Result<bool, AppError>;

    /// Remove the path-scoped Git credential for `host`/`path`, returning the
    /// repository to the globally active credential.
    fn unpin_path(&self, host: &str, path: &str) -> Result<(), AppError>;
}

// ---- Smart Identity Switching (Sprint 6) --------------------------------

/// Watches tracked repositories for Git activity (checkout / commit / merge /
/// reset / rebase / pull) and reports the affected repository so the orchestrator
/// can react. Kept behind a port so future activation sources (editor or terminal
/// integrations) can be introduced without touching the orchestration service.
///
/// Implementations run on a background thread, must be low-CPU, must debounce and
/// collapse duplicate events, and must ignore working-tree noise (`node_modules`,
/// build output, temporary files, and Git object writes).
pub(crate) trait WorkspaceWatcher: Send + Sync {
    /// Begin (or replace) watching the given repository roots. `on_activity` is
    /// invoked with the `git_root` of a repository whose Git state changed.
    fn start(
        &self,
        git_roots: &[String],
        on_activity: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<(), AppError>;
    /// Stop watching and release all OS watches. Idempotent.
    fn stop(&self);
    /// Suspend reacting to events without releasing the watches.
    fn pause(&self);
    /// Resume reacting after a [`pause`](Self::pause).
    fn resume(&self);
    fn is_watching(&self) -> bool;
    fn is_paused(&self) -> bool;
    fn watched_count(&self) -> usize;
}

/// Resolves which profile a repository is assigned and enumerates the
/// repositories eligible for watching. Backed by the repository store.
pub(crate) trait IdentityResolver: Send + Sync {
    /// The tracked repository at `git_root`, with its assignment, if any.
    fn resolve(&self, git_root: &str) -> Result<Option<ResolvedRepo>, AppError>;
    /// Every tracked repository's `git_root`, for the watcher to monitor.
    fn watchable_roots(&self) -> Result<Vec<String>, AppError>;
}

/// Applies a resolved identity by delegating to the existing profile-apply
/// pipeline (git config → HTTPS credentials → active marker, atomic with
/// rollback), and reports the currently active profile. The orchestrator never
/// edits Git or the vault directly — it only drives this port.
pub(crate) trait IdentitySwitcher: Send + Sync {
    fn active_profile_id(&self) -> Result<Option<Uuid>, AppError>;
    fn profile_label(&self, id: Uuid) -> Result<Option<String>, AppError>;
    fn apply(&self, profile_id: Uuid) -> Result<(), AppError>;
    /// Pin `profile_id`'s identity into the repository at `git_root`'s local Git
    /// config, so a commit there reads the right author before it is written.
    fn apply_local(&self, git_root: &str, profile_id: Uuid) -> Result<(), AppError>;
}

/// Sink for the orchestrator's observable side effects: switch notifications and
/// status changes the UI listens for.
pub(crate) trait SwitchObserver: Send + Sync {
    fn switched(&self, event: &crate::domain::identity_switch::SwitchEvent);
    fn status_changed(&self, status: &crate::domain::identity_switch::SmartSwitchStatus);
}
