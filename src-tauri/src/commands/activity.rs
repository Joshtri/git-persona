use crate::{domain::audit::AuditEntry, error::AppError, state::AppState};
use tauri::State;

#[tauri::command]
pub(crate) async fn activity_list(state: State<'_, AppState>) -> Result<Vec<AuditEntry>, AppError> {
    state.activity.list()
}

#[tauri::command]
pub(crate) async fn activity_purge(
    state: State<'_, AppState>,
    days: u32,
) -> Result<u32, AppError> {
    state.activity.purge(days)
}
