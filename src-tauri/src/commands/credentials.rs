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
    pin: Option<String>,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    // Own the secret in a zeroizing buffer for its whole (short) lifetime.
    let secret = Zeroizing::new(token);
    let credential = state
        .credentials
        .create(profile_id, &host, username, &secret)?;
    // An optional PIN makes the token revealable later. Apply it as a second
    // step; the credential itself is already safely stored if this is skipped.
    match pin {
        Some(pin) => {
            let pin = Zeroizing::new(pin);
            state.credentials.set_pin(credential.id, &pin)
        }
        None => Ok(credential),
    }
}

/// Set (or replace) a credential's reveal PIN. Enables revealing the token for
/// credentials that were created without one.
#[tauri::command]
pub(crate) async fn credential_set_pin(
    id: Uuid,
    pin: String,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    let pin = Zeroizing::new(pin);
    state.credentials.set_pin(id, &pin)
}

/// Read a credential's token back after verifying its PIN. Returns the raw token
/// to the caller — the single command that ever surfaces secret material.
#[tauri::command]
pub(crate) async fn credential_reveal(
    id: Uuid,
    pin: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let pin = Zeroizing::new(pin);
    let secret = state.credentials.reveal(id, &pin)?;
    Ok(secret.as_str().to_owned())
}

#[tauri::command]
pub(crate) async fn credential_update(
    id: Uuid,
    username: String,
    token: Option<String>,
    state: State<'_, AppState>,
) -> Result<Credential, AppError> {
    let rotated = token.is_some();
    let secret = token.map(Zeroizing::new);
    let credential = state.credentials.update(id, username, secret.as_ref())?;
    // A rotated token only lands in the backing vault slot. Re-promote it onto
    // the canonical and per-repo path-scoped targets that Git actually reads, so
    // an assigned profile keeps pushing without a manual re-assignment. Best-
    // effort: the token is already saved, so a re-pin hiccup must not fail here.
    if rotated {
        if let Some(profile_id) = credential.profile_id {
            let _ = state.profiles.repin_credentials(profile_id);
        }
    }
    Ok(credential)
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
pub(crate) async fn credential_delete(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.credentials.delete(id)
}

/// Open the native Windows Credential Manager UI. Windows-only — no reliable,
/// universal way to launch a credential manager exists on Linux/macOS, so this
/// returns `Unsupported` there. The frontend gates the action on
/// `platform_capabilities().native_credential_manager`, so this error is a
/// backend safety net, not something a user reaches by clicking a visible button.
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
