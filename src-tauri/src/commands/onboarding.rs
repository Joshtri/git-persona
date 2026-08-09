use crate::{
    domain::onboarding::SystemScan, error::AppError, infra::paths,
    services::onboarding_service::ApplyParams, state::AppState,
};
use tauri::State;

#[tauri::command]
pub(crate) async fn onboarding_scan(state: State<'_, AppState>) -> Result<SystemScan, AppError> {
    let dir = paths::ssh_dir()?;
    state.onboarding.scan(&dir)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn onboarding_apply(
    create_profile: bool,
    label: String,
    name: String,
    email: String,
    signing_key: Option<String>,
    color: Option<String>,
    ssh_paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.onboarding.apply(ApplyParams {
        create_profile,
        label,
        name,
        email,
        signing_key,
        color,
        ssh_paths,
    })
}

#[tauri::command]
pub(crate) async fn onboarding_skip(state: State<'_, AppState>) -> Result<(), AppError> {
    state.onboarding.skip()
}
