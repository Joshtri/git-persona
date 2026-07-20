use crate::domain::{ports::SshStore, ssh::SshKey};
use crate::error::AppError;
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const SSH_KEYS_KEY: &str = "ssh_keys";
const STORE_FILE: &str = "gitpersona.json";

pub(crate) struct TauriSshStore {
    app: tauri::AppHandle,
    lock: Mutex<()>,
}

impl TauriSshStore {
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

    fn read(store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<Vec<SshKey>, AppError> {
        match store.get(SSH_KEYS_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(vec![]),
        }
    }

    fn write(
        store: &tauri_plugin_store::Store<tauri::Wry>,
        keys: &[SshKey],
    ) -> Result<(), AppError> {
        let val = serde_json::to_value(keys).map_err(|e| AppError::Store(e.to_string()))?;
        store.set(SSH_KEYS_KEY, val);
        store.save().map_err(|e| AppError::Store(e.to_string()))
    }
}

impl SshStore for TauriSshStore {
    fn load_all(&self) -> Result<Vec<SshKey>, AppError> {
        self.with_store(Self::read)
    }

    fn find(&self, id: Uuid) -> Result<Option<SshKey>, AppError> {
        Ok(self.load_all()?.into_iter().find(|k| k.id == id))
    }

    fn find_by_path(&self, private_key_path: &str) -> Result<Option<SshKey>, AppError> {
        Ok(self
            .load_all()?
            .into_iter()
            .find(|k| k.private_key_path == private_key_path))
    }

    fn save(&self, key: &SshKey) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut keys = Self::read(store)?;
            if let Some(slot) = keys.iter_mut().find(|k| k.id == key.id) {
                *slot = key.clone();
            } else {
                keys.push(key.clone());
            }
            Self::write(store, &keys)
        })
    }

    fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut keys = Self::read(store)?;
            let before = keys.len();
            keys.retain(|k| k.id != id);
            if keys.len() == before {
                return Err(AppError::NotFound(format!("ssh key {id}")));
            }
            Self::write(store, &keys)
        })
    }
}
