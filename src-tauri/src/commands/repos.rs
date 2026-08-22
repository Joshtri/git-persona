use crate::{
    domain::repo::{Repo, RepoGroup},
    error::AppError,
    state::AppState,
};
use std::path::Path;
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
    let repo = state.repos.assign_profile(id, profile_id)?;
    // Pin the resolved identity (author + path-scoped HTTPS credential) into the
    // repo's local Git config right away, so the correct author and account are
    // present before the user's next commit/pull rather than a step late.
    // Best-effort: a pin failure must not fail the assignment.
    if profile_id.is_some() {
        let _ = state.identity_switch.pin_repo(&repo.git_root);
    } else {
        // Un-assigning drops the per-repo credential and returns the repo to the
        // globally active credential.
        let _ = state.profiles.unpin_local(Path::new(&repo.git_root));
    }
    // Keep the Commit Guard marker in sync with the new expected identity when the
    // repository is protected (or auto-protect is on). Best-effort — never fails
    // the assignment.
    state.commit_guard.refresh_after_assignment(&repo.git_root);
    Ok(repo)
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

#[tauri::command]
pub(crate) async fn repo_group_list(
    state: State<'_, AppState>,
) -> Result<Vec<RepoGroup>, AppError> {
    state.repos.list_groups()
}

#[tauri::command]
pub(crate) async fn repo_group_create(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<RepoGroup, AppError> {
    state.repos.create_group(&name, color)
}

#[tauri::command]
pub(crate) async fn repo_group_update(
    id: Uuid,
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<RepoGroup, AppError> {
    state.repos.update_group(id, &name, color)
}

#[tauri::command]
pub(crate) async fn repo_group_delete(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.repos.delete_group(id)
}

#[tauri::command]
pub(crate) async fn repo_set_group(
    id: Uuid,
    group_id: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<Repo, AppError> {
    state.repos.set_group(id, group_id)
}

#[tauri::command]
pub(crate) async fn repo_set_group_many(
    ids: Vec<Uuid>,
    group_id: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<Vec<Repo>, AppError> {
    state.repos.set_group_many(ids, group_id)
}
