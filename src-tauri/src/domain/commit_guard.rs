use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::settings::GuardMode;

/// The install state of the managed `pre-commit` hook in one repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum HookState {
    /// No `pre-commit` hook exists.
    NotInstalled,
    /// The managed hook is installed and no foreign hook was displaced.
    Managed,
    /// The managed hook is installed and chains a pre-existing hook that was
    /// backed up during installation.
    ManagedChained,
    /// A foreign (non-managed) `pre-commit` hook is present and untouched.
    /// Installation would chain, not overwrite.
    Foreign,
    /// The repository routes hooks through `core.hooksPath`, so a hook in
    /// `.git/hooks` would be ignored. Requires manual setup.
    Unsupported,
}

/// Per-repository Commit Guard status, surfaced in the UI so the user can see
/// exactly which repositories are protected and how.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RepoGuardStatus {
    pub(crate) repo_id: Uuid,
    pub(crate) git_root: String,
    pub(crate) repo_name: String,
    pub(crate) hook_state: HookState,
    /// `true` when the managed hook is installed (`Managed` or `ManagedChained`).
    pub(crate) protected: bool,
    /// The mode recorded in the repo's local config, when protected.
    pub(crate) mode: Option<GuardMode>,
    /// The profile this repository is expected to commit as, if one can be
    /// resolved (rule match or repository assignment). `None` means no expected
    /// profile — the guard allows commits (never blocks on the unknown).
    pub(crate) expected_profile_id: Option<Uuid>,
    pub(crate) expected_label: Option<String>,
    /// The `user.email` a commit here would currently use (local config, else
    /// global). Lets the UI show a live match/mismatch. Never a secret.
    pub(crate) current_email: Option<String>,
    /// `Some(true/false)` when an expected profile is known; `None` when no
    /// expected identity can be resolved (guard would allow the commit).
    pub(crate) identity_matches: Option<bool>,
}

/// Aggregate Commit Guard status for the dashboard/settings view.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct CommitGuardStatus {
    pub(crate) enabled: bool,
    pub(crate) mode: GuardMode,
    pub(crate) auto_protect: bool,
    /// Number of tracked repositories with the managed hook installed.
    pub(crate) protected_count: u32,
    pub(crate) repos: Vec<RepoGuardStatus>,
}

/// The identity written into a protected repository's local config for the hook
/// to compare against. Holds only non-secret Git identity fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedIdentity {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) mode: GuardMode,
}
