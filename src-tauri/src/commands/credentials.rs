use crate::{domain::credential::Credential, error::AppError, state::AppState};
use tauri::State;
use uuid::Uuid;
use zeroize::Zeroizing;

#[tauri::command]
pub(crate) async fn credential_list(
    state: State<'_, AppState>,
) -> Result<Vec<Credential>, AppError> {
    state.credentials.list()
}

#[tauri::command]
pub(crate) async fn credential_create(
    profile_id: Option<Uuid>,
    host: String,
    username: String,
    token: String,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    // Own the secret in a zeroizing buffer for its whole (short) lifetime.
    let secret = Zeroizing::new(token);
    state.credentials.create(profile_id, &host, username, &secret)
}

#[tauri::command]
pub(crate) async fn credential_update(
    id: Uuid,
    username: String,
    token: Option<String>,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    let secret = token.map(Zeroizing::new);
    state.credentials.update(id, username, secret.as_ref())
}

#[tauri::command]
pub(crate) async fn credential_assign_profile(
    id: Uuid,
    profile_id: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    state.credentials.assign_profile(id, profile_id)
}

#[tauri::command]
pub(crate) async fn credential_switch(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    state.credentials.switch(id)
}

#[tauri::command]
pub(crate) async fn credential_delete(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.credentials.delete(id)
}

/// Open the native Windows Credential Manager UI. Windows-only; a no-op error
/// elsewhere until Keychain / Secret Service land.
#[tauri::command]
pub(crate) async fn credential_open_manager() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        std::process::Command::new("rundll32.exe")
            .args(["keymgr.dll,KRShowKeyMgr"])
            .spawn()
            .map_err(|e| AppError::Io(e.to_string()))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(AppError::Unsupported(
            "the Windows Credential Manager is only available on Windows".into(),
        ))
    }
}
