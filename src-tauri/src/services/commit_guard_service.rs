use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    audit::AuditEntry,
    commit_guard::{CommitGuardStatus, ExpectedIdentity, HookState, RepoGuardStatus},
    identity::Identity,
    ports::{AuditSink, CommitHookInstaller, GitConfigBackend, ProfileStore, RepoStore, RuleResolver},
    repo::Repo,
    rule::EvaluationContext,
    settings::{CommitGuardSettings, GuardMode},
};
use crate::error::AppError;

/// Orchestrates Commit Guard: resolves the profile a repository is expected to
/// commit as (reusing the Rules Engine and repository assignment), installs and
/// repairs the managed `pre-commit` hook non-destructively, and reports status.
///
/// It coordinates existing ports only — it never edits Git config, the vault, or
/// hook files directly beyond the [`CommitHookInstaller`] port.
pub(crate) struct CommitGuardService {
    repos: Arc<dyn RepoStore>,
    rules: Arc<dyn RuleResolver>,
    store: Arc<dyn ProfileStore>,
    git_config: Arc<dyn GitConfigBackend>,
    hooks: Arc<dyn CommitHookInstaller>,
    audit: Arc<dyn AuditSink>,
}

impl CommitGuardService {
    pub(crate) fn new(
        repos: Arc<dyn RepoStore>,
        rules: Arc<dyn RuleResolver>,
        store: Arc<dyn ProfileStore>,
        git_config: Arc<dyn GitConfigBackend>,
        hooks: Arc<dyn CommitHookInstaller>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            repos,
            rules,
            store,
            git_config,
            hooks,
            audit,
        }
    }

    fn settings(&self) -> Result<CommitGuardSettings, AppError> {
        Ok(self.store.load_settings()?.commit_guard)
    }

    /// Resolve the profile a repository should commit as: a matching rule wins,
    /// else the manual assignment. Returns the profile id, its identity, and its
    /// label, or `None` when neither applies.
    fn resolve_expected(
        &self,
        repo: &Repo,
    ) -> Result<Option<(Uuid, Identity, String)>, AppError> {
        let ctx = EvaluationContext::from_parts(
            &repo.git_root,
            &repo.name,
            repo.remote_origin.as_deref(),
        );
        let profile_id = match self.rules.resolve(&ctx)? {
            Some(m) => Some(m.profile_id),
            None => repo.active_profile_id,
        };
        match profile_id {
            Some(id) => Ok(self
                .store
                .find(id)?
                .map(|p| (id, p.identity, p.label))),
            None => Ok(None),
        }
    }

    fn expected_identity(
        expected: Option<&(Uuid, Identity, String)>,
        mode: GuardMode,
    ) -> ExpectedIdentity {
        match expected {
            Some((_, identity, _)) => ExpectedIdentity {
                name: identity.name.clone(),
                email: identity.email.clone(),
                mode,
            },
            // No expected profile → empty marker so the hook allows the commit.
            None => ExpectedIdentity {
                name: String::new(),
                email: String::new(),
                mode,
            },
        }
    }

    fn is_installed(&self, git_root: &str) -> bool {
        matches!(
            self.hooks
                .hook_state(Path::new(git_root))
                .unwrap_or(HookState::NotInstalled),
            HookState::Managed | HookState::ManagedChained
        )
    }

    pub(crate) fn status(&self) -> Result<CommitGuardStatus, AppError> {
        let cg = self.settings()?;
        let repos = self.repos.load_all()?;
        let mut protected_count = 0u32;
        let mut out = Vec::with_capacity(repos.len());

        for repo in repos {
            let root = Path::new(&repo.git_root);
            let hook_state = self
                .hooks
                .hook_state(root)
                .unwrap_or(HookState::NotInstalled);
            let protected =
                matches!(hook_state, HookState::Managed | HookState::ManagedChained);
            if protected {
                protected_count += 1;
            }
            let marker = self.hooks.read_marker(root).ok().flatten();
            let expected = self.resolve_expected(&repo).ok().flatten();
            let current_email = self.git_config.get_local_email(root).unwrap_or(None);
            let identity_matches = expected
                .as_ref()
                .map(|(_, id, _)| current_email.as_deref() == Some(id.email.as_str()));

            out.push(RepoGuardStatus {
                repo_id: repo.id,
                git_root: repo.git_root.clone(),
                repo_name: repo.name.clone(),
                hook_state,
                protected,
                mode: marker.map(|m| m.mode),
                expected_profile_id: expected.as_ref().map(|(id, _, _)| *id),
                expected_label: expected.as_ref().map(|(_, _, label)| label.clone()),
                current_email,
                identity_matches,
            });
        }

        Ok(CommitGuardStatus {
            enabled: cg.enabled,
            mode: cg.mode,
            auto_protect: cg.auto_protect,
            protected_count,
            repos: out,
        })
    }

    fn save<F: FnOnce(&mut CommitGuardSettings)>(&self, mutate: F) -> Result<(), AppError> {
        let mut settings = self.store.load_settings()?;
        mutate(&mut settings.commit_guard);
        self.store.save_settings(&settings)
    }

    /// Bring installed hooks in line with the current settings. When disabled,
    /// every managed hook is removed (restoring any chained foreign hook). When
    /// enabled, already-protected repositories are refreshed to the current mode,
    /// and (with `auto_protect`) repositories with an expected profile are
    /// protected. Best-effort per repository — one failure never aborts the rest.
    fn reconcile(&self) -> Result<(), AppError> {
        let cg = self.settings()?;
        if !cg.enabled {
            for repo in self.repos.load_all()? {
                if self.is_installed(&repo.git_root) {
                    let _ = self.hooks.uninstall(Path::new(&repo.git_root));
                }
            }
            return Ok(());
        }
        for repo in self.repos.load_all()? {
            let expected = self.resolve_expected(&repo)?;
            let want = self.is_installed(&repo.git_root) || (cg.auto_protect && expected.is_some());
            if want {
                let identity = Self::expected_identity(expected.as_ref(), cg.mode);
                let _ = self.hooks.install(Path::new(&repo.git_root), &identity);
            }
        }
        Ok(())
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<CommitGuardStatus, AppError> {
        self.save(|c| c.enabled = enabled)?;
        self.reconcile()?;
        self.record(
            if enabled {
                "commit_guard.enabled"
            } else {
                "commit_guard.disabled"
            },
            None,
            None,
        )?;
        self.status()
    }

    pub(crate) fn set_mode(&self, mode: GuardMode) -> Result<CommitGuardStatus, AppError> {
        self.save(|c| c.mode = mode)?;
        // Reconcile rewrites the marker (including mode) on every protected repo.
        self.reconcile()?;
        self.status()
    }

    pub(crate) fn set_auto_protect(&self, enabled: bool) -> Result<CommitGuardStatus, AppError> {
        self.save(|c| c.auto_protect = enabled)?;
        self.reconcile()?;
        self.status()
    }

    pub(crate) fn install(&self, repo_id: Uuid) -> Result<CommitGuardStatus, AppError> {
        let repo = self
            .repos
            .find(repo_id)?
            .ok_or_else(|| AppError::NotFound(format!("repository {repo_id}")))?;
        let mode = self.settings()?.mode;
        let expected = self.resolve_expected(&repo)?;
        let identity = Self::expected_identity(expected.as_ref(), mode);
        self.hooks.install(Path::new(&repo.git_root), &identity)?;
        self.record(
            "commit_guard.installed",
            expected.as_ref().map(|(id, _, _)| *id),
            Some(&repo.git_root),
        )?;
        self.status()
    }

    pub(crate) fn repair(&self, repo_id: Uuid) -> Result<CommitGuardStatus, AppError> {
        // Repair is an idempotent re-install: it rewrites the managed hook and
        // marker while preserving any previously backed-up foreign hook.
        self.install(repo_id)
    }

    pub(crate) fn uninstall(&self, repo_id: Uuid) -> Result<CommitGuardStatus, AppError> {
        let repo = self
            .repos
            .find(repo_id)?
            .ok_or_else(|| AppError::NotFound(format!("repository {repo_id}")))?;
        self.hooks.uninstall(Path::new(&repo.git_root))?;
        self.record("commit_guard.uninstalled", None, Some(&repo.git_root))?;
        self.status()
    }

    /// Refresh a single repository's expected-identity marker after its assigned
    /// profile changes, when the repo is protected (or auto-protect is on). Called
    /// from the assignment path. Best-effort: never fails the caller.
    pub(crate) fn refresh_after_assignment(&self, git_root: &str) {
        let Ok(cg) = self.settings() else { return };
        if !cg.enabled {
            return;
        }
        let Ok(Some(repo)) = self.repos.find_by_git_root(git_root) else {
            return;
        };
        let should = self.is_installed(git_root) || cg.auto_protect;
        if !should {
            return;
        }
        let Ok(expected) = self.resolve_expected(&repo) else {
            return;
        };
        // Only auto-install a fresh repo when there is an expected profile.
        if !self.is_installed(git_root) && expected.is_none() {
            return;
        }
        let identity = Self::expected_identity(expected.as_ref(), cg.mode);
        let _ = self.hooks.install(Path::new(git_root), &identity);
    }

    fn record(
        &self,
        action: &str,
        profile_id: Option<Uuid>,
        git_root: Option<&str>,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path: git_root.map(str::to_string),
        })
    }
}
