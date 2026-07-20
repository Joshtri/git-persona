use crate::{domain::repo::Repo, error::AppError, state::AppState};
use tauri::{Emitter, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

const SCAN_PROGRESS_EVENT: &str = "repo-scan-progress";

#[tauri::command]
pub(crate) async fn repo_scan(
    paths: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<Repo>, AppError> {
    if paths.is_empty() {
        return Err(AppError::Validation("no folders selected".into()));
    }
    let emitter = app.clone();
    let on_progress = move |progress| {
        let _ = emitter.emit(SCAN_PROGRESS_EVENT, progress);
    };
    state.repos.scan(&paths, &on_progress)
}

#[tauri::command]
pub(crate) async fn repo_list(state: State<'_, AppState>) -> Result<Vec<Repo>, AppError> {
    state.repos.list()
}

#[tauri::command]
pub(crate) async fn repo_get(id: Uuid, state: State<'_, AppState>) -> Result<Repo, AppError> {
    state.repos.get(id)
}

#[tauri::command]
pub(crate) async fn repo_refresh(id: Uuid, state: State<'_, AppState>) -> Result<Repo, AppError> {
    state.repos.refresh(id)
}

#[tauri::command]
pub(crate) async fn repo_remove(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.repos.remove(id)
}

#[tauri::command]
pub(crate) async fn repo_assign_profile(
    id: Uuid,
    profile_id: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<Repo, AppError> {
    state.repos.assign_profile(id, profile_id)
}

#[tauri::command]
pub(crate) async fn repo_toggle_favorite(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<Repo, AppError> {
    state.repos.toggle_favorite(id)
}

#[tauri::command]
pub(crate) async fn repo_reveal(
    id: Uuid,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Only reveal a path that is already stored and validated — never arbitrary input.
    let repo = state.repos.mark_opened(id)?;
    app.opener()
        .reveal_item_in_dir(&repo.path)
        .map_err(|e| AppError::Io(e.to_string()))
}
