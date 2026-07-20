use crate::domain::{
    audit::AuditEntry,
    identity::Identity,
    ports::{AuditSink, GitConfigBackend, ProfileStore, RepoStore},
    profile::Profile,
};
use crate::error::AppError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) struct ProfileService {
    store: Arc<dyn ProfileStore>,
    repos: Arc<dyn RepoStore>,
    git_config: Arc<dyn GitConfigBackend>,
    audit: Arc<dyn AuditSink>,
}

impl ProfileService {
    pub(crate) fn new(
        store: Arc<dyn ProfileStore>,
        repos: Arc<dyn RepoStore>,
        git_config: Arc<dyn GitConfigBackend>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            repos,
            git_config,
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

    pub(crate) fn apply(&self, id: Uuid) -> Result<(), AppError> {
        let profile = self.find(id)?;
        self.git_config.set_global_name(&profile.identity.name)?;
        self.git_config.set_global_email(&profile.identity.email)?;
        self.git_config
            .set_global_signing_key(profile.identity.signing_key.as_deref())?;
        self.store.set_active_id(Some(id))?;
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "profile.apply".into(),
            profile_id: Some(id),
            repo_path: None,
        })?;
        Ok(())
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
    use crate::domain::{audit::AuditEntry, ports::AuditSink, repo::Repo, settings::AppSettings};
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
    }

    impl MockGitConfig {
        fn new() -> Self {
            Self {
                name: Mutex::new(None),
                email: Mutex::new(None),
                signing_key: Mutex::new(None),
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
        fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError> {
            *self.signing_key.lock().unwrap() = key.map(Into::into);
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
            audit.clone(),
        );
        (svc, git_config, audit)
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
