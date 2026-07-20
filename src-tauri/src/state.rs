use crate::{
    infra::{
        audit_jsonl::JsonlAuditSink, git_config_gix::GixGitConfig, git_meta::FsGitMetadataReader,
        paths, profile_store_tauri::TauriProfileStore, repo_scanner_fs::FsRepoScanner,
        repo_store_tauri::TauriRepoStore, ssh_config_writer::FsSshConfigWriter,
        ssh_key_generator::SshKeyPairGenerator, ssh_key_reader::SshKeyFileReader,
        ssh_scanner_fs::FsSshScanner, ssh_store_tauri::TauriSshStore,
    },
    services::{
        activity_service::ActivityService, profile_service::ProfileService,
        repo_service::RepoService, settings_service::SettingsService, ssh_service::SshService,
    },
};
use std::sync::Arc;

pub(crate) struct AppState {
    pub(crate) settings: SettingsService,
    pub(crate) profiles: ProfileService,
    pub(crate) activity: ActivityService,
    pub(crate) repos: RepoService,
    pub(crate) ssh: SshService,
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

        Ok(Self {
            settings: SettingsService::new(store.clone()),
            profiles: ProfileService::new(
                store.clone(),
                repo_store.clone(),
                git_config,
                audit.clone(),
            ),
            activity: ActivityService::new(audit.clone()),
            repos: RepoService::new(repo_store, scanner, git_meta, audit.clone()),
            ssh: SshService::new(
                ssh_store,
                ssh_scanner,
                ssh_reader,
                ssh_generator,
                ssh_config,
                store,
                audit,
            ),
        })
    }
}
