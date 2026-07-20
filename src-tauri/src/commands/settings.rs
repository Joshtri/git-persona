use crate::{domain::settings::AppSettings, error::AppError, state::AppState};
use tauri::State;

#[tauri::command]
pub(crate) async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state.settings.get()
}

#[tauri::command]
pub(crate) async fn settings_set(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<AppSettings, AppError> {
    state.settings.set(settings)
}
