use crate::{domain::platform::PlatformCapabilities, error::AppError};

/// Report the credential-related capabilities of the current platform so the
/// frontend can render only actions that can actually succeed. Pure and
/// state-free — the answer is fixed at compile time by the target `cfg`.
#[tauri::command]
pub(crate) async fn platform_capabilities() -> Result<PlatformCapabilities, AppError> {
    Ok(PlatformCapabilities::current())
}
