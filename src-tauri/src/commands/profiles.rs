use crate::{
    domain::{identity::Identity, profile::Profile},
    error::AppError,
    state::AppState,
};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub(crate) async fn profile_list(state: State<'_, AppState>) -> Result<Vec<Profile>, AppError> {
    state.profiles.list()
}

#[tauri::command]
pub(crate) async fn profile_create(
    label: String,
    name: String,
    email: String,
    signing_key: Option<String>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Profile, AppError> {
    let signing_key = signing_key.filter(|k| !k.trim().is_empty());
    let identity = Identity {
        name,
        email,
        signing_key,
    };
    state.profiles.create(label, identity, color)
}

#[tauri::command]
pub(crate) async fn profile_update(
    id: Uuid,
    label: String,
    name: String,
    email: String,
    signing_key: Option<String>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Profile, AppError> {
    let signing_key = signing_key.filter(|k| !k.trim().is_empty());
    let identity = Identity {
        name,
        email,
        signing_key,
    };
    state.profiles.update(id, label, identity, color)
}

#[tauri::command]
pub(crate) async fn profile_delete(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.profiles.delete(id)
}

#[tauri::command]
pub(crate) async fn profile_apply(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.profiles.apply(id)
}

#[tauri::command]
pub(crate) async fn profile_get_active(
    state: State<'_, AppState>,
) -> Result<Option<Profile>, AppError> {
    state.profiles.get_active()
}
