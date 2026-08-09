use crate::domain::{
    credential::{Secret, VaultSnapshot},
    ports::CredentialVault,
};
use crate::error::AppError;

/// Fallback [`CredentialVault`] for platforms without a native integration yet
/// (macOS Keychain and Linux Secret Service arrive in a later sprint). Every
/// mutating operation reports `Unsupported` so the UI can surface a clear
/// message rather than silently losing a secret.
pub(crate) struct NoopCredentialVault;

fn unsupported() -> AppError {
    AppError::Unsupported("credential storage is only available on Windows in this release".into())
}

impl CredentialVault for NoopCredentialVault {
    fn store(
        &self,
        _target: &str,
        _username: &str,
        _secret: &Secret,
    ) -> Result<VaultSnapshot, AppError> {
        Err(unsupported())
    }

    fn username(&self, _target: &str) -> Result<Option<String>, AppError> {
        Ok(None)
    }

    fn reveal(&self, _target: &str) -> Result<Option<Secret>, AppError> {
        Err(unsupported())
    }

    fn promote(&self, _from: &str, _to: &str, _username: &str) -> Result<VaultSnapshot, AppError> {
        Err(unsupported())
    }

    fn delete(&self, _target: &str) -> Result<VaultSnapshot, AppError> {
        Err(unsupported())
    }

    fn restore(&self, _snapshot: &VaultSnapshot) -> Result<(), AppError> {
        Ok(())
    }
}
