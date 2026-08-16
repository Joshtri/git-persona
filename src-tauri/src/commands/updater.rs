use serde::Serialize;
use tauri_plugin_updater::UpdaterExt;

use crate::error::AppError;

#[derive(Serialize)]
pub(crate) struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
}

#[tauri::command]
pub(crate) async fn updater_check(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, AppError> {
    let updater = app
        .updater()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let update = updater
        .check()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        body: u.body.clone(),
    }))
}

#[tauri::command]
pub(crate) async fn updater_install(app: tauri::AppHandle) -> Result<(), AppError> {
    let updater = app
        .updater()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    else {
        return Ok(());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    app.request_restart();
    #[allow(unreachable_code)]
    Ok(())
}
