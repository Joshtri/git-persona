//! Cross-platform secure credential vault backed by the `keyring` crate.
//!
//! - **Linux** → Secret Service (GNOME Keyring / KWallet via the freedesktop
//!   Secret Service D-Bus API).
//! - **macOS** → Apple Keychain.
//!
//! Secrets live only in the OS secure store — never in GitPersona's own files,
//! never serialized into profile/credential metadata, never logged. When the
//! platform service is unavailable at runtime (e.g. a headless Linux session
//! with no Secret Service), operations return [`AppError::Unsupported`] instead
//! of falling back to any insecure store.
//!
//! Git integration note: unlike the Windows adapter — whose target string is the
//! exact key Git's `wincred` helper reads — this adapter stores secrets under
//! GitPersona's own service namespace. Automatic hand-off to Git's HTTPS helper
//! on these platforms is out of scope for this iteration; the vault provides
//! secure per-profile storage, retrieval, and switching within GitPersona.

use crate::domain::{
    credential::{Secret, VaultSecret, VaultSnapshot},
    ports::CredentialVault,
};
use crate::error::AppError;
use keyring::{Entry, Error as KeyringError};
use zeroize::Zeroizing;

/// The keyring "user" component. Both the username and the secret are packed
/// into the stored password blob (see [`pack`]/[`unpack`]), so a single opaque
/// `target` is the whole lookup key — matching the [`CredentialVault`] contract.
const KEYRING_USER: &str = "gitpersona";

/// Separator between the packed username and secret. NUL never appears in Git
/// usernames or tokens, so it is a safe, allocation-free delimiter.
const SEP: char = '\u{0}';

pub(crate) struct KeyringCredentialVault;

fn entry(target: &str) -> Result<Entry, AppError> {
    Entry::new(target, KEYRING_USER).map_err(map_err)
}

/// Map a keyring error to an [`AppError`]. A missing platform service becomes
/// `Unsupported` so the UI can show a clear "vault unavailable" state; a missing
/// entry is handled by callers (not routed here).
fn map_err(e: KeyringError) -> AppError {
    match e {
        KeyringError::NoStorageAccess(inner) => AppError::Unsupported(format!(
            "secure credential storage is unavailable on this system: {inner}"
        )),
        KeyringError::PlatformFailure(inner) => AppError::Unsupported(format!(
            "the OS credential service could not be reached: {inner}"
        )),
        other => AppError::Credential(other.to_string()),
    }
}

fn pack(username: &str, secret: &str) -> String {
    format!("{username}{SEP}{secret}")
}

/// Split a stored blob into `(username, secret)`. A blob written before this
/// scheme (no separator) is treated as secret-only with an empty username.
fn unpack(blob: &str) -> (String, Zeroizing<String>) {
    match blob.split_once(SEP) {
        Some((u, s)) => (u.to_string(), Zeroizing::new(s.to_string())),
        None => (String::new(), Zeroizing::new(blob.to_string())),
    }
}

/// Read the raw stored blob at `target`, or `None` when the entry is absent.
fn read_blob(target: &str) -> Result<Option<Zeroizing<String>>, AppError> {
    match entry(target)?.get_password() {
        Ok(blob) => Ok(Some(Zeroizing::new(blob))),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

fn write_blob(target: &str, username: &str, secret: &str) -> Result<(), AppError> {
    let packed = Zeroizing::new(pack(username, secret));
    entry(target)?.set_password(&packed).map_err(map_err)
}

fn delete_entry(target: &str) -> Result<(), AppError> {
    match entry(target)?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(map_err(e)),
    }
}

/// Capture a target's current contents for rollback.
fn capture(target: &str) -> Result<VaultSnapshot, AppError> {
    let prior = read_blob(target)?.map(|blob| {
        let (username, secret) = unpack(&blob);
        VaultSecret { username, secret }
    });
    Ok(VaultSnapshot {
        target: target.into(),
        prior,
    })
}

fn verify(target: &str, username: &str) -> Result<(), AppError> {
    match read_blob(target)? {
        Some(blob) => {
            let (stored, _secret) = unpack(&blob);
            if stored == username {
                Ok(())
            } else {
                Err(AppError::Credential("credential verification failed".into()))
            }
        }
        None => Err(AppError::Credential("credential verification failed".into())),
    }
}

impl CredentialVault for KeyringCredentialVault {
    fn store(
        &self,
        target: &str,
        username: &str,
        secret: &Secret,
    ) -> Result<VaultSnapshot, AppError> {
        let prior = capture(target)?;
        write_blob(target, username, secret.as_str())?;
        verify(target, username)?;
        Ok(prior)
    }

    fn username(&self, target: &str) -> Result<Option<String>, AppError> {
        Ok(read_blob(target)?.map(|blob| unpack(&blob).0))
    }

    fn reveal(&self, target: &str) -> Result<Option<Secret>, AppError> {
        Ok(read_blob(target)?.map(|blob| unpack(&blob).1))
    }

    fn promote(&self, from: &str, to: &str, username: &str) -> Result<VaultSnapshot, AppError> {
        let blob =
            read_blob(from)?.ok_or_else(|| AppError::Credential("backing secret missing".into()))?;
        let (_, secret) = unpack(&blob);
        let prior = capture(to)?;
        write_blob(to, username, secret.as_str())?;
        verify(to, username)?;
        Ok(prior)
    }

    fn delete(&self, target: &str) -> Result<VaultSnapshot, AppError> {
        let prior = capture(target)?;
        delete_entry(target)?;
        Ok(prior)
    }

    fn restore(&self, snapshot: &VaultSnapshot) -> Result<(), AppError> {
        match &snapshot.prior {
            Some(vs) => write_blob(&snapshot.target, &vs.username, vs.secret.as_str()),
            None => delete_entry(&snapshot.target),
        }
    }
}
