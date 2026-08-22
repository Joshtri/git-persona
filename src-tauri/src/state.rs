use crate::{
    domain::ports::CredentialVault,
    infra::{
        audit_jsonl::JsonlAuditSink, commit_hook_fs::FsCommitHookInstaller,
        credential_store_tauri::TauriCredentialStore,
        git_config_gix::GixGitConfig, git_dir_watcher::GitDirWatcher,
        git_meta::FsGitMetadataReader, paths, profile_store_tauri::TauriProfileStore,
        repo_scanner_fs::FsRepoScanner, repo_store_tauri::TauriRepoStore,
        rule_store_tauri::TauriRuleStore, ssh_config_writer::FsSshConfigWriter,
        ssh_key_generator::SshKeyPairGenerator, ssh_key_reader::SshKeyFileReader,
        ssh_scanner_fs::FsSshScanner, ssh_store_tauri::TauriSshStore,
        switch_observer_tauri::TauriSwitchObserver,
    },
    services::bootstrap_service::BootstrapService,
    services::{
        activity_service::ActivityService,
        commit_guard_service::CommitGuardService,
        credential_service::CredentialService,
        identity_switch_service::{
            IdentitySwitchService, ProfileIdentitySwitcher, RepoIdentityResolver,
        },
        onboarding_service::OnboardingService,
        profile_service::ProfileService,
        repo_service::RepoService,
        rule_service::RuleService,
        settings_service::SettingsService,
        ssh_service::SshService,
    },
};
use std::sync::Arc;

#[cfg(windows)]
use crate::infra::credential_vault_windows::WindowsCredentialVault;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::infra::credential_vault_keyring::KeyringCredentialVault;

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
use crate::infra::credential_vault_noop::NoopCredentialVault;

pub(crate) struct AppState {
    pub(crate) settings: SettingsService,
    pub(crate) profiles: Arc<ProfileService>,
    pub(crate) activity: ActivityService,
    pub(crate) repos: RepoService,
    pub(crate) ssh: Arc<SshService>,
    pub(crate) credentials: Arc<CredentialService>,
    pub(crate) rules: Arc<RuleService>,
    pub(crate) identity_switch: Arc<IdentitySwitchService>,
    pub(crate) commit_guard: CommitGuardService,
    pub(crate) onboarding: OnboardingService,
    pub(crate) bootstrap: BootstrapService,
}

impl AppState {
    pub(crate) fn init(app: &tauri::AppHandle) -> Result<Self, String> {
        let audit_path = paths::audit_log_path(app).map_err(|e| e.to_string())?;
        if let Some(parent) = audit_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let store = Arc::new(TauriProfileStore::new(app.clone()));
        let git_config: Arc<GixGitConfig> = Arc::new(GixGitConfig);
        let audit = Arc::new(JsonlAuditSink::new(audit_path));

        let repo_store = Arc::new(TauriRepoStore::new(app.clone()));
        let scanner = Arc::new(FsRepoScanner::default());
        let git_meta = Arc::new(FsGitMetadataReader);

        let ssh_store = Arc::new(TauriSshStore::new(app.clone()));
        let ssh_scanner = Arc::new(FsSshScanner);
        let ssh_reader = Arc::new(SshKeyFileReader);
        let ssh_generator = Arc::new(SshKeyPairGenerator);
        let ssh_config_path = paths::ssh_config_path().map_err(|e| e.to_string())?;
        let ssh_config = Arc::new(FsSshConfigWriter::new(ssh_config_path));

        let cred_store = Arc::new(TauriCredentialStore::new(app.clone()));
        #[cfg(windows)]
        let cred_vault: Arc<dyn CredentialVault> = Arc::new(WindowsCredentialVault);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let cred_vault: Arc<dyn CredentialVault> = Arc::new(KeyringCredentialVault);
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        let cred_vault: Arc<dyn CredentialVault> = Arc::new(NoopCredentialVault);
        let credentials = Arc::new(CredentialService::new(
            cred_store,
            cred_vault.clone(),
            store.clone(),
            audit.clone(),
        ));

        let profiles = Arc::new(ProfileService::new(
            store.clone(),
            repo_store.clone(),
            git_config.clone(),
            credentials.clone(),
            audit.clone(),
        ));

        // Rule Engine (Sprint 7) — the decision layer. It selects a profile for a
        // repository; the switching orchestrator below consults it before falling
        // back to the manual repository assignment.
        let rules = Arc::new(RuleService::new(
            Arc::new(TauriRuleStore::new(app.clone())),
            store.clone(),
            audit.clone(),
        ));

        // Smart Identity Switching orchestrator (Sprint 6). It only coordinates
        // existing services — the resolver reads the repo store, and the switcher
        // delegates to the profile-apply pipeline.
        let resolver = Arc::new(RepoIdentityResolver::new(repo_store.clone()));
        let switcher = Arc::new(ProfileIdentitySwitcher::new(
            profiles.clone(),
            store.clone(),
        ));
        let watcher = Arc::new(GitDirWatcher::new());
        let observer = Arc::new(TauriSwitchObserver::new(app.clone()));
        let identity_switch = Arc::new(IdentitySwitchService::new(
            resolver,
            switcher,
            watcher,
            observer,
            store.clone(),
            rules.clone(),
            audit.clone(),
        ));

        // Commit Guard (final safety layer). Reuses the rule resolver and repo
        // store for expected-profile resolution and the git-config backend for
        // reading the repository's current identity; installs hooks via the
        // filesystem adapter. Independent of Smart Switching.
        let commit_guard = CommitGuardService::new(
            repo_store.clone(),
            rules.clone(),
            store.clone(),
            git_config.clone(),
            Arc::new(FsCommitHookInstaller),
            audit.clone(),
        );

        let ssh = Arc::new(SshService::new(
            ssh_store,
            ssh_scanner.clone(),
            ssh_reader.clone(),
            ssh_generator,
            ssh_config,
            store.clone(),
            audit.clone(),
        ));

        // First-run onboarding composes read-only detection over the existing
        // ports and delegates imports to the profile/SSH services.
        let onboarding = OnboardingService::new(
            git_config,
            cred_vault,
            ssh_scanner,
            ssh_reader,
            profiles.clone(),
            ssh.clone(),
            store.clone(),
        );

        Ok(Self {
            settings: SettingsService::new(store),
            profiles,
            activity: ActivityService::new(audit.clone()),
            repos: RepoService::new(repo_store, scanner, git_meta, audit),
            ssh,
            credentials,
            rules,
            identity_switch,
            commit_guard,
            onboarding,
            bootstrap: BootstrapService::new(),
        })
    }
}
