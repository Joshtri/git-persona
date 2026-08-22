use crate::{
    domain::{commit_guard::CommitGuardStatus, settings::GuardMode},
    error::AppError,
    state::AppState,
};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub(crate) async fn commit_guard_status(
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.status()
}

#[tauri::command]
pub(crate) async fn commit_guard_set_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.set_enabled(enabled)
}

#[tauri::command]
pub(crate) async fn commit_guard_set_mode(
    mode: GuardMode,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.set_mode(mode)
}

#[tauri::command]
pub(crate) async fn commit_guard_set_auto_protect(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.set_auto_protect(enabled)
}

#[tauri::command]
pub(crate) async fn commit_guard_install(
    repo_id: Uuid,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.install(repo_id)
}

#[tauri::command]
pub(crate) async fn commit_guard_repair(
    repo_id: Uuid,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.repair(repo_id)
}

#[tauri::command]
pub(crate) async fn commit_guard_uninstall(
    repo_id: Uuid,
    state: State<'_, AppState>,
) -> Result<CommitGuardStatus, AppError> {
    state.commit_guard.uninstall(repo_id)
}
