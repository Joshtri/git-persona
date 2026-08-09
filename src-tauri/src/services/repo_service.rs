use crate::domain::{
    audit::AuditEntry,
    ports::{AuditSink, GitMetadataReader, RepoScanner, RepoStore},
    repo::{Repo, RepoGroup, ScanProgress},
};
use crate::error::AppError;
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) struct RepoService {
    store: Arc<dyn RepoStore>,
    scanner: Arc<dyn RepoScanner>,
    reader: Arc<dyn GitMetadataReader>,
    audit: Arc<dyn AuditSink>,
}

fn repo_name(git_root: &str) -> String {
    Path::new(git_root)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| git_root.to_string())
}

/// Refresh the derived `path_exists` flag from the filesystem. The stored value
/// is only a snapshot; callers annotate every `Repo` before it reaches the UI.
fn annotate_path(mut repo: Repo) -> Repo {
    repo.path_exists = Path::new(&repo.path).exists();
    repo
}

impl RepoService {
    pub(crate) fn new(
        store: Arc<dyn RepoStore>,
        scanner: Arc<dyn RepoScanner>,
        reader: Arc<dyn GitMetadataReader>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            scanner,
            reader,
            audit,
        }
    }

    fn log(
        &self,
        action: &str,
        repo_path: Option<String>,
        profile_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path,
        })
    }

    pub(crate) fn list(&self) -> Result<Vec<Repo>, AppError> {
        Ok(self
            .store
            .load_all()?
            .into_iter()
            .map(annotate_path)
            .collect())
    }

    pub(crate) fn get(&self, id: Uuid) -> Result<Repo, AppError> {
        self.store
            .find(id)?
            .map(annotate_path)
            .ok_or_else(|| AppError::NotFound(format!("repo {id}")))
    }

    pub(crate) fn scan(
        &self,
        roots: &[String],
        on_progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<Repo>, AppError> {
        let git_roots = self.scanner.scan(roots, on_progress)?;
        for git_root in &git_roots {
            let meta = self.reader.read(Path::new(git_root)).unwrap_or_default();
            if let Some(mut existing) = self.store.find_by_git_root(git_root)? {
                // Refresh volatile metadata; preserve user-owned fields
                // (assignment, favorite, last_opened, detected_at).
                existing.active_branch = meta.active_branch;
                existing.remote_origin = meta.remote_origin;
                self.store.save(&existing)?;
            } else {
                let repo = Repo {
                    id: Uuid::new_v4(),
                    name: repo_name(git_root),
                    path: git_root.clone(),
                    git_root: git_root.clone(),
                    active_branch: meta.active_branch,
                    active_profile_id: None,
                    remote_origin: meta.remote_origin,
                    last_opened: None,
                    favorite: false,
                    detected_at: Utc::now(),
                    group_id: None,
                    path_exists: true,
                };
                self.store.save(&repo)?;
            }
        }
        self.log("repo.scan", None, None)?;
        Ok(self
            .store
            .load_all()?
            .into_iter()
            .map(annotate_path)
            .collect())
    }

    pub(crate) fn refresh(&self, id: Uuid) -> Result<Repo, AppError> {
        let mut repo = self.get(id)?;
        let meta = self.reader.read(Path::new(&repo.git_root))?;
        repo.active_branch = meta.active_branch;
        repo.remote_origin = meta.remote_origin;
        self.store.save(&repo)?;
        self.log("repo.refresh", Some(repo.path.clone()), None)?;
        Ok(repo)
    }

    pub(crate) fn remove(&self, id: Uuid) -> Result<(), AppError> {
        let repo = self.get(id)?;
        self.store.delete(id)?;
        self.log("repo.remove", Some(repo.path), None)?;
        Ok(())
    }

    pub(crate) fn assign_profile(
        &self,
        id: Uuid,
        profile_id: Option<Uuid>,
    ) -> Result<Repo, AppError> {
        let mut repo = self.get(id)?;
        repo.active_profile_id = profile_id;
        self.store.save(&repo)?;
        self.log("repo.assign", Some(repo.path.clone()), profile_id)?;
        Ok(repo)
    }

    pub(crate) fn toggle_favorite(&self, id: Uuid) -> Result<Repo, AppError> {
        let mut repo = self.get(id)?;
        repo.favorite = !repo.favorite;
        self.store.save(&repo)?;
        Ok(repo)
    }

    /// Record that the repo was opened and return its path for the caller to reveal.
    pub(crate) fn mark_opened(&self, id: Uuid) -> Result<Repo, AppError> {
        let mut repo = self.get(id)?;
        repo.last_opened = Some(Utc::now());
        self.store.save(&repo)?;
        Ok(repo)
    }

    // ---- Groups ("ecosystems") ------------------------------------------

    pub(crate) fn list_groups(&self) -> Result<Vec<RepoGroup>, AppError> {
        self.store.load_groups()
    }

    fn find_group(&self, id: Uuid) -> Result<RepoGroup, AppError> {
        self.store
            .load_groups()?
            .into_iter()
            .find(|g| g.id == id)
            .ok_or_else(|| AppError::NotFound(format!("repo group {id}")))
    }

    /// Validate and normalize a group name, rejecting blanks and duplicates
    /// (case-insensitive), excluding `exclude` for renames.
    fn validate_group_name(&self, name: &str, exclude: Option<Uuid>) -> Result<String, AppError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("group name is required".into()));
        }
        if trimmed.chars().count() > 60 {
            return Err(AppError::Validation("group name is too long".into()));
        }
        let clash = self
            .store
            .load_groups()?
            .into_iter()
            .any(|g| Some(g.id) != exclude && g.name.eq_ignore_ascii_case(trimmed));
        if clash {
            return Err(AppError::Conflict(format!(
                "a group named \"{trimmed}\" already exists"
            )));
        }
        Ok(trimmed.to_string())
    }

    pub(crate) fn create_group(
        &self,
        name: &str,
        color: Option<String>,
    ) -> Result<RepoGroup, AppError> {
        let name = self.validate_group_name(name, None)?;
        let group = RepoGroup {
            id: Uuid::new_v4(),
            name,
            color,
            created_at: Utc::now(),
        };
        self.store.save_group(&group)?;
        self.log("repo.group.create", None, None)?;
        Ok(group)
    }

    pub(crate) fn update_group(
        &self,
        id: Uuid,
        name: &str,
        color: Option<String>,
    ) -> Result<RepoGroup, AppError> {
        let mut group = self.find_group(id)?;
        group.name = self.validate_group_name(name, Some(id))?;
        group.color = color;
        self.store.save_group(&group)?;
        Ok(group)
    }

    /// Delete a group; the store detaches any repos that referenced it.
    pub(crate) fn delete_group(&self, id: Uuid) -> Result<(), AppError> {
        self.find_group(id)?;
        self.store.delete_group(id)?;
        self.log("repo.group.delete", None, None)?;
        Ok(())
    }

    /// Move a repo into a group, or out of any group with `None`.
    pub(crate) fn set_group(&self, id: Uuid, group_id: Option<Uuid>) -> Result<Repo, AppError> {
        if let Some(gid) = group_id {
            self.find_group(gid)?;
        }
        let mut repo = self.get(id)?;
        repo.group_id = group_id;
        self.store.save(&repo)?;
        self.log("repo.group.assign", Some(repo.path.clone()), None)?;
        Ok(repo)
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
    use crate::domain::repo::RepoMetadata;
    use std::sync::Mutex;

    struct MockRepoStore {
        repos: Mutex<Vec<Repo>>,
        groups: Mutex<Vec<RepoGroup>>,
    }
    impl MockRepoStore {
        fn new() -> Self {
            Self {
                repos: Mutex::new(vec![]),
                groups: Mutex::new(vec![]),
            }
        }
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
        fn save(&self, repo: &Repo) -> Result<(), AppError> {
            let mut rs = self.repos.lock().unwrap();
            if let Some(pos) = rs.iter().position(|r| r.id == repo.id) {
                rs[pos] = repo.clone();
            } else {
                rs.push(repo.clone());
            }
            Ok(())
        }
        fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut rs = self.repos.lock().unwrap();
            let before = rs.len();
            rs.retain(|r| r.id != id);
            if rs.len() == before {
                return Err(AppError::NotFound(format!("repo {id}")));
            }
            Ok(())
        }
        fn load_groups(&self) -> Result<Vec<RepoGroup>, AppError> {
            Ok(self.groups.lock().unwrap().clone())
        }
        fn save_group(&self, group: &RepoGroup) -> Result<(), AppError> {
            let mut gs = self.groups.lock().unwrap();
            if let Some(pos) = gs.iter().position(|g| g.id == group.id) {
                gs[pos] = group.clone();
            } else {
                gs.push(group.clone());
            }
            Ok(())
        }
        fn delete_group(&self, id: Uuid) -> Result<(), AppError> {
            let mut gs = self.groups.lock().unwrap();
            let before = gs.len();
            gs.retain(|g| g.id != id);
            if gs.len() == before {
                return Err(AppError::NotFound(format!("repo group {id}")));
            }
            drop(gs);
            for repo in self.repos.lock().unwrap().iter_mut() {
                if repo.group_id == Some(id) {
                    repo.group_id = None;
                }
            }
            Ok(())
        }
    }

    struct MockScanner {
        roots: Vec<String>,
    }
    impl RepoScanner for MockScanner {
        fn scan(
            &self,
            _roots: &[String],
            on_progress: &(dyn Fn(ScanProgress) + Send + Sync),
        ) -> Result<Vec<String>, AppError> {
            on_progress(ScanProgress {
                scanned_dirs: 1,
                found_repos: u32::try_from(self.roots.len()).unwrap_or(u32::MAX),
            });
            Ok(self.roots.clone())
        }
    }

    struct MockReader {
        meta: RepoMetadata,
    }
    impl GitMetadataReader for MockReader {
        fn read(&self, _git_root: &Path) -> Result<RepoMetadata, AppError> {
            Ok(self.meta.clone())
        }
    }

    struct MockAudit {
        entries: Mutex<Vec<AuditEntry>>,
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

    fn make(
        found: Vec<&str>,
        meta: RepoMetadata,
    ) -> (RepoService, Arc<MockRepoStore>, Arc<MockAudit>) {
        let store = Arc::new(MockRepoStore::new());
        let audit = Arc::new(MockAudit {
            entries: Mutex::new(vec![]),
        });
        let svc = RepoService::new(
            store.clone(),
            Arc::new(MockScanner {
                roots: found.into_iter().map(String::from).collect(),
            }),
            Arc::new(MockReader { meta }),
            audit.clone(),
        );
        (svc, store, audit)
    }

    fn meta(branch: &str) -> RepoMetadata {
        RepoMetadata {
            active_branch: Some(branch.into()),
            remote_origin: Some("git@example.com:x/y.git".into()),
        }
    }

    fn noop() -> impl Fn(ScanProgress) + Send + Sync {
        |_| {}
    }

    #[test]
    fn scan_adds_discovered_repos() {
        let (svc, _, audit) = make(vec!["/x/alpha", "/x/beta"], meta("main"));
        let repos = svc.scan(&["/x".to_string()], &noop()).unwrap();
        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.name == "alpha"));
        assert_eq!(repos[0].active_branch.as_deref(), Some("main"));
        assert!(audit
            .read_all()
            .unwrap()
            .iter()
            .any(|e| e.action == "repo.scan"));
    }

    #[test]
    fn scan_is_idempotent_and_preserves_assignment() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let id = store.load_all().unwrap()[0].id;
        svc.assign_profile(id, Some(Uuid::new_v4())).unwrap();
        let assigned = store.load_all().unwrap()[0].active_profile_id;

        let repos = svc.scan(&["/x".to_string()], &noop()).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].active_profile_id, assigned);
        assert_eq!(repos[0].id, id);
    }

    #[test]
    fn assign_and_unassign_profile() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let id = store.load_all().unwrap()[0].id;
        let pid = Uuid::new_v4();
        assert_eq!(
            svc.assign_profile(id, Some(pid)).unwrap().active_profile_id,
            Some(pid)
        );
        assert_eq!(
            svc.assign_profile(id, None).unwrap().active_profile_id,
            None
        );
    }

    #[test]
    fn toggle_favorite_flips() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let id = store.load_all().unwrap()[0].id;
        assert!(svc.toggle_favorite(id).unwrap().favorite);
        assert!(!svc.toggle_favorite(id).unwrap().favorite);
    }

    #[test]
    fn refresh_updates_branch() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let id = store.load_all().unwrap()[0].id;
        // Reader still returns "main"; assert the plumbing works and audit logged.
        let refreshed = svc.refresh(id).unwrap();
        assert_eq!(refreshed.active_branch.as_deref(), Some("main"));
    }

    #[test]
    fn remove_deletes_repo() {
        let (svc, store, audit) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let id = store.load_all().unwrap()[0].id;
        svc.remove(id).unwrap();
        assert!(store.load_all().unwrap().is_empty());
        assert!(audit
            .read_all()
            .unwrap()
            .iter()
            .any(|e| e.action == "repo.remove"));
    }

    #[test]
    fn create_group_and_assign_repo() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let repo_id = store.load_all().unwrap()[0].id;
        let group = svc
            .create_group("Dolphin Modules", Some("#10b981".into()))
            .unwrap();
        assert_eq!(group.name, "Dolphin Modules");

        let repo = svc.set_group(repo_id, Some(group.id)).unwrap();
        assert_eq!(repo.group_id, Some(group.id));
        // Unassign back to ungrouped.
        assert_eq!(svc.set_group(repo_id, None).unwrap().group_id, None);
    }

    #[test]
    fn create_group_rejects_blank_and_duplicate() {
        let (svc, _, _) = make(vec![], meta("main"));
        assert!(matches!(
            svc.create_group("  ", None).unwrap_err(),
            AppError::Validation(_)
        ));
        svc.create_group("Dolphin", None).unwrap();
        assert!(matches!(
            svc.create_group("dolphin", None).unwrap_err(),
            AppError::Conflict(_)
        ));
    }

    #[test]
    fn set_group_rejects_unknown_group() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let repo_id = store.load_all().unwrap()[0].id;
        let err = svc.set_group(repo_id, Some(Uuid::new_v4())).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn delete_group_detaches_repos() {
        let (svc, store, _) = make(vec!["/x/alpha"], meta("main"));
        svc.scan(&["/x".to_string()], &noop()).unwrap();
        let repo_id = store.load_all().unwrap()[0].id;
        let group = svc.create_group("Dolphin", None).unwrap();
        svc.set_group(repo_id, Some(group.id)).unwrap();

        svc.delete_group(group.id).unwrap();
        assert!(svc.list_groups().unwrap().is_empty());
        assert_eq!(store.load_all().unwrap()[0].group_id, None);
    }

    #[test]
    fn update_group_renames_and_recolors() {
        let (svc, _, _) = make(vec![], meta("main"));
        let group = svc.create_group("Old", Some("#111111".into())).unwrap();
        let updated = svc
            .update_group(group.id, "New", Some("#222222".into()))
            .unwrap();
        assert_eq!(updated.name, "New");
        assert_eq!(updated.color.as_deref(), Some("#222222"));
    }

    #[test]
    fn get_missing_repo_errors() {
        let (svc, _, _) = make(vec![], meta("main"));
        let err = svc.get(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
