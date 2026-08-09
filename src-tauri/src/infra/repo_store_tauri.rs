use crate::domain::{
    ports::RepoStore,
    repo::{Repo, RepoGroup},
};
use crate::error::AppError;
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const REPOS_KEY: &str = "repos";
const REPO_GROUPS_KEY: &str = "repo_groups";
const STORE_FILE: &str = "gitpersona.json";

pub(crate) struct TauriRepoStore {
    app: tauri::AppHandle,
    lock: Mutex<()>,
}

impl TauriRepoStore {
    pub(crate) fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            lock: Mutex::new(()),
        }
    }

    fn with_store<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&tauri_plugin_store::Store<tauri::Wry>) -> Result<T, AppError>,
    {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::Store("lock poisoned".into()))?;
        let store = self
            .app
            .store(STORE_FILE)
            .map_err(|e| AppError::Store(e.to_string()))?;
        f(&store)
    }

    fn read(store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<Vec<Repo>, AppError> {
        match store.get(REPOS_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(vec![]),
        }
    }

    fn write(
        store: &tauri_plugin_store::Store<tauri::Wry>,
        repos: &[Repo],
    ) -> Result<(), AppError> {
        let val = serde_json::to_value(repos).map_err(|e| AppError::Store(e.to_string()))?;
        store.set(REPOS_KEY, val);
        store.save().map_err(|e| AppError::Store(e.to_string()))
    }

    fn read_groups(
        store: &tauri_plugin_store::Store<tauri::Wry>,
    ) -> Result<Vec<RepoGroup>, AppError> {
        match store.get(REPO_GROUPS_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(vec![]),
        }
    }

    fn write_groups(
        store: &tauri_plugin_store::Store<tauri::Wry>,
        groups: &[RepoGroup],
    ) -> Result<(), AppError> {
        let val = serde_json::to_value(groups).map_err(|e| AppError::Store(e.to_string()))?;
        store.set(REPO_GROUPS_KEY, val);
        store.save().map_err(|e| AppError::Store(e.to_string()))
    }
}

impl RepoStore for TauriRepoStore {
    fn load_all(&self) -> Result<Vec<Repo>, AppError> {
        self.with_store(Self::read)
    }

    fn find(&self, id: Uuid) -> Result<Option<Repo>, AppError> {
        Ok(self.load_all()?.into_iter().find(|r| r.id == id))
    }

    fn find_by_git_root(&self, git_root: &str) -> Result<Option<Repo>, AppError> {
        Ok(self
            .load_all()?
            .into_iter()
            .find(|r| r.git_root == git_root))
    }

    fn save(&self, repo: &Repo) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut repos = Self::read(store)?;
            if let Some(slot) = repos.iter_mut().find(|r| r.id == repo.id) {
                *slot = repo.clone();
            } else {
                repos.push(repo.clone());
            }
            Self::write(store, &repos)
        })
    }

    fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut repos = Self::read(store)?;
            let before = repos.len();
            repos.retain(|r| r.id != id);
            if repos.len() == before {
                return Err(AppError::NotFound(format!("repo {id}")));
            }
            Self::write(store, &repos)
        })
    }

    fn load_groups(&self) -> Result<Vec<RepoGroup>, AppError> {
        self.with_store(Self::read_groups)
    }

    fn save_group(&self, group: &RepoGroup) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut groups = Self::read_groups(store)?;
            if let Some(slot) = groups.iter_mut().find(|g| g.id == group.id) {
                *slot = group.clone();
            } else {
                groups.push(group.clone());
            }
            Self::write_groups(store, &groups)
        })
    }

    fn delete_group(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut groups = Self::read_groups(store)?;
            let before = groups.len();
            groups.retain(|g| g.id != id);
            if groups.len() == before {
                return Err(AppError::NotFound(format!("repo group {id}")));
            }
            Self::write_groups(store, &groups)?;
            // Detach any repos that pointed at the deleted group.
            let mut repos = Self::read(store)?;
            let mut changed = false;
            for repo in &mut repos {
                if repo.group_id == Some(id) {
                    repo.group_id = None;
                    changed = true;
                }
            }
            if changed {
                Self::write(store, &repos)?;
            }
            Ok(())
        })
    }
}
