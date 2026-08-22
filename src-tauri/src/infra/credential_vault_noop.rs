use crate::domain::{
    credential::{Secret, VaultSnapshot},
    ports::CredentialVault,
};
use crate::error::AppError;

/// Fallback [`CredentialVault`] for platforms with no native secure-storage
/// integration — i.e. anything that is not Windows (Credential Manager), macOS
/// (Keychain), or Linux (Secret Service). Every mutating operation reports
/// `Unsupported` so the UI can surface a clear message rather than silently
/// losing a secret.
pub(crate) struct NoopCredentialVault;

fn unsupported() -> AppError {
    AppError::Unsupported("secure credential storage is not available on this platform".into())
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
