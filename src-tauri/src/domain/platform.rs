use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The host operating system family, as `GitPersona`'s platform abstraction
/// distinguishes it. `Other` covers targets with no native secure-storage
/// integration (the no-op vault).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum PlatformOs {
    Windows,
    Macos,
    Linux,
    Other,
}

/// Whether a platform-specific capability is currently usable. `Planned`
/// distinguishes a capability that is intentionally on the roadmap (so the UI
/// can describe it honestly) from one that simply does not apply (`Unavailable`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum CapabilityStatus {
    Available,
    Planned,
    Unavailable,
}

/// The credential-related capabilities of the current platform, derived at
/// compile time from the same `cfg` gating that selects the concrete
/// [`CredentialVault`](crate::domain::ports::CredentialVault) implementation in
/// `state.rs`. The frontend consumes this to render only actions that can
/// actually succeed, so a user never sees a primary action that is guaranteed
/// to fail on their OS.
///
/// The three capabilities are deliberately independent — secure storage does
/// **not** imply automatic Git HTTPS credential integration:
///
/// - `credential_vault` — secrets can be stored in the OS-native secure store.
/// - `native_credential_manager` — `GitPersona` can open a native credential
///   management UI (only Windows' Credential Manager today).
/// - `git_credential_integration` — stored secrets are automatically consumed
///   by Git's HTTPS credential helper. `Available` only on Windows, whose vault
///   writes under Git's `wincred` target convention; `Planned` on macOS/Linux,
///   where the keyring vault stores under `GitPersona`'s own namespace.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct PlatformCapabilities {
    pub(crate) os: PlatformOs,
    pub(crate) credential_vault: CapabilityStatus,
    pub(crate) native_credential_manager: CapabilityStatus,
    pub(crate) git_credential_integration: CapabilityStatus,
}

impl PlatformCapabilities {
    /// Resolve the capabilities of the platform this binary was compiled for.
    /// Kept in lock-step with the vault selection in `AppState::init`.
    pub(crate) fn current() -> Self {
        #[cfg(windows)]
        {
            Self {
                os: PlatformOs::Windows,
                credential_vault: CapabilityStatus::Available,
                native_credential_manager: CapabilityStatus::Available,
                git_credential_integration: CapabilityStatus::Available,
            }
        }
        #[cfg(target_os = "macos")]
        {
            Self {
                os: PlatformOs::Macos,
                credential_vault: CapabilityStatus::Available,
                native_credential_manager: CapabilityStatus::Unavailable,
                git_credential_integration: CapabilityStatus::Planned,
            }
        }
        #[cfg(target_os = "linux")]
        {
            Self {
                os: PlatformOs::Linux,
                credential_vault: CapabilityStatus::Available,
                native_credential_manager: CapabilityStatus::Unavailable,
                git_credential_integration: CapabilityStatus::Planned,
            }
        }
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        {
            Self {
                os: PlatformOs::Other,
                credential_vault: CapabilityStatus::Unavailable,
                native_credential_manager: CapabilityStatus::Unavailable,
                git_credential_integration: CapabilityStatus::Unavailable,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_platform_reports_secure_storage() {
        let caps = PlatformCapabilities::current();
        // Every platform this project targets for release (Windows, macOS,
        // Linux) ships a secure vault; only the fallback `Other` does not.
        match caps.os {
            PlatformOs::Windows | PlatformOs::Macos | PlatformOs::Linux => {
                assert_eq!(caps.credential_vault, CapabilityStatus::Available);
            }
            PlatformOs::Other => {
                assert_eq!(caps.credential_vault, CapabilityStatus::Unavailable);
            }
        }
    }

    #[test]
    fn native_manager_implies_windows() {
        let caps = PlatformCapabilities::current();
        // Opening a native credential manager is only wired for Windows. If this
        // ever reports Available elsewhere, the `credential_open_manager` command
        // and its cfg guard have drifted from this model.
        if caps.native_credential_manager == CapabilityStatus::Available {
            assert_eq!(caps.os, PlatformOs::Windows);
        }
    }

    #[test]
    fn git_integration_available_only_on_windows() {
        let caps = PlatformCapabilities::current();
        if caps.git_credential_integration == CapabilityStatus::Available {
            assert_eq!(caps.os, PlatformOs::Windows);
        }
    }
}
