use crate::domain::{credential::Credential, ports::CredentialStore};
use crate::error::AppError;
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const CREDENTIALS_KEY: &str = "credentials";
const STORE_FILE: &str = "gitpersona.json";

/// Persists [`Credential`] **metadata** in the shared Tauri store. Secrets are
/// never written here — they live only in the OS credential vault.
pub(crate) struct TauriCredentialStore {
    app: tauri::AppHandle,
    lock: Mutex<()>,
}

impl TauriCredentialStore {
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

    fn read(store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<Vec<Credential>, AppError> {
        match store.get(CREDENTIALS_KEY) {
            Some(v) => serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string())),
            None => Ok(vec![]),
        }
    }

    fn write(
        store: &tauri_plugin_store::Store<tauri::Wry>,
        items: &[Credential],
    ) -> Result<(), AppError> {
        let val = serde_json::to_value(items).map_err(|e| AppError::Store(e.to_string()))?;
        store.set(CREDENTIALS_KEY, val);
        store.save().map_err(|e| AppError::Store(e.to_string()))
    }
}

impl CredentialStore for TauriCredentialStore {
    fn load_all(&self) -> Result<Vec<Credential>, AppError> {
        self.with_store(Self::read)
    }

    fn find(&self, id: Uuid) -> Result<Option<Credential>, AppError> {
        Ok(self.load_all()?.into_iter().find(|c| c.id == id))
    }

    fn find_by_host(&self, profile_id: Uuid, host: &str) -> Result<Option<Credential>, AppError> {
        Ok(self
            .load_all()?
            .into_iter()
            .find(|c| c.profile_id == Some(profile_id) && c.host == host))
    }

    fn save(&self, credential: &Credential) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut items = Self::read(store)?;
            if let Some(slot) = items.iter_mut().find(|c| c.id == credential.id) {
                *slot = credential.clone();
            } else {
                items.push(credential.clone());
            }
            Self::write(store, &items)
        })
    }

    fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut items = Self::read(store)?;
            let before = items.len();
            items.retain(|c| c.id != id);
            if items.len() == before {
                return Err(AppError::NotFound(format!("credential {id}")));
            }
            Self::write(store, &items)
        })
    }
}
