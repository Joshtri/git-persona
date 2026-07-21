use crate::domain::{ports::RuleStore, rule::Rule};
use crate::error::AppError;
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const RULES_KEY: &str = "rules";
const STORE_FILE: &str = "gitpersona.json";

/// Persists [`Rule`] definitions in the shared Tauri store. Rules are purely
/// declarative metadata — they hold no secrets.
pub(crate) struct TauriRuleStore {
    app: tauri::AppHandle,
    lock: Mutex<()>,
}

impl TauriRuleStore {
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

    fn read(store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<Vec<Rule>, AppError> {
        match store.get(RULES_KEY) {
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| AppError::Store(e.to_string()))
            }
            None => Ok(vec![]),
        }
    }

    fn write(
        store: &tauri_plugin_store::Store<tauri::Wry>,
        items: &[Rule],
    ) -> Result<(), AppError> {
        let val = serde_json::to_value(items).map_err(|e| AppError::Store(e.to_string()))?;
        store.set(RULES_KEY, val);
        store.save().map_err(|e| AppError::Store(e.to_string()))
    }
}

impl RuleStore for TauriRuleStore {
    fn load_all(&self) -> Result<Vec<Rule>, AppError> {
        self.with_store(Self::read)
    }

    fn find(&self, id: Uuid) -> Result<Option<Rule>, AppError> {
        Ok(self.load_all()?.into_iter().find(|r| r.id == id))
    }

    fn save(&self, rule: &Rule) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut items = Self::read(store)?;
            if let Some(slot) = items.iter_mut().find(|r| r.id == rule.id) {
                *slot = rule.clone();
            } else {
                items.push(rule.clone());
            }
            Self::write(store, &items)
        })
    }

    fn save_all(&self, rules: &[Rule]) -> Result<(), AppError> {
        self.with_store(|store| Self::write(store, rules))
    }

    fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.with_store(|store| {
            let mut items = Self::read(store)?;
            let before = items.len();
            items.retain(|r| r.id != id);
            if items.len() == before {
                return Err(AppError::NotFound(format!("rule {id}")));
            }
            Self::write(store, &items)
        })
    }
}
