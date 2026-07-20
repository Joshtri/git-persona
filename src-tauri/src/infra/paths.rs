use crate::error::AppError;
use std::path::PathBuf;

pub(crate) fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    tauri::Manager::path(app)
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub(crate) fn audit_log_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    Ok(app_data_dir(app)?.join("audit.jsonl"))
}

/// The user's `~/.ssh` directory. Not created here — callers that write must
/// create it explicitly.
pub(crate) fn ssh_dir() -> Result<PathBuf, AppError> {
    dirs_next::home_dir()
        .map(|h| h.join(".ssh"))
        .ok_or_else(|| AppError::Io("could not resolve home directory".into()))
}

/// Path to `~/.ssh/config`.
pub(crate) fn ssh_config_path() -> Result<PathBuf, AppError> {
    Ok(ssh_dir()?.join("config"))
}
