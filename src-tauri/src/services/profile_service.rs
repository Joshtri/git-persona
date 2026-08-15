use crate::domain::{
    audit::AuditEntry,
    credential::{is_supported_host, parse_https_remote},
    identity::Identity,
    ports::{AuditSink, GitConfigBackend, ProfileCredentialSync, ProfileStore, RepoStore},
    profile::Profile,
};
use crate::error::AppError;
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// The git identity captured before an apply, used to roll back stage 1.
struct GitSnapshot {
    name: Option<String>,
    email: Option<String>,
    signing_key: Option<String>,
}

pub(crate) struct ProfileService {
    store: Arc<dyn ProfileStore>,
    repos: Arc<dyn RepoStore>,
    git_config: Arc<dyn GitConfigBackend>,
    credentials: Arc<dyn ProfileCredentialSync>,
    audit: Arc<dyn AuditSink>,
}

impl ProfileService {
    pub(crate) fn new(
        store: Arc<dyn ProfileStore>,
        repos: Arc<dyn RepoStore>,
        git_config: Arc<dyn GitConfigBackend>,
        credentials: Arc<dyn ProfileCredentialSync>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            repos,
            git_config,
            credentials,
            audit,
        }
    }

    pub(crate) fn list(&self) -> Result<Vec<Profile>, AppError> {
        self.store.load_all()
    }

    pub(crate) fn find(&self, id: Uuid) -> Result<Profile, AppError> {
        self.store
            .find(id)?
            .ok_or_else(|| AppError::NotFound(format!("profile {id}")))
    }

    pub(crate) fn create(
        &self,
        label: String,
        identity: Identity,
        color: Option<String>,
    ) -> Result<Profile, AppError> {
        let profiles = self.store.load_all()?;
        if profiles.iter().any(|p| p.identity.email == identity.email) {
            return Err(AppError::Conflict(format!(
                "a profile with email \"{}\" already exists",
                identity.email
            )));
        }
        let profile = Profile {
            id: Uuid::new_v4(),
            label,
            identity,
            color,
        };
        self.store.save(&profile)?;
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "profile.create".into(),
            profile_id: Some(profile.id),
            repo_path: None,
        })?;
        Ok(profile)
    }

    pub(crate) fn update(
        &self,
        id: Uuid,
        label: String,
        identity: Identity,
        color: Option<String>,
    ) -> Result<Profile, AppError> {
        let mut profile = self.find(id)?;
        let profiles = self.store.load_all()?;
        if profiles
            .iter()
            .any(|p| p.id != id && p.identity.email == identity.email)
        {
            return Err(AppError::Conflict(format!(
                "a profile with email \"{}\" already exists",
                identity.email
            )));
        }
        profile.label = label;
        profile.identity = identity;
        profile.color = color;
        self.store.save(&profile)?;
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "profile.update".into(),
            profile_id: Some(id),
            repo_path: None,
        })?;
        Ok(profile)
    }

    pub(crate) fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if let Some(active_id) = self.store.get_active_id()? {
            if active_id == id {
                return Err(AppError::Validation(
                    "cannot delete the active profile — apply another profile first".into(),
                ));
            }
        }
        let assigned = self
            .repos
            .load_all()?
            .into_iter()
            .filter(|r| r.active_profile_id == Some(id))
            .count();
        if assigned > 0 {
            let noun = if assigned == 1 {
                "repository"
            } else {
                "repositories"
            };
            return Err(AppError::Conflict(format!(
                "this profile is assigned to {assigned} {noun} — reassign or remove them first"
            )));
        }
        self.store.delete(id)?;
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "profile.delete".into(),
            profile_id: Some(id),
            repo_path: None,
        })?;
        Ok(())
    }

    /// Apply a profile as one multi-stage identity switch — git config, then
    /// HTTPS credentials, then active marker — rolling back completed stages if a
    /// later one fails so the machine never lands in a half-switched state.
    pub(crate) fn apply(&self, id: Uuid) -> Result<(), AppError> {
        let profile = self.find(id)?;

        // Snapshot everything a later failure must undo.
        let git_before = GitSnapshot {
            name: self.git_config.get_global_name()?,
            email: self.git_config.get_global_email()?,
            signing_key: self.git_config.get_global_signing_key()?,
        };
        let active_before = self.store.get_active_id()?;

        // Stage 1 — git config.
        if let Err(e) = self.apply_git_config(&profile.identity) {
            self.restore_git_config(&git_before);
            return Err(e);
        }

        // Stage 2 — HTTPS credentials (empty/no-op when the profile has none).
        let credential_txn = match self.credentials.switch_profile(id) {
            Ok(txn) => txn,
            Err(e) => {
                self.restore_git_config(&git_before);
                return Err(e);
            }
        };

        // Stage 3 — mark active.
        if let Err(e) = self.store.set_active_id(Some(id)) {
            let _ = self.credentials.rollback(credential_txn);
            self.restore_git_config(&git_before);
            let _ = self.store.set_active_id(active_before);
            return Err(e);
        }

        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "profile.apply".into(),
            profile_id: Some(id),
            repo_path: None,
        })?;
        Ok(())
    }

    /// Pin a profile's identity into a single repository's **local** Git config.
    ///
    /// Unlike [`apply`](Self::apply), this touches neither global config,
    /// credentials, nor the active marker — it only guarantees that a `git
    /// commit` in this repository reads the correct author *before* the commit
    /// is written. Applied proactively (on assignment / rule match / launch),
    /// this is what removes the one-commit lag of purely reactive switching.
    ///
    /// When the profile owns an HTTPS credential for the repository's remote,
    /// the credential is also pinned onto the repo's **path-scoped** Git target
    /// (and `credential.<scheme>://<host>.useHttpPath` enabled locally), so
    /// `pull`/`push` authenticate as this profile even while another profile is
    /// globally active.
    pub(crate) fn apply_local(&self, git_root: &Path, profile_id: Uuid) -> Result<(), AppError> {
        let profile = self.find(profile_id)?;
        self.git_config
            .set_local_identity(git_root, &profile.identity)?;
        // Credential pinning is a convenience for push/pull auth; a failure here
        // (e.g. the profile's backing secret is missing from the vault) must not
        // fail the identity pin or the caller's switch. Best-effort.
        let _ = self.pin_local_credential(git_root, profile_id);
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "identity.pin_local".into(),
            profile_id: Some(profile_id),
            repo_path: Some(git_root.to_string_lossy().into_owned()),
        })?;
        Ok(())
    }

    /// Enable path-scoped HTTPS auth for a repository and promote the assigned
    /// profile's credential onto the repo's path-scoped target. No-ops when the
    /// repository is untracked, the remote is not HTTPS, the host is unsupported,
    /// or the profile owns no credential for the host. `useHttpPath` is only
    /// enabled when a credential is actually pinned, so a profile without a
    /// credential leaves the repository's lookup behavior unchanged.
    fn pin_local_credential(&self, git_root: &Path, profile_id: Uuid) -> Result<(), AppError> {
        let Some(repo) = self
            .repos
            .find_by_git_root(git_root.to_string_lossy().as_ref())?
        else {
            return Ok(());
        };
        let Some(remote) = repo.remote_origin.as_deref() else {
            return Ok(());
        };
        let Some((host, path)) = parse_https_remote(remote) else {
            return Ok(());
        };
        if !is_supported_host(&host) {
            return Ok(());
        }
        if self.credentials.pin_path(profile_id, &host, &path)? {
            self.git_config
                .set_local_use_http_path(git_root, &host, true)?;
        }
        Ok(())
    }

    /// Undo a per-repository pin: disable path-scoped HTTPS auth locally and
    /// drop the repo's path-scoped credential so git falls back to the globally
    /// active credential. No-ops for untracked repositories or non-HTTPS remotes.
    pub(crate) fn unpin_local(&self, git_root: &Path) -> Result<(), AppError> {
        let Some(repo) = self
            .repos
            .find_by_git_root(git_root.to_string_lossy().as_ref())?
        else {
            return Ok(());
        };
        let Some(remote) = repo.remote_origin.as_deref() else {
            return Ok(());
        };
        let Some((host, path)) = parse_https_remote(remote) else {
            return Ok(());
        };
        self.git_config
            .set_local_use_http_path(git_root, &host, false)?;
        self.credentials.unpin_path(&host, &path)?;
        Ok(())
    }

    fn apply_git_config(&self, identity: &Identity) -> Result<(), AppError> {
        self.git_config.set_global_name(&identity.name)?;
        self.git_config.set_global_email(&identity.email)?;
        self.git_config
            .set_global_signing_key(identity.signing_key.as_deref())?;
        Ok(())
    }

    /// Best-effort restore of the pre-apply git identity. Values that were unset
    /// before cannot be cleared here, so only present values are rewritten.
    fn restore_git_config(&self, before: &GitSnapshot) {
        if let Some(name) = &before.name {
            let _ = self.git_config.set_global_name(name);
        }
        if let Some(email) = &before.email {
            let _ = self.git_config.set_global_email(email);
        }
        let _ = self
            .git_config
            .set_global_signing_key(before.signing_key.as_deref());
    }

    pub(crate) fn get_active(&self) -> Result<Option<Profile>, AppError> {
        match self.store.get_active_id()? {
            Some(id) => self.store.find(id),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;
    use crate::domain::{
        audit::AuditEntry, credential::CredentialTxn, ports::AuditSink, repo::Repo,
        settings::AppSettings,
    };
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockStore {
        profiles: Mutex<Vec<Profile>>,
        active_id: Mutex<Option<Uuid>>,
    }

    impl MockStore {
        fn new(profiles: Vec<Profile>) -> Self {
            Self {
                profiles: Mutex::new(profiles),
                active_id: Mutex::new(None),
            }
        }
    }

    impl ProfileStore for MockStore {
        fn load_all(&self) -> Result<Vec<Profile>, AppError> {
            Ok(self.profiles.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<Profile>, AppError> {
            Ok(self
                .profiles
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }
        fn save(&self, profile: &Profile) -> Result<(), AppError> {
            let mut ps = self.profiles.lock().unwrap();
            if let Some(pos) = ps.iter().position(|p| p.id == profile.id) {
                ps[pos] = profile.clone();
            } else {
                ps.push(profile.clone());
            }
            Ok(())
        }
        fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut ps = self.profiles.lock().unwrap();
            let before = ps.len();
            ps.retain(|p| p.id != id);
            if ps.len() == before {
                return Err(AppError::NotFound(format!("profile {id}")));
            }
            Ok(())
        }
        fn load_settings(&self) -> Result<AppSettings, AppError> {
            Ok(AppSettings::default())
        }
        fn save_settings(&self, _: &AppSettings) -> Result<(), AppError> {
            Ok(())
        }
        fn get_active_id(&self) -> Result<Option<Uuid>, AppError> {
            Ok(*self.active_id.lock().unwrap())
        }
        fn set_active_id(&self, id: Option<Uuid>) -> Result<(), AppError> {
            *self.active_id.lock().unwrap() = id;
            Ok(())
        }
    }

    struct MockGitConfig {
        name: Mutex<Option<String>>,
        email: Mutex<Option<String>>,
        signing_key: Mutex<Option<String>>,
        local: Mutex<Vec<(String, Identity)>>,
        use_http_path: Mutex<Vec<(String, String)>>,
    }

    impl MockGitConfig {
        fn new() -> Self {
            Self {
                name: Mutex::new(None),
                email: Mutex::new(None),
                signing_key: Mutex::new(None),
                local: Mutex::new(vec![]),
                use_http_path: Mutex::new(vec![]),
            }
        }
    }

    impl GitConfigBackend for MockGitConfig {
        fn get_global_name(&self) -> Result<Option<String>, AppError> {
            Ok(self.name.lock().unwrap().clone())
        }
        fn get_global_email(&self) -> Result<Option<String>, AppError> {
            Ok(self.email.lock().unwrap().clone())
        }
        fn set_global_name(&self, name: &str) -> Result<(), AppError> {
            *self.name.lock().unwrap() = Some(name.into());
            Ok(())
        }
        fn set_global_email(&self, email: &str) -> Result<(), AppError> {
            *self.email.lock().unwrap() = Some(email.into());
            Ok(())
        }
        fn get_global_signing_key(&self) -> Result<Option<String>, AppError> {
            Ok(self.signing_key.lock().unwrap().clone())
        }
        fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError> {
            *self.signing_key.lock().unwrap() = key.map(Into::into);
            Ok(())
        }
        fn set_local_identity(&self, git_root: &Path, identity: &Identity) -> Result<(), AppError> {
            self.local
                .lock()
                .unwrap()
                .push((git_root.to_string_lossy().into_owned(), identity.clone()));
            Ok(())
        }
        fn set_local_use_http_path(
            &self,
            git_root: &Path,
            host: &str,
            enabled: bool,
        ) -> Result<(), AppError> {
            self.use_http_path.lock().unwrap().push((
                git_root.to_string_lossy().into_owned(),
                format!("{host}:{enabled}"),
            ));
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockCredentialSync {
        switched: Mutex<Vec<Uuid>>,
        rolled_back: Mutex<u32>,
        pinned: Mutex<Vec<(Uuid, String, String)>>,
        unpinned: Mutex<Vec<(String, String)>>,
    }

    impl ProfileCredentialSync for MockCredentialSync {
        fn switch_profile(&self, profile_id: Uuid) -> Result<CredentialTxn, AppError> {
            self.switched.lock().unwrap().push(profile_id);
            Ok(CredentialTxn { snapshots: vec![] })
        }
        fn rollback(&self, _txn: CredentialTxn) -> Result<(), AppError> {
            *self.rolled_back.lock().unwrap() += 1;
            Ok(())
        }
        fn pin_path(&self, profile_id: Uuid, host: &str, path: &str) -> Result<bool, AppError> {
            self.pinned
                .lock()
                .unwrap()
                .push((profile_id, host.to_string(), path.to_string()));
            Ok(true)
        }
        fn unpin_path(&self, host: &str, path: &str) -> Result<(), AppError> {
            self.unpinned
                .lock()
                .unwrap()
                .push((host.to_string(), path.to_string()));
            Ok(())
        }
    }

    struct MockAudit {
        entries: Mutex<Vec<AuditEntry>>,
    }

    impl MockAudit {
        fn new() -> Self {
            Self {
                entries: Mutex::new(vec![]),
            }
        }
    }

    impl AuditSink for MockAudit {
        fn append(&self, entry: &AuditEntry) -> Result<(), AppError> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(())
        }
        fn read_all(&self) -> Result<Vec<AuditEntry>, AppError> {
            Ok(self.entries.lock().unwrap().clone())
        }
    }

    struct MockRepoStore {
        repos: Mutex<Vec<Repo>>,
    }

    impl RepoStore for MockRepoStore {
        fn load_all(&self) -> Result<Vec<Repo>, AppError> {
            Ok(self.repos.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<Repo>, AppError> {
            Ok(self
                .repos
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned())
        }
        fn find_by_git_root(&self, git_root: &str) -> Result<Option<Repo>, AppError> {
            Ok(self
                .repos
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.git_root == git_root)
                .cloned())
        }
        fn save(&self, _repo: &Repo) -> Result<(), AppError> {
            Ok(())
        }
        fn delete(&self, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        fn load_groups(&self) -> Result<Vec<crate::domain::repo::RepoGroup>, AppError> {
            Ok(vec![])
        }
        fn save_group(&self, _group: &crate::domain::repo::RepoGroup) -> Result<(), AppError> {
            Ok(())
        }
        fn delete_group(&self, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn make_service(
        profiles: Vec<Profile>,
    ) -> (ProfileService, Arc<MockGitConfig>, Arc<MockAudit>) {
        make_service_with_repos(profiles, vec![])
    }

    fn make_service_with_repos(
        profiles: Vec<Profile>,
        repos: Vec<Repo>,
    ) -> (ProfileService, Arc<MockGitConfig>, Arc<MockAudit>) {
        let git_config = Arc::new(MockGitConfig::new());
        let audit = Arc::new(MockAudit::new());
        let svc = ProfileService::new(
            Arc::new(MockStore::new(profiles)),
            Arc::new(MockRepoStore {
                repos: Mutex::new(repos),
            }),
            git_config.clone(),
            Arc::new(MockCredentialSync::default()),
            audit.clone(),
        );
        (svc, git_config, audit)
    }

    fn make_service_with_credentials(
        credentials: Arc<MockCredentialSync>,
    ) -> (ProfileService, Arc<MockGitConfig>) {
        let git_config = Arc::new(MockGitConfig::new());
        let svc = ProfileService::new(
            Arc::new(MockStore::new(vec![])),
            Arc::new(MockRepoStore {
                repos: Mutex::new(vec![]),
            }),
            git_config.clone(),
            credentials,
            Arc::new(MockAudit::new()),
        );
        (svc, git_config)
    }

    #[allow(clippy::type_complexity)]
    fn make_service_full(
        profiles: Vec<Profile>,
        repos: Vec<Repo>,
        credentials: Arc<MockCredentialSync>,
    ) -> (
        ProfileService,
        Arc<MockGitConfig>,
        Arc<MockAudit>,
        Arc<MockCredentialSync>,
    ) {
        let git_config = Arc::new(MockGitConfig::new());
        let audit = Arc::new(MockAudit::new());
        let svc = ProfileService::new(
            Arc::new(MockStore::new(profiles)),
            Arc::new(MockRepoStore {
                repos: Mutex::new(repos),
            }),
            git_config.clone(),
            credentials.clone(),
            audit.clone(),
        );
        (svc, git_config, audit, credentials)
    }

    fn repo_assigned_to(profile_id: Uuid) -> Repo {
        Repo {
            id: Uuid::new_v4(),
            name: "demo".into(),
            path: "/tmp/demo".into(),
            git_root: "/tmp/demo".into(),
            active_branch: None,
            active_profile_id: Some(profile_id),
            remote_origin: None,
            last_opened: None,
            favorite: false,
            detected_at: Utc::now(),
            group_id: None,
            path_exists: true,
        }
    }

    fn identity(email: &str) -> Identity {
        Identity {
            name: "Test User".into(),
            email: email.into(),
            signing_key: None,
        }
    }

    #[test]
    fn create_assigns_new_uuid() {
        let (svc, _, _) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("a@example.com"), None)
            .unwrap();
        assert!(!p.id.is_nil());
        assert_eq!(p.label, "Work");
    }

    #[test]
    fn create_rejects_duplicate_email() {
        let (svc, _, _) = make_service(vec![]);
        svc.create("Work".into(), identity("a@example.com"), None)
            .unwrap();
        let err = svc
            .create("Work2".into(), identity("a@example.com"), None)
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn delete_active_is_rejected() {
        let (svc, _, _) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("a@example.com"), None)
            .unwrap();
        svc.apply(p.id).unwrap();
        let err = svc.delete(p.id).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn delete_rejected_when_assigned_to_repositories() {
        let profile_id = Uuid::new_v4();
        let profile = Profile {
            id: profile_id,
            label: "Work".into(),
            identity: identity("work@example.com"),
            color: None,
        };
        let (svc, _, _) = make_service_with_repos(
            vec![profile],
            vec![repo_assigned_to(profile_id), repo_assigned_to(profile_id)],
        );
        let err = svc.delete(profile_id).unwrap_err();
        match err {
            AppError::Conflict(msg) => assert!(msg.contains('2')),
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[test]
    fn delete_allowed_when_not_assigned() {
        let (svc, _, _) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("a@example.com"), None)
            .unwrap();
        assert!(svc.delete(p.id).is_ok());
    }

    #[test]
    fn apply_writes_git_config_and_sets_active_id() {
        let (svc, git, _) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("work@example.com"), None)
            .unwrap();
        svc.apply(p.id).unwrap();
        assert_eq!(*git.email.lock().unwrap(), Some("work@example.com".into()));
        assert_eq!(*git.name.lock().unwrap(), Some("Test User".into()));
        assert_eq!(svc.get_active().unwrap().unwrap().id, p.id);
    }

    #[test]
    fn apply_switches_credentials_for_profile() {
        let credentials = Arc::new(MockCredentialSync::default());
        let (svc, _git) = make_service_with_credentials(credentials.clone());
        let p = svc
            .create("Work".into(), identity("work@example.com"), None)
            .unwrap();
        svc.apply(p.id).unwrap();
        assert_eq!(credentials.switched.lock().unwrap().as_slice(), &[p.id]);
    }

    #[test]
    fn apply_local_pins_repo_config_without_touching_global() {
        let (svc, git, audit) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("work@example.com"), None)
            .unwrap();
        svc.apply_local(Path::new("/tmp/repo"), p.id).unwrap();

        let local = git.local.lock().unwrap();
        assert_eq!(local.len(), 1);
        assert_eq!(local[0].0, "/tmp/repo");
        assert_eq!(local[0].1.email, "work@example.com");
        // Global config and active marker untouched by a local pin.
        assert!(git.email.lock().unwrap().is_none());
        assert!(svc.get_active().unwrap().is_none());
        assert!(audit
            .read_all()
            .unwrap()
            .iter()
            .any(|e| e.action == "identity.pin_local"));
    }

    #[test]
    fn apply_local_pins_path_credential_for_https_remote() {
        let pid = Uuid::new_v4();
        let profile = Profile {
            id: pid,
            label: "Joshtri".into(),
            identity: identity("joshtri@users.noreply.github.com"),
            color: None,
        };
        let repo = Repo {
            id: Uuid::new_v4(),
            name: "laundry".into(),
            path: "/tmp/laundry".into(),
            git_root: "/tmp/laundry".into(),
            active_branch: None,
            active_profile_id: Some(pid),
            remote_origin: Some(
                "https://github.com/Joshtri/landing-page-dolphin-laundry.git/".into(),
            ),
            last_opened: None,
            favorite: false,
            detected_at: Utc::now(),
            group_id: None,
            path_exists: true,
        };
        let creds = Arc::new(MockCredentialSync::default());
        let (svc, git, _, creds) = make_service_full(vec![profile], vec![repo], creds.clone());

        svc.apply_local(Path::new("/tmp/laundry"), pid).unwrap();

        assert_eq!(
            creds.pinned.lock().unwrap().as_slice(),
            &[(
                pid,
                "github.com".to_string(),
                "Joshtri/landing-page-dolphin-laundry.git".to_string()
            )]
        );
        // useHttpPath is enabled for the host in the repo's local config.
        assert_eq!(
            git.use_http_path.lock().unwrap().as_slice(),
            &[("/tmp/laundry".to_string(), "github.com:true".to_string())]
        );
    }

    #[test]
    fn apply_local_skips_credential_pin_for_ssh_remote() {
        let pid = Uuid::new_v4();
        let profile = Profile {
            id: pid,
            label: "Work".into(),
            identity: identity("work@example.com"),
            color: None,
        };
        let repo = Repo {
            id: Uuid::new_v4(),
            name: "repo".into(),
            path: "/tmp/repo".into(),
            git_root: "/tmp/repo".into(),
            active_branch: None,
            active_profile_id: Some(pid),
            remote_origin: Some("git@github.com:acme/repo.git".into()),
            last_opened: None,
            favorite: false,
            detected_at: Utc::now(),
            group_id: None,
            path_exists: true,
        };
        let creds = Arc::new(MockCredentialSync::default());
        let (svc, git, _, creds) = make_service_full(vec![profile], vec![repo], creds.clone());

        svc.apply_local(Path::new("/tmp/repo"), pid).unwrap();

        assert!(creds.pinned.lock().unwrap().is_empty());
        assert!(git.use_http_path.lock().unwrap().is_empty());
    }

    #[test]
    fn unpin_local_disables_use_http_path_and_removes_credential() {
        let repo = Repo {
            id: Uuid::new_v4(),
            name: "laundry".into(),
            path: "/tmp/laundry".into(),
            git_root: "/tmp/laundry".into(),
            active_branch: None,
            active_profile_id: None,
            remote_origin: Some(
                "https://github.com/Joshtri/landing-page-dolphin-laundry.git".into(),
            ),
            last_opened: None,
            favorite: false,
            detected_at: Utc::now(),
            group_id: None,
            path_exists: true,
        };
        let creds = Arc::new(MockCredentialSync::default());
        let (svc, git, _, creds) = make_service_full(vec![], vec![repo], creds.clone());

        svc.unpin_local(Path::new("/tmp/laundry")).unwrap();

        assert_eq!(
            git.use_http_path.lock().unwrap().as_slice(),
            &[("/tmp/laundry".to_string(), "github.com:false".to_string())]
        );
        assert_eq!(
            creds.unpinned.lock().unwrap().as_slice(),
            &[(
                "github.com".to_string(),
                "Joshtri/landing-page-dolphin-laundry.git".to_string()
            )]
        );
    }

    #[test]
    fn apply_logs_audit_entry() {
        let (svc, _, audit) = make_service(vec![]);
        let p = svc
            .create("Work".into(), identity("a@example.com"), None)
            .unwrap();
        svc.apply(p.id).unwrap();
        let entries = audit.read_all().unwrap();
        assert!(entries.iter().any(|e| e.action == "profile.apply"));
    }
}
