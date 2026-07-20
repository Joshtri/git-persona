use crate::domain::{ports::ProfileStore, profile::Profile, settings::AppSettings};
use crate::error::AppError;
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const PROFILES_KEY: &str = "profiles";
const SETTINGS_KEY: &str = "settings";
const ACTIVE_ID_KEY: &str = "active_profile_id";
const STORE_FILE: &str = "gitpersona.json";

pub(crate) struct TauriProfileStore {
    app: tauri::AppHandle,
    lock: Mutex<()>,
}

impl TauriProfileStore {
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
}

impl ProfileStore for TauriProfileStore {
    fn load_all(&self) -> Result<Vec<Profile>, AppError> {
        self.with_store(|store| match store.get(PROFILES_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(vec![]),
        })
    }

    fn find(&self, id: Uuid) -> Result<Option<Profile>, AppError> {
        Ok(self.load_all()?.into_iter().find(|p| p.id == id))
    }

    fn save(&self, profile: &Profile) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut profiles: Vec<Profile> = match store.get(PROFILES_KEY) {
                Some(v) => {
                    serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))?
                }
                None => vec![],
            };
            if let Some(slot) = profiles.iter_mut().find(|p| p.id == profile.id) {
                *slot = profile.clone();
            } else {
                profiles.push(profile.clone());
            }
            let val =
                serde_json::to_value(&profiles).map_err(|e| AppError::Store(e.to_string()))?;
            store.set(PROFILES_KEY, val);
            store.save().map_err(|e| AppError::Store(e.to_string()))
        })
    }

    fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut profiles: Vec<Profile> = match store.get(PROFILES_KEY) {
                Some(v) => {
                    serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))?
                }
                None => return Ok(()),
            };
            let before = profiles.len();
            profiles.retain(|p| p.id != id);
            if profiles.len() == before {
                return Err(AppError::NotFound(format!("profile {id}")));
            }
            let val =
                serde_json::to_value(&profiles).map_err(|e| AppError::Store(e.to_string()))?;
            store.set(PROFILES_KEY, val);
            store.save().map_err(|e| AppError::Store(e.to_string()))
        })
    }

    fn load_settings(&self) -> Result<AppSettings, AppError> {
        self.with_store(|store| match store.get(SETTINGS_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(AppSettings::default()),
        })
    }

    fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.with_store(|store| {
            let val = serde_json::to_value(settings).map_err(|e| AppError::Store(e.to_string()))?;
            store.set(SETTINGS_KEY, val);
            store.save().map_err(|e| AppError::Store(e.to_string()))
        })
    }

    fn get_active_id(&self) -> Result<Option<Uuid>, AppError> {
        self.with_store(|store| match store.get(ACTIVE_ID_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(None),
        })
    }

    fn set_active_id(&self, id: Option<Uuid>) -> Result<(), AppError> {
        self.with_store(|store| {
            let val = serde_json::to_value(id).map_err(|e| AppError::Store(e.to_string()))?;
            store.set(ACTIVE_ID_KEY, val);
            store.save().map_err(|e| AppError::Store(e.to_string()))
        })
    }
}
