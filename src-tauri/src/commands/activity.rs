use crate::{domain::audit::AuditEntry, error::AppError, state::AppState};
use tauri::State;

#[tauri::command]
pub(crate) async fn activity_list(state: State<'_, AppState>) -> Result<Vec<AuditEntry>, AppError> {
    state.activity.list()
}
