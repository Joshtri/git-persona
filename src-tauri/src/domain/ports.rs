use crate::domain::{
    audit::AuditEntry,
    profile::Profile,
    repo::{Repo, RepoMetadata, ScanProgress},
    settings::AppSettings,
    ssh::{GeneratedKey, SshAlgorithm, SshConfigEntry, SshKey, SshKeyMetadata},
};
use crate::error::AppError;
use std::path::Path;
use uuid::Uuid;

pub(crate) trait GitConfigBackend: Send + Sync {
    #[allow(dead_code)]
    fn get_global_name(&self) -> Result<Option<String>, AppError>;
    #[allow(dead_code)]
    fn get_global_email(&self) -> Result<Option<String>, AppError>;
    fn set_global_name(&self, name: &str) -> Result<(), AppError>;
    fn set_global_email(&self, email: &str) -> Result<(), AppError>;
    fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError>;
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
}

pub(crate) trait RepoStore: Send + Sync {
    fn load_all(&self) -> Result<Vec<Repo>, AppError>;
    fn find(&self, id: Uuid) -> Result<Option<Repo>, AppError>;
    fn find_by_git_root(&self, git_root: &str) -> Result<Option<Repo>, AppError>;
    fn save(&self, repo: &Repo) -> Result<(), AppError>;
    fn delete(&self, id: Uuid) -> Result<(), AppError>;
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
