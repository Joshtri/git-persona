use crate::{domain::identity_switch::SmartSwitchStatus, error::AppError, state::AppState};
use tauri::State;

#[tauri::command]
pub(crate) async fn smart_switch_status(
    state: State<'_, AppState>,
) -> Result<SmartSwitchStatus, AppError> {
    state.identity_switch.status()
}

#[tauri::command]
pub(crate) async fn smart_switch_set_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<SmartSwitchStatus, AppError> {
    state.identity_switch.set_enabled(enabled)?;
    state.identity_switch.status()
}

#[tauri::command]
pub(crate) async fn smart_switch_restart(
    state: State<'_, AppState>,
) -> Result<SmartSwitchStatus, AppError> {
    state.identity_switch.restart()?;
    state.identity_switch.status()
}

#[tauri::command]
pub(crate) async fn smart_switch_pause(
    state: State<'_, AppState>,
) -> Result<SmartSwitchStatus, AppError> {
    state.identity_switch.pause();
    state.identity_switch.status()
}

#[tauri::command]
pub(crate) async fn smart_switch_resume(
    state: State<'_, AppState>,
) -> Result<SmartSwitchStatus, AppError> {
    state.identity_switch.resume();
    state.identity_switch.status()
}

#[tauri::command]
pub(crate) async fn smart_switch_confirm(
    git_root: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.identity_switch.confirm(&git_root)
}

#[tauri::command]
pub(crate) async fn smart_switch_cancel(
    git_root: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.identity_switch.cancel(&git_root)
}
