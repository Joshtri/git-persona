use crate::domain::{
    credential::{Protocol, SUPPORTED_HOSTS},
    identity::Identity,
    onboarding::{DetectedCredential, DetectedIdentity, DetectedSshKey, SystemScan},
    ports::{CredentialVault, GitConfigBackend, ProfileStore, SshKeyReader, SshScanner},
};
use crate::error::AppError;
use crate::services::{
    credential_service::canonical_target, profile_service::ProfileService, ssh_service::SshService,
};
use std::path::Path;
use std::sync::Arc;

/// User selections collected by the onboarding wizard and passed to
/// [`OnboardingService::apply`].
pub(crate) struct ApplyParams {
    pub(crate) create_profile: bool,
    pub(crate) label: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) signing_key: Option<String>,
    pub(crate) color: Option<String>,
    /// Absolute paths of the SSH private keys the user chose to import.
    pub(crate) ssh_paths: Vec<String>,
}

/// First-run helper that detects existing Git setup (identity, SSH keys, HTTPS
/// credential usernames) and, on confirmation, imports the selected pieces.
///
/// Detection is strictly read-only. `apply` reuses the already-validated
/// [`ProfileService::create`] and [`SshService::import`] so onboarding never
/// duplicates business rules or audit logging.
pub(crate) struct OnboardingService {
    git_config: Arc<dyn GitConfigBackend>,
    vault: Arc<dyn CredentialVault>,
    ssh_scanner: Arc<dyn SshScanner>,
    ssh_reader: Arc<dyn SshKeyReader>,
    profiles: Arc<ProfileService>,
    ssh: Arc<SshService>,
    store: Arc<dyn ProfileStore>,
}

impl OnboardingService {
    pub(crate) fn new(
        git_config: Arc<dyn GitConfigBackend>,
        vault: Arc<dyn CredentialVault>,
        ssh_scanner: Arc<dyn SshScanner>,
        ssh_reader: Arc<dyn SshKeyReader>,
        profiles: Arc<ProfileService>,
        ssh: Arc<SshService>,
        store: Arc<dyn ProfileStore>,
    ) -> Self {
        Self {
            git_config,
            vault,
            ssh_scanner,
            ssh_reader,
            profiles,
            ssh,
            store,
        }
    }

    /// Whether the user has already been through onboarding — either the flag is
    /// set or at least one profile already exists.
    fn already_onboarded(&self) -> Result<bool, AppError> {
        Ok(self.store.load_settings()?.onboarded || !self.store.load_all()?.is_empty())
    }

    /// Read-only scan of the system for existing Git setup. Writes nothing.
    pub(crate) fn scan(&self, ssh_dir: &Path) -> Result<SystemScan, AppError> {
        let already = self.already_onboarded()?;
        scan_system(
            self.git_config.as_ref(),
            self.vault.as_ref(),
            self.ssh_scanner.as_ref(),
            self.ssh_reader.as_ref(),
            ssh_dir,
            already,
        )
    }

    /// Import the user's selections, then mark onboarding complete. Idempotent:
    /// a profile whose email already exists or an SSH key already imported is
    /// skipped rather than failing the whole flow.
    pub(crate) fn apply(&self, params: ApplyParams) -> Result<(), AppError> {
        if params.create_profile {
            let identity = Identity {
                name: params.name,
                email: params.email,
                signing_key: params.signing_key.filter(|k| !k.trim().is_empty()),
            };
            match self.profiles.create(params.label, identity, params.color) {
                Ok(_) | Err(AppError::Conflict(_)) => {}
                Err(e) => return Err(e),
            }
        }

        for path in params.ssh_paths {
            match self.ssh.import(path, None, None) {
                Ok(_) | Err(AppError::Conflict(_)) => {}
                Err(e) => return Err(e),
            }
        }

        self.mark_onboarded()
    }

    /// Record that onboarding is done without importing anything (wizard skipped).
    pub(crate) fn skip(&self) -> Result<(), AppError> {
        self.mark_onboarded()
    }

    fn mark_onboarded(&self) -> Result<(), AppError> {
        let mut settings = self.store.load_settings()?;
        settings.onboarded = true;
        self.store.save_settings(&settings)
    }
}

/// Pure detection over the ports, extracted so it can be unit-tested without the
/// profile/SSH services. Never writes.
fn scan_system(
    git_config: &dyn GitConfigBackend,
    vault: &dyn CredentialVault,
    ssh_scanner: &dyn SshScanner,
    ssh_reader: &dyn SshKeyReader,
    ssh_dir: &Path,
    already_onboarded: bool,
) -> Result<SystemScan, AppError> {
    let identity = detect_identity(git_config)?;
    let ssh_keys = detect_ssh_keys(ssh_scanner, ssh_reader, ssh_dir)?;
    let credentials = detect_credentials(vault)?;
    Ok(SystemScan {
        identity,
        ssh_keys,
        credentials,
        already_onboarded,
    })
}

fn detect_identity(
    git_config: &dyn GitConfigBackend,
) -> Result<Option<DetectedIdentity>, AppError> {
    let name = git_config
        .get_global_name()?
        .filter(|s| !s.trim().is_empty());
    let email = git_config
        .get_global_email()?
        .filter(|s| !s.trim().is_empty());
    if name.is_none() && email.is_none() {
        return Ok(None);
    }
    let signing_key = git_config
        .get_global_signing_key()?
        .filter(|s| !s.trim().is_empty());
    Ok(Some(DetectedIdentity {
        name,
        email,
        signing_key,
    }))
}

fn detect_ssh_keys(
    ssh_scanner: &dyn SshScanner,
    ssh_reader: &dyn SshKeyReader,
    ssh_dir: &Path,
) -> Result<Vec<DetectedSshKey>, AppError> {
    let mut keys = Vec::new();
    for path in ssh_scanner.scan(ssh_dir)? {
        // Unparseable / unsupported keys are silently skipped — the same policy
        // as SshService::scan.
        let Ok(meta) = ssh_reader.read(Path::new(&path)) else {
            continue;
        };
        keys.push(DetectedSshKey {
            private_key_path: path,
            algorithm: meta.algorithm,
            fingerprint: meta.fingerprint,
            comment: meta.comment,
        });
    }
    Ok(keys)
}

fn detect_credentials(vault: &dyn CredentialVault) -> Result<Vec<DetectedCredential>, AppError> {
    let mut found = Vec::new();
    for host in SUPPORTED_HOSTS {
        let target = canonical_target(host, Protocol::Https);
        if let Some(username) = vault.username(&target)? {
            if !username.trim().is_empty() {
                found.push(DetectedCredential {
                    host: (*host).to_string(),
                    username,
                });
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_pass_by_value
)]
mod tests {
    use super::*;
    use crate::domain::{
        credential::{Secret, VaultSnapshot},
        identity::Identity,
        ssh::{SshAlgorithm, SshKeyMetadata},
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- mock ports ------------------------------------------------------

    #[derive(Default)]
    struct MockGitConfig {
        name: Option<String>,
        email: Option<String>,
        signing_key: Option<String>,
    }
    impl GitConfigBackend for MockGitConfig {
        fn get_global_name(&self) -> Result<Option<String>, AppError> {
            Ok(self.name.clone())
        }
        fn get_global_email(&self) -> Result<Option<String>, AppError> {
            Ok(self.email.clone())
        }
        fn get_global_signing_key(&self) -> Result<Option<String>, AppError> {
            Ok(self.signing_key.clone())
        }
        fn set_global_name(&self, _name: &str) -> Result<(), AppError> {
            Ok(())
        }
        fn set_global_email(&self, _email: &str) -> Result<(), AppError> {
            Ok(())
        }
        fn set_global_signing_key(&self, _key: Option<&str>) -> Result<(), AppError> {
            Ok(())
        }
        fn get_local_name(&self, _root: &Path) -> Result<Option<String>, AppError> {
            Ok(self.name.clone())
        }
        fn get_local_email(&self, _root: &Path) -> Result<Option<String>, AppError> {
            Ok(self.email.clone())
        }
        fn set_local_identity(&self, _root: &Path, _id: &Identity) -> Result<(), AppError> {
            Ok(())
        }
        fn set_local_use_http_path(
            &self,
            _root: &Path,
            _host: &str,
            _enabled: bool,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockVault {
        usernames: HashMap<String, String>,
    }
    impl CredentialVault for MockVault {
        fn store(
            &self,
            target: &str,
            _username: &str,
            _secret: &Secret,
        ) -> Result<VaultSnapshot, AppError> {
            Ok(VaultSnapshot {
                target: target.into(),
                prior: None,
            })
        }
        fn username(&self, target: &str) -> Result<Option<String>, AppError> {
            Ok(self.usernames.get(target).cloned())
        }
        fn reveal(&self, _target: &str) -> Result<Option<Secret>, AppError> {
            Ok(None)
        }
        fn promote(&self, _f: &str, to: &str, _u: &str) -> Result<VaultSnapshot, AppError> {
            Ok(VaultSnapshot {
                target: to.into(),
                prior: None,
            })
        }
        fn delete(&self, target: &str) -> Result<VaultSnapshot, AppError> {
            Ok(VaultSnapshot {
                target: target.into(),
                prior: None,
            })
        }
        fn restore(&self, _snapshot: &VaultSnapshot) -> Result<(), AppError> {
            Ok(())
        }
    }

    struct MockScanner {
        found: Vec<String>,
    }
    impl SshScanner for MockScanner {
        fn scan(&self, _ssh_dir: &Path) -> Result<Vec<String>, AppError> {
            Ok(self.found.clone())
        }
    }

    struct MockReader {
        table: Mutex<Vec<(String, SshKeyMetadata)>>,
    }
    impl SshKeyReader for MockReader {
        fn read(&self, path: &Path) -> Result<SshKeyMetadata, AppError> {
            let key = path.to_string_lossy().into_owned();
            self.table
                .lock()
                .unwrap()
                .iter()
                .find(|(p, _)| *p == key)
                .map(|(_, m)| m.clone())
                .ok_or_else(|| AppError::Crypto("unparseable".into()))
        }
    }

    fn meta() -> SshKeyMetadata {
        SshKeyMetadata {
            algorithm: SshAlgorithm::Ed25519,
            fingerprint: "SHA256:abc".into(),
            comment: Some("me@host".into()),
            public_key_path: None,
        }
    }

    fn run_scan(
        git: MockGitConfig,
        vault: MockVault,
        found: Vec<&str>,
        known: Vec<&str>,
        already: bool,
    ) -> SystemScan {
        let scanner = MockScanner {
            found: found.into_iter().map(String::from).collect(),
        };
        let reader = MockReader {
            table: Mutex::new(known.into_iter().map(|p| (p.to_string(), meta())).collect()),
        };
        scan_system(&git, &vault, &scanner, &reader, Path::new("/ssh"), already).unwrap()
    }

    // ---- tests -----------------------------------------------------------

    #[test]
    fn detects_global_identity() {
        let git = MockGitConfig {
            name: Some("Alice".into()),
            email: Some("alice@example.com".into()),
            signing_key: Some("KEY1".into()),
        };
        let scan = run_scan(git, MockVault::default(), vec![], vec![], false);
        let id = scan.identity.unwrap();
        assert_eq!(id.name.as_deref(), Some("Alice"));
        assert_eq!(id.email.as_deref(), Some("alice@example.com"));
        assert_eq!(id.signing_key.as_deref(), Some("KEY1"));
    }

    #[test]
    fn identity_none_when_name_and_email_blank() {
        let git = MockGitConfig {
            name: Some("  ".into()),
            email: None,
            signing_key: None,
        };
        let scan = run_scan(git, MockVault::default(), vec![], vec![], false);
        assert!(scan.identity.is_none());
    }

    #[test]
    fn detects_parseable_ssh_and_skips_the_rest() {
        let scan = run_scan(
            MockGitConfig::default(),
            MockVault::default(),
            vec!["/ssh/id_ed25519", "/ssh/garbage"],
            vec!["/ssh/id_ed25519"],
            false,
        );
        assert_eq!(scan.ssh_keys.len(), 1);
        assert_eq!(scan.ssh_keys[0].private_key_path, "/ssh/id_ed25519");
        assert_eq!(scan.ssh_keys[0].algorithm, SshAlgorithm::Ed25519);
    }

    #[test]
    fn detects_credential_usernames_for_supported_hosts() {
        let mut usernames = HashMap::new();
        usernames.insert("git:https://github.com".to_string(), "octocat".to_string());
        usernames.insert("git:https://gitlab.com".to_string(), "  ".to_string());
        let scan = run_scan(
            MockGitConfig::default(),
            MockVault { usernames },
            vec![],
            vec![],
            false,
        );
        assert_eq!(scan.credentials.len(), 1);
        assert_eq!(scan.credentials[0].host, "github.com");
        assert_eq!(scan.credentials[0].username, "octocat");
    }

    #[test]
    fn already_onboarded_flag_is_passed_through() {
        let scan = run_scan(
            MockGitConfig::default(),
            MockVault::default(),
            vec![],
            vec![],
            true,
        );
        assert!(scan.already_onboarded);
    }
}
