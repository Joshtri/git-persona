use crate::domain::{ports::ProfileStore, settings::AppSettings};
use crate::error::AppError;
use std::sync::Arc;

pub(crate) struct SettingsService {
    store: Arc<dyn ProfileStore>,
}

impl SettingsService {
    pub(crate) fn new(store: Arc<dyn ProfileStore>) -> Self {
        Self { store }
    }

    pub(crate) fn get(&self) -> Result<AppSettings, AppError> {
        self.store.load_settings()
    }

    pub(crate) fn set(&self, settings: AppSettings) -> Result<AppSettings, AppError> {
        self.store.save_settings(&settings)?;
        Ok(settings)
    }
}
