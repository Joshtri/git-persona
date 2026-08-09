// The single module permitted to use `unsafe`: it bridges to the Win32
// Credential Management API (CredRead/CredWrite/CredDelete). Every unsafe block
// is a direct FFI call with a documented safety rationale; no unsafe leaks past
// this file.
#![allow(unsafe_code, clippy::ptr_as_ptr)]

use crate::domain::{
    credential::{Secret, VaultSecret, VaultSnapshot},
    ports::CredentialVault,
};
use crate::error::AppError;
use std::ffi::c_void;
use windows::core::{HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NOT_FOUND, FILETIME};
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_FLAGS,
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};
use zeroize::{Zeroize, Zeroizing};

/// Native Windows Credential Manager adapter. Stores each secret as a generic
/// credential whose target matches Git's `wincred` convention so Git actually
/// uses it. Blobs are UTF-16LE, matching what Git reads/writes.
pub(crate) struct WindowsCredentialVault;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn utf16le_bytes(s: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(s.len() * 2);
    for unit in s.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn string_from_utf16le(bytes: &[u8]) -> String {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let lo = *chunk.first().unwrap_or(&0);
        let hi = *chunk.get(1).unwrap_or(&0);
        units.push(u16::from_le_bytes([lo, hi]));
    }
    String::from_utf16_lossy(&units)
}

fn is_not_found(err: &windows::core::Error) -> bool {
    err.code() == HRESULT::from_win32(ERROR_NOT_FOUND.0)
}

/// Read a target's username + secret blob. `Ok(None)` if the target is absent.
/// The returned blob holds secret bytes; callers must zeroize it.
fn read_raw(target: &str) -> Result<Option<(String, Vec<u8>)>, AppError> {
    let wide = to_wide(target);
    let mut pcred: *mut CREDENTIALW = std::ptr::null_mut();
    // SAFETY: `wide` is a valid NUL-terminated UTF-16 buffer alive for the call;
    // `pcred` is a valid out-pointer. On success the OS allocates `*pcred`.
    let result = unsafe { CredReadW(PCWSTR(wide.as_ptr()), CRED_TYPE_GENERIC, 0, &raw mut pcred) };
    match result {
        Ok(()) => {
            // SAFETY: on Ok, `pcred` points to a valid CREDENTIALW owned by us
            // until CredFree. We only read its fields.
            let cred = unsafe { &*pcred };
            let username = if cred.UserName.is_null() {
                String::new()
            } else {
                // SAFETY: UserName is a NUL-terminated wide string when non-null.
                unsafe { cred.UserName.to_string() }.unwrap_or_default()
            };
            let blob = if cred.CredentialBlobSize > 0 && !cred.CredentialBlob.is_null() {
                let len = cred.CredentialBlobSize as usize;
                // SAFETY: CredentialBlob points to CredentialBlobSize valid bytes.
                unsafe { std::slice::from_raw_parts(cred.CredentialBlob, len) }.to_vec()
            } else {
                Vec::new()
            };
            // SAFETY: `pcred` was allocated by CredReadW and is freed exactly once.
            unsafe { CredFree(pcred as *const c_void) };
            Ok(Some((username, blob)))
        }
        Err(e) if is_not_found(&e) => Ok(None),
        Err(e) => Err(AppError::Credential(e.to_string())),
    }
}

fn write_raw(target: &str, username: &str, blob: &[u8]) -> Result<(), AppError> {
    let mut target_w = to_wide(target);
    let mut user_w = to_wide(username);
    let mut blob_owned = blob.to_vec();
    let size = u32::try_from(blob_owned.len())
        .map_err(|_| AppError::Credential("secret too large".into()))?;

    let cred = CREDENTIALW {
        Flags: CRED_FLAGS(0),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target_w.as_mut_ptr()),
        Comment: PWSTR::null(),
        LastWritten: FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
        CredentialBlobSize: size,
        CredentialBlob: blob_owned.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: std::ptr::null_mut(),
        TargetAlias: PWSTR::null(),
        UserName: PWSTR(user_w.as_mut_ptr()),
    };

    // SAFETY: every pointer in `cred` refers to a local buffer that outlives the
    // call; CredWriteW copies the data, so the buffers can drop afterwards.
    let result = unsafe { CredWriteW(&raw const cred, 0) };
    blob_owned.zeroize();
    result.map_err(|e| AppError::Credential(e.to_string()))
}

fn delete_raw(target: &str) -> Result<(), AppError> {
    let wide = to_wide(target);
    // SAFETY: `wide` is a valid NUL-terminated UTF-16 buffer alive for the call.
    let result = unsafe { CredDeleteW(PCWSTR(wide.as_ptr()), CRED_TYPE_GENERIC, 0) };
    match result {
        Ok(()) => Ok(()),
        Err(e) if is_not_found(&e) => Ok(()),
        Err(e) => Err(AppError::Credential(e.to_string())),
    }
}

/// Capture a target's current contents for rollback, zeroizing the transient
/// blob once converted into a `Zeroizing` secret.
fn capture(target: &str) -> Result<VaultSnapshot, AppError> {
    let prior = read_raw(target)?.map(|(username, mut blob)| {
        let secret = Zeroizing::new(string_from_utf16le(&blob));
        blob.zeroize();
        VaultSecret { username, secret }
    });
    Ok(VaultSnapshot {
        target: target.into(),
        prior,
    })
}

fn verify(target: &str, username: &str) -> Result<(), AppError> {
    match read_raw(target)? {
        Some((stored, mut blob)) => {
            blob.zeroize();
            if stored == username {
                Ok(())
            } else {
                Err(AppError::Credential(
                    "credential verification failed".into(),
                ))
            }
        }
        None => Err(AppError::Credential(
            "credential verification failed".into(),
        )),
    }
}

impl CredentialVault for WindowsCredentialVault {
    fn store(
        &self,
        target: &str,
        username: &str,
        secret: &Secret,
    ) -> Result<VaultSnapshot, AppError> {
        let prior = capture(target)?;
        let mut blob = utf16le_bytes(secret.as_str());
        let result = write_raw(target, username, &blob);
        blob.zeroize();
        result?;
        verify(target, username)?;
        Ok(prior)
    }

    fn username(&self, target: &str) -> Result<Option<String>, AppError> {
        Ok(read_raw(target)?.map(|(username, mut blob)| {
            blob.zeroize();
            username
        }))
    }

    fn reveal(&self, target: &str) -> Result<Option<Secret>, AppError> {
        Ok(read_raw(target)?.map(|(_, mut blob)| {
            let secret = Zeroizing::new(string_from_utf16le(&blob));
            blob.zeroize();
            secret
        }))
    }

    fn promote(&self, from: &str, to: &str, username: &str) -> Result<VaultSnapshot, AppError> {
        let (_, mut blob) =
            read_raw(from)?.ok_or_else(|| AppError::Credential("backing secret missing".into()))?;
        let prior = capture(to)?;
        let result = write_raw(to, username, &blob);
        blob.zeroize();
        result?;
        verify(to, username)?;
        Ok(prior)
    }

    fn delete(&self, target: &str) -> Result<VaultSnapshot, AppError> {
        let prior = capture(target)?;
        delete_raw(target)?;
        Ok(prior)
    }

    fn restore(&self, snapshot: &VaultSnapshot) -> Result<(), AppError> {
        match &snapshot.prior {
            Some(vs) => {
                let mut blob = utf16le_bytes(vs.secret.as_str());
                let result = write_raw(&snapshot.target, &vs.username, &blob);
                blob.zeroize();
                result
            }
            None => delete_raw(&snapshot.target),
        }
    }
}
