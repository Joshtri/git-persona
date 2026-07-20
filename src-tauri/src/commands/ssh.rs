use crate::{
    domain::ssh::{GenerateParams, SshAlgorithm, SshKey},
    error::AppError,
    infra::paths,
    state::AppState,
};
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

#[tauri::command]
pub(crate) async fn ssh_list(state: State<'_, AppState>) -> Result<Vec<SshKey>, AppError> {
    state.ssh.list()
}

#[tauri::command]
pub(crate) async fn ssh_scan(state: State<'_, AppState>) -> Result<Vec<SshKey>, AppError> {
    let dir = paths::ssh_dir()?;
    state.ssh.scan(&dir)
}

#[tauri::command]
pub(crate) async fn ssh_import(
    private_key_path: String,
    host_alias: Option<String>,
    host_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<SshKey, AppError> {
    if private_key_path.trim().is_empty() {
        return Err(AppError::Validation("no key file selected".into()));
    }
    state.ssh.import(private_key_path, host_alias, host_name)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn ssh_generate(
    label: String,
    algorithm: SshAlgorithm,
    comment: Option<String>,
    output_dir: String,
    file_name: String,
    host_alias: Option<String>,
    host_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<SshKey, AppError> {
    state.ssh.generate(GenerateParams {
        label,
        algorithm,
        comment,
        output_dir,
        file_name,
        host_alias,
        host_name,
    })
}

#[tauri::command]
pub(crate) async fn ssh_remove(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.ssh.remove(id)
}

#[tauri::command]
pub(crate) async fn ssh_assign_profile(
    id: Uuid,
    profile_id: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<SshKey, AppError> {
    state.ssh.assign_profile(id, profile_id)
}

#[tauri::command]
pub(crate) async fn ssh_reveal(
    id: Uuid,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Only reveal a validated, stored key path — never arbitrary input.
    let key = state.ssh.mark_used(id)?;
    app.opener()
        .reveal_item_in_dir(&key.private_key_path)
        .map_err(|e| AppError::Io(e.to_string()))
}

#[tauri::command]
pub(crate) async fn ssh_reveal_folder(app: tauri::AppHandle) -> Result<(), AppError> {
    let dir = paths::ssh_dir()?;
    app.opener()
        .reveal_item_in_dir(&dir)
        .map_err(|e| AppError::Io(e.to_string()))
}

#[tauri::command]
pub(crate) async fn ssh_config_open(app: tauri::AppHandle) -> Result<(), AppError> {
    // Opens only the fixed ~/.ssh/config path in the default editor.
    let path = paths::ssh_config_path()?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::Io(e.to_string()))
}

#[tauri::command]
pub(crate) async fn ssh_config_preview(state: State<'_, AppState>) -> Result<String, AppError> {
    state.ssh.config_preview()
}
