use crate::{domain::bootstrap::BootstrapData, error::AppError, state::AppState};
use tauri::State;

/// Fetch the client initialization data from the `GitPersona` server.
///
/// Callers should treat any error as non-fatal — the server may be
/// temporarily unavailable, and the desktop must remain fully functional
/// while offline. The TypeScript store silently swallows errors here.
#[tauri::command]
pub(crate) async fn bootstrap_fetch(
    state: State<'_, AppState>,
) -> Result<BootstrapData, AppError> {
    let version = env!("CARGO_PKG_VERSION");
    state.bootstrap.fetch(version, "stable").await
}
