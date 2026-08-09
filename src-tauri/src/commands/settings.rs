use crate::{domain::settings::AppSettings, error::AppError, state::AppState};
use tauri::State;

#[tauri::command]
pub(crate) async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state.settings.get()
}

#[tauri::command]
pub(crate) async fn settings_set(
    settings: AppSettings,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, AppError> {
    let saved = state.settings.set(settings)?;
    // Keep the workspace watcher in sync with the freshly persisted preferences
    // (e.g. Smart Switching toggled on/off from the Settings view).
    state.identity_switch.reconcile()?;
    // Register/unregister the OS autostart entry to match the saved preference.
    crate::sync_autostart(&app, saved.launch_at_startup);
    Ok(saved)
}
