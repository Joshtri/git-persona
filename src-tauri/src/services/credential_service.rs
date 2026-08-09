use crate::domain::{
    audit::AuditEntry,
    credential::{is_supported_host, Credential, CredentialTxn, Protocol, Secret, VaultSnapshot},
    ports::{AuditSink, CredentialStore, CredentialVault, ProfileCredentialSync, ProfileStore},
};
use crate::error::AppError;
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Bounds for a reveal PIN. A short numeric PIN is the intended case; the upper
/// bound just guards against absurd input. The vault (protected by the OS login)
/// remains the primary boundary — the PIN is a second, deliberate gate.
const PIN_MIN_LEN: usize = 4;
const PIN_MAX_LEN: usize = 64;

fn hash_pin(pin: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Crypto(e.to_string()))
}

fn verify_pin(pin: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash).map_err(|e| AppError::Crypto(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(pin.as_bytes(), &parsed)
        .is_ok())
}

/// The canonical, Git-readable vault target for a host — exactly what the
/// `wincred` / GCM credential helpers look up (`git:https://github.com`).
pub(crate) fn canonical_target(host: &str, protocol: Protocol) -> String {
    format!("git:{}://{}", protocol.scheme(), host)
}

/// A path-scoped canonical target (`git:https://github.com/<owner>/<repo>.git`)
/// for per-repository credentials. Git builds the exact same target when
/// `credential.<scheme>://<host>.useHttpPath` is enabled and the remote has the
/// given path — leading/trailing slashes are dropped to match Git's `path`.
pub(crate) fn path_scoped_target(host: &str, protocol: Protocol, path: &str) -> String {
    let path = path.trim_matches('/');
    format!("git:{}://{}/{path}", protocol.scheme(), host)
}

/// A per-credential backing target where a profile's secret is parked until the
/// profile is applied. Namespaced so it is never mistaken for a Git credential.
fn backing_target(id: Uuid, host: &str, protocol: Protocol) -> String {
    format!("gitpersona:{id}:{}://{}", protocol.scheme(), host)
}

pub(crate) struct CredentialService {
    store: Arc<dyn CredentialStore>,
    vault: Arc<dyn CredentialVault>,
    profiles: Arc<dyn ProfileStore>,
    audit: Arc<dyn AuditSink>,
}

impl CredentialService {
    pub(crate) fn new(
        store: Arc<dyn CredentialStore>,
        vault: Arc<dyn CredentialVault>,
        profiles: Arc<dyn ProfileStore>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            vault,
            profiles,
            audit,
        }
    }

    fn log(
        &self,
        action: &str,
        profile_id: Option<Uuid>,
        host: Option<String>,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path: host,
        })
    }

    pub(crate) fn list(&self) -> Result<Vec<Credential>, AppError> {
        self.store.load_all()
    }

    pub(crate) fn get(&self, id: Uuid) -> Result<Credential, AppError> {
        self.store
            .find(id)?
            .ok_or_else(|| AppError::NotFound(format!("credential {id}")))
    }

    /// Validate that `profile_id` exists and does not already own a credential
    /// for `host` (other than `exclude`). Enforces one credential per host per
    /// profile.
    fn check_unique_host(
        &self,
        profile_id: Uuid,
        host: &str,
        exclude: Option<Uuid>,
    ) -> Result<(), AppError> {
        if self.profiles.find(profile_id)?.is_none() {
            return Err(AppError::NotFound(format!("profile {profile_id}")));
        }
        if let Some(existing) = self.store.find_by_host(profile_id, host)? {
            if Some(existing.id) != exclude {
                return Err(AppError::Conflict(format!(
                    "this profile already has a credential for {host}"
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn create(
        &self,
        profile_id: Option<Uuid>,
        host: &str,
        username: String,
        secret: &Secret,
    ) -> Result<Credential, AppError> {
        let host = host.trim().to_ascii_lowercase();
        if !is_supported_host(&host) {
            return Err(AppError::Validation(format!("unsupported host \"{host}\"")));
        }
        if username.trim().is_empty() {
            return Err(AppError::Validation("username is required".into()));
        }
        if secret.trim().is_empty() {
            return Err(AppError::Validation("token is required".into()));
        }
        if let Some(pid) = profile_id {
            self.check_unique_host(pid, &host, None)?;
        }

        let protocol = Protocol::Https;
        let id = Uuid::new_v4();
        // Park the secret in the OS vault before recording any metadata.
        self.vault
            .store(&backing_target(id, &host, protocol), &username, secret)?;

        let now = Utc::now();
        let credential = Credential {
            id,
            profile_id,
            host: host.clone(),
            protocol,
            username,
            created_at: now,
            updated_at: now,
            last_used: None,
            has_pin: false,
        };
        self.store.save(&credential)?;
        self.log("credential.create", profile_id, Some(host))?;
        Ok(credential)
    }

    /// Set (or replace) the reveal PIN for a credential. Only the Argon2 hash is
    /// persisted; the raw PIN is never stored. Enabling a PIN is what makes a
    /// credential's token readable again via [`reveal`](Self::reveal).
    pub(crate) fn set_pin(&self, id: Uuid, pin: &str) -> Result<Credential, AppError> {
        if pin.len() < PIN_MIN_LEN || pin.len() > PIN_MAX_LEN {
            return Err(AppError::Validation(format!(
                "PIN must be between {PIN_MIN_LEN} and {PIN_MAX_LEN} characters"
            )));
        }
        let mut credential = self.get(id)?;
        let hash = hash_pin(pin)?;
        self.store.save_pin(id, &hash)?;
        credential.has_pin = true;
        credential.updated_at = Utc::now();
        self.store.save(&credential)?;
        self.log(
            "credential.pin_set",
            credential.profile_id,
            Some(credential.host.clone()),
        )?;
        Ok(credential)
    }

    /// Read a credential's token back from the vault after verifying its PIN.
    /// Fails if no PIN is set or the PIN is wrong; the secret is returned in a
    /// zeroizing buffer and never touches the audit log.
    pub(crate) fn reveal(&self, id: Uuid, pin: &str) -> Result<Secret, AppError> {
        let credential = self.get(id)?;
        let hash = self
            .store
            .load_pin(id)?
            .ok_or_else(|| AppError::Validation("no PIN is set for this credential".into()))?;
        if !verify_pin(pin, &hash)? {
            return Err(AppError::Validation("incorrect PIN".into()));
        }
        let backing = backing_target(credential.id, &credential.host, credential.protocol);
        let secret = self
            .vault
            .reveal(&backing)?
            .ok_or_else(|| AppError::Credential("backing secret missing".into()))?;
        self.log(
            "credential.reveal",
            credential.profile_id,
            Some(credential.host),
        )?;
        Ok(secret)
    }

    /// Update the username and optionally rotate the token. A rotated token is
    /// written to the backing slot; the caller must `switch` (or re-apply the
    /// profile) for it to become the active Git credential.
    pub(crate) fn update(
        &self,
        id: Uuid,
        username: String,
        secret: Option<&Secret>,
    ) -> Result<Credential, AppError> {
        if username.trim().is_empty() {
            return Err(AppError::Validation("username is required".into()));
        }
        let mut credential = self.get(id)?;
        if let Some(sec) = secret {
            if sec.trim().is_empty() {
                return Err(AppError::Validation("token is required".into()));
            }
            self.vault.store(
                &backing_target(credential.id, &credential.host, credential.protocol),
                &username,
                sec,
            )?;
        }
        credential.username = username;
        credential.updated_at = Utc::now();
        self.store.save(&credential)?;
        self.log(
            "credential.update",
            credential.profile_id,
            Some(credential.host.clone()),
        )?;
        Ok(credential)
    }

    /// Assign (or clear, with `None`) a credential's owning profile. Enforces
    /// one credential per host per profile.
    pub(crate) fn assign_profile(
        &self,
        id: Uuid,
        profile_id: Option<Uuid>,
    ) -> Result<Credential, AppError> {
        let mut credential = self.get(id)?;
        if let Some(pid) = profile_id {
            self.check_unique_host(pid, &credential.host, Some(id))?;
        }
        credential.profile_id = profile_id;
        credential.updated_at = Utc::now();
        self.store.save(&credential)?;
        self.log(
            "credential.assign",
            profile_id,
            Some(credential.host.clone()),
        )?;
        Ok(credential)
    }

    /// Remove a credential's metadata and its backing secret. Never deletes the
    /// canonical Git credential (that may still be serving another profile).
    pub(crate) fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let credential = self.get(id)?;
        if credential.profile_id.is_some() {
            return Err(AppError::Conflict(
                "unassign this credential from its profile before deleting it".into(),
            ));
        }
        self.vault.delete(&backing_target(
            credential.id,
            &credential.host,
            credential.protocol,
        ))?;
        self.store.delete(id)?;
        self.log("credential.delete", None, Some(credential.host))?;
        Ok(())
    }

    /// Make one credential the active Git credential for its host by promoting
    /// its backing secret onto the canonical target.
    pub(crate) fn switch(&self, id: Uuid) -> Result<Credential, AppError> {
        let mut credential = self.get(id)?;
        self.promote(&credential)?;
        credential.last_used = Some(Utc::now());
        self.store.save(&credential)?;
        self.log(
            "credential.switch",
            credential.profile_id,
            Some(credential.host.clone()),
        )?;
        Ok(credential)
    }

    /// Promote a profile's credential for `host` onto the **path-scoped** target
    /// for that repository's remote path, so git operations in that repository
    /// authenticate as the profile regardless of which profile is globally
    /// active. Returns whether a credential was pinned (`false` when the profile
    /// owns no credential for `host`).
    pub(crate) fn pin_path(
        &self,
        profile_id: Uuid,
        host: &str,
        path: &str,
    ) -> Result<bool, AppError> {
        let Some(credential) = self.store.find_by_host(profile_id, host)? else {
            return Ok(false);
        };
        let target = path_scoped_target(&credential.host, credential.protocol, path);
        self.promote_to(&credential, &target)?;
        let mut used = credential.clone();
        used.last_used = Some(Utc::now());
        self.store.save(&used)?;
        self.log("credential.pin_path", Some(profile_id), Some(target))?;
        Ok(true)
    }

    /// Remove the path-scoped credential for `host`/`path`, returning the
    /// repository to the globally active credential.
    pub(crate) fn unpin_path(&self, host: &str, path: &str) -> Result<(), AppError> {
        let target = path_scoped_target(host, Protocol::Https, path);
        let _ = self.vault.delete(&target)?;
        self.log("credential.unpin_path", None, Some(target))?;
        Ok(())
    }

    /// Promote a single credential's backing secret onto a given target,
    /// returning the target slot's prior contents for rollback.
    fn promote_to(
        &self,
        credential: &Credential,
        target: &str,
    ) -> Result<VaultSnapshot, AppError> {
        let backing = backing_target(credential.id, &credential.host, credential.protocol);
        self.vault.promote(&backing, target, &credential.username)
    }

    /// Promote a single credential's backing secret onto its canonical target,
    /// returning the canonical slot's prior contents for rollback.
    fn promote(&self, credential: &Credential) -> Result<VaultSnapshot, AppError> {
        let canonical = canonical_target(&credential.host, credential.protocol);
        self.promote_to(credential, &canonical)
    }
}

impl ProfileCredentialSync for CredentialService {
    fn switch_profile(&self, profile_id: Uuid) -> Result<CredentialTxn, AppError> {
        let owned: Vec<Credential> = self
            .store
            .load_all()?
            .into_iter()
            .filter(|c| c.profile_id == Some(profile_id))
            .collect();

        let mut snapshots = Vec::new();
        for credential in owned {
            match self.promote(&credential) {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(e) => {
                    // Undo the promotions already done in this pass, then fail.
                    let _ = self.rollback(CredentialTxn { snapshots });
                    return Err(e);
                }
            }
            let mut used = credential.clone();
            used.last_used = Some(Utc::now());
            if let Err(e) = self.store.save(&used).and_then(|()| {
                self.log(
                    "credential.switch",
                    Some(profile_id),
                    Some(credential.host.clone()),
                )
            }) {
                let _ = self.rollback(CredentialTxn { snapshots });
                return Err(e);
            }
        }
        Ok(CredentialTxn { snapshots })
    }

    fn rollback(&self, txn: CredentialTxn) -> Result<(), AppError> {
        let mut first_err = None;
        for snapshot in txn.snapshots.iter().rev() {
            if let Err(e) = self.vault.restore(snapshot) {
                first_err.get_or_insert(e);
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn pin_path(&self, profile_id: Uuid, host: &str, path: &str) -> Result<bool, AppError> {
        Self::pin_path(self, profile_id, host, path)
    }

    fn unpin_path(&self, host: &str, path: &str) -> Result<(), AppError> {
        Self::unpin_path(self, host, path)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;
    use crate::domain::{
        credential::VaultSecret, identity::Identity, profile::Profile, settings::AppSettings,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;
    use zeroize::Zeroizing;

    // ---- mock ports ------------------------------------------------------

    #[derive(Default)]
    struct MockCredStore {
        items: Mutex<Vec<Credential>>,
        pins: Mutex<HashMap<Uuid, String>>,
    }
    impl CredentialStore for MockCredStore {
        fn load_all(&self) -> Result<Vec<Credential>, AppError> {
            Ok(self.items.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<Credential>, AppError> {
            Ok(self
                .items
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id == id)
                .cloned())
        }
        fn find_by_host(
            &self,
            profile_id: Uuid,
            host: &str,
        ) -> Result<Option<Credential>, AppError> {
            Ok(self
                .items
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.profile_id == Some(profile_id) && c.host == host)
                .cloned())
        }
        fn save(&self, credential: &Credential) -> Result<(), AppError> {
            let mut items = self.items.lock().unwrap();
            if let Some(slot) = items.iter_mut().find(|c| c.id == credential.id) {
                *slot = credential.clone();
            } else {
                items.push(credential.clone());
            }
            Ok(())
        }
        fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut items = self.items.lock().unwrap();
            let before = items.len();
            items.retain(|c| c.id != id);
            if items.len() == before {
                return Err(AppError::NotFound(format!("credential {id}")));
            }
            self.pins.lock().unwrap().remove(&id);
            Ok(())
        }
        fn save_pin(&self, id: Uuid, hash: &str) -> Result<(), AppError> {
            self.pins.lock().unwrap().insert(id, hash.to_owned());
            Ok(())
        }
        fn load_pin(&self, id: Uuid) -> Result<Option<String>, AppError> {
            Ok(self.pins.lock().unwrap().get(&id).cloned())
        }
    }

    #[derive(Default)]
    struct MockVault {
        slots: Mutex<HashMap<String, (String, String)>>,
    }
    impl MockVault {
        fn get_raw(&self, target: &str) -> Option<(String, String)> {
            self.slots.lock().unwrap().get(target).cloned()
        }
        fn snapshot_of(&self, target: &str) -> Option<VaultSecret> {
            self.slots
                .lock()
                .unwrap()
                .get(target)
                .map(|(u, s)| VaultSecret {
                    username: u.clone(),
                    secret: Zeroizing::new(s.clone()),
                })
        }
    }
    impl CredentialVault for MockVault {
        fn store(
            &self,
            target: &str,
            username: &str,
            secret: &Secret,
        ) -> Result<VaultSnapshot, AppError> {
            let prior = self.snapshot_of(target);
            self.slots
                .lock()
                .unwrap()
                .insert(target.into(), (username.into(), secret.as_str().into()));
            Ok(VaultSnapshot {
                target: target.into(),
                prior,
            })
        }
        fn username(&self, target: &str) -> Result<Option<String>, AppError> {
            Ok(self
                .slots
                .lock()
                .unwrap()
                .get(target)
                .map(|(u, _)| u.clone()))
        }
        fn reveal(&self, target: &str) -> Result<Option<Secret>, AppError> {
            Ok(self
                .slots
                .lock()
                .unwrap()
                .get(target)
                .map(|(_, s)| Zeroizing::new(s.clone())))
        }
        fn promote(&self, from: &str, to: &str, username: &str) -> Result<VaultSnapshot, AppError> {
            let secret = self
                .slots
                .lock()
                .unwrap()
                .get(from)
                .map(|(_, s)| s.clone())
                .ok_or_else(|| AppError::Credential("backing secret missing".into()))?;
            let prior = self.snapshot_of(to);
            self.slots
                .lock()
                .unwrap()
                .insert(to.into(), (username.into(), secret));
            Ok(VaultSnapshot {
                target: to.into(),
                prior,
            })
        }
        fn delete(&self, target: &str) -> Result<VaultSnapshot, AppError> {
            let prior = self
                .slots
                .lock()
                .unwrap()
                .remove(target)
                .map(|(u, s)| VaultSecret {
                    username: u,
                    secret: Zeroizing::new(s),
                });
            Ok(VaultSnapshot {
                target: target.into(),
                prior,
            })
        }
        fn restore(&self, snapshot: &VaultSnapshot) -> Result<(), AppError> {
            let mut slots = self.slots.lock().unwrap();
            match &snapshot.prior {
                Some(vs) => {
                    slots.insert(
                        snapshot.target.clone(),
                        (vs.username.clone(), vs.secret.as_str().into()),
                    );
                }
                None => {
                    slots.remove(&snapshot.target);
                }
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockProfileStore {
        profiles: Mutex<Vec<Profile>>,
    }
    impl ProfileStore for MockProfileStore {
        fn load_all(&self) -> Result<Vec<Profile>, AppError> {
            Ok(self.profiles.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<Profile>, AppError> {
            Ok(self
                .profiles
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }
        fn save(&self, profile: &Profile) -> Result<(), AppError> {
            self.profiles.lock().unwrap().push(profile.clone());
            Ok(())
        }
        fn delete(&self, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        fn load_settings(&self) -> Result<AppSettings, AppError> {
            Err(AppError::Internal("unused".into()))
        }
        fn save_settings(&self, _settings: &AppSettings) -> Result<(), AppError> {
            Ok(())
        }
        fn get_active_id(&self) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
        fn set_active_id(&self, _id: Option<Uuid>) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockAudit {
        entries: Mutex<Vec<AuditEntry>>,
    }
    impl AuditSink for MockAudit {
        fn append(&self, entry: &AuditEntry) -> Result<(), AppError> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(())
        }
        fn read_all(&self) -> Result<Vec<AuditEntry>, AppError> {
            Ok(self.entries.lock().unwrap().clone())
        }
    }

    // ---- fixtures --------------------------------------------------------

    struct Harness {
        svc: CredentialService,
        store: Arc<MockCredStore>,
        vault: Arc<MockVault>,
        profiles: Arc<MockProfileStore>,
        audit: Arc<MockAudit>,
    }

    fn harness() -> Harness {
        let store = Arc::new(MockCredStore::default());
        let vault = Arc::new(MockVault::default());
        let profiles = Arc::new(MockProfileStore::default());
        let audit = Arc::new(MockAudit::default());
        let svc = CredentialService::new(
            store.clone(),
            vault.clone(),
            profiles.clone(),
            audit.clone(),
        );
        Harness {
            svc,
            store,
            vault,
            profiles,
            audit,
        }
    }

    fn add_profile(h: &Harness) -> Uuid {
        let id = Uuid::new_v4();
        h.profiles.profiles.lock().unwrap().push(Profile {
            id,
            label: "Work".into(),
            identity: Identity {
                name: "N".into(),
                email: "e@x".into(),
                signing_key: None,
            },
            color: None,
        });
        id
    }

    fn secret(s: &str) -> Secret {
        Zeroizing::new(s.to_string())
    }

    fn audit_has(h: &Harness, action: &str) -> bool {
        h.audit
            .read_all()
            .unwrap()
            .iter()
            .any(|e| e.action == action)
    }

    // ---- tests -----------------------------------------------------------

    #[test]
    fn create_stores_metadata_and_parks_secret() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok_123"))
            .unwrap();
        assert_eq!(c.host, "github.com");
        assert_eq!(c.username, "alice");
        // Secret lives in the backing vault slot, never on the metadata record.
        let backing = backing_target(c.id, "github.com", Protocol::Https);
        assert_eq!(h.vault.get_raw(&backing).unwrap().1, "tok_123");
        // Canonical Git target is untouched until a switch/apply.
        assert!(h.vault.get_raw("git:https://github.com").is_none());
        assert!(audit_has(&h, "credential.create"));
    }

    #[test]
    fn credential_never_serializes_secret() {
        let h = harness();
        let c = h
            .svc
            .create(
                None,
                "github.com",
                "alice".into(),
                &secret("super_secret_token"),
            )
            .unwrap();
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("super_secret_token"));
    }

    #[test]
    fn create_normalizes_host() {
        let h = harness();
        let c = h
            .svc
            .create(None, "  GitHub.com  ", "alice".into(), &secret("t"))
            .unwrap();
        assert_eq!(c.host, "github.com");
    }

    #[test]
    fn create_rejects_unsupported_host() {
        let h = harness();
        let err = h
            .svc
            .create(None, "evil.example.com", "alice".into(), &secret("t"))
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn create_rejects_empty_username() {
        let h = harness();
        let err = h
            .svc
            .create(None, "github.com", "   ".into(), &secret("t"))
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn create_rejects_empty_token() {
        let h = harness();
        let err = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("  "))
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn create_rejects_duplicate_host_for_profile() {
        let h = harness();
        let pid = add_profile(&h);
        h.svc
            .create(Some(pid), "github.com", "alice".into(), &secret("t"))
            .unwrap();
        let err = h
            .svc
            .create(Some(pid), "github.com", "bob".into(), &secret("t"))
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn create_same_host_different_profiles_is_allowed() {
        let h = harness();
        let p1 = add_profile(&h);
        let p2 = add_profile(&h);
        h.svc
            .create(Some(p1), "github.com", "alice".into(), &secret("t"))
            .unwrap();
        assert!(h
            .svc
            .create(Some(p2), "github.com", "bob".into(), &secret("t"))
            .is_ok());
    }

    #[test]
    fn assign_unknown_profile_errors() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("t"))
            .unwrap();
        let err = h
            .svc
            .assign_profile(c.id, Some(Uuid::new_v4()))
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn assign_rejects_duplicate_host_for_profile() {
        let h = harness();
        let pid = add_profile(&h);
        h.svc
            .create(Some(pid), "github.com", "alice".into(), &secret("t"))
            .unwrap();
        let other = h
            .svc
            .create(None, "github.com", "bob".into(), &secret("t"))
            .unwrap();
        let err = h.svc.assign_profile(other.id, Some(pid)).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn delete_refuses_when_assigned() {
        let h = harness();
        let pid = add_profile(&h);
        let c = h
            .svc
            .create(Some(pid), "github.com", "alice".into(), &secret("t"))
            .unwrap();
        let err = h.svc.delete(c.id).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn delete_removes_backing_secret() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("t"))
            .unwrap();
        let backing = backing_target(c.id, "github.com", Protocol::Https);
        h.svc.delete(c.id).unwrap();
        assert!(h.vault.get_raw(&backing).is_none());
        assert!(h.store.load_all().unwrap().is_empty());
        assert!(audit_has(&h, "credential.delete"));
    }

    #[test]
    fn switch_promotes_backing_to_canonical() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok_switch"))
            .unwrap();
        h.svc.switch(c.id).unwrap();
        let canonical = h.vault.get_raw("git:https://github.com").unwrap();
        assert_eq!(canonical.0, "alice");
        assert_eq!(canonical.1, "tok_switch");
        assert!(h.svc.get(c.id).unwrap().last_used.is_some());
        assert!(audit_has(&h, "credential.switch"));
    }

    #[test]
    fn update_rotates_token_in_backing() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("old"))
            .unwrap();
        h.svc
            .update(c.id, "alice2".into(), Some(&secret("new")))
            .unwrap();
        let backing = backing_target(c.id, "github.com", Protocol::Https);
        let raw = h.vault.get_raw(&backing).unwrap();
        assert_eq!(raw.0, "alice2");
        assert_eq!(raw.1, "new");
        assert_eq!(h.svc.get(c.id).unwrap().username, "alice2");
    }

    #[test]
    fn switch_profile_promotes_all_owned_credentials() {
        let h = harness();
        let pid = add_profile(&h);
        h.svc
            .create(Some(pid), "github.com", "alice".into(), &secret("gh"))
            .unwrap();
        h.svc
            .create(Some(pid), "gitlab.com", "alice".into(), &secret("gl"))
            .unwrap();
        let txn = h.svc.switch_profile(pid).unwrap();
        assert_eq!(txn.snapshots.len(), 2);
        assert_eq!(h.vault.get_raw("git:https://github.com").unwrap().1, "gh");
        assert_eq!(h.vault.get_raw("git:https://gitlab.com").unwrap().1, "gl");
    }

    #[test]
    fn switch_profile_with_no_credentials_is_empty_ok() {
        let h = harness();
        let pid = add_profile(&h);
        let txn = h.svc.switch_profile(pid).unwrap();
        assert!(txn.snapshots.is_empty());
    }

    #[test]
    fn rollback_restores_prior_canonical_credential() {
        let h = harness();
        // Profile A active on github.com.
        let pa = add_profile(&h);
        h.svc
            .create(Some(pa), "github.com", "alice".into(), &secret("A_tok"))
            .unwrap();
        h.svc.switch_profile(pa).unwrap();
        assert_eq!(
            h.vault.get_raw("git:https://github.com").unwrap().1,
            "A_tok"
        );

        // Switch to profile B, then roll back — canonical must return to A.
        let pb = add_profile(&h);
        h.svc
            .create(Some(pb), "github.com", "bob".into(), &secret("B_tok"))
            .unwrap();
        let txn = h.svc.switch_profile(pb).unwrap();
        assert_eq!(
            h.vault.get_raw("git:https://github.com").unwrap().1,
            "B_tok"
        );

        h.svc.rollback(txn).unwrap();
        let restored = h.vault.get_raw("git:https://github.com").unwrap();
        assert_eq!(restored.0, "alice");
        assert_eq!(restored.1, "A_tok");
    }

    #[test]
    fn set_pin_enables_reveal_of_stored_token() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("ghp_reveal_me"))
            .unwrap();
        assert!(!c.has_pin);
        let updated = h.svc.set_pin(c.id, "1234").unwrap();
        assert!(updated.has_pin);
        let revealed = h.svc.reveal(c.id, "1234").unwrap();
        assert_eq!(revealed.as_str(), "ghp_reveal_me");
        assert!(audit_has(&h, "credential.reveal"));
    }

    #[test]
    fn reveal_rejects_wrong_pin() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok"))
            .unwrap();
        h.svc.set_pin(c.id, "1234").unwrap();
        let err = h.svc.reveal(c.id, "9999").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn reveal_without_pin_set_errors() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok"))
            .unwrap();
        let err = h.svc.reveal(c.id, "1234").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn set_pin_rejects_too_short() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok"))
            .unwrap();
        let err = h.svc.set_pin(c.id, "12").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn delete_removes_pin() {
        let h = harness();
        let c = h
            .svc
            .create(None, "github.com", "alice".into(), &secret("tok"))
            .unwrap();
        h.svc.set_pin(c.id, "1234").unwrap();
        h.svc.delete(c.id).unwrap();
        assert!(h.store.load_pin(c.id).unwrap().is_none());
    }

    #[test]
    fn get_missing_errors() {
        let h = harness();
        let err = h.svc.get(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn pin_path_promotes_backing_to_path_scoped_target() {
        let h = harness();
        let pid = add_profile(&h);
        h.svc
            .create(Some(pid), "github.com", "joshtri".into(), &secret("ghp_pin"))
            .unwrap();

        h.svc.pin_path(pid, "github.com", "Joshtri/landing-page-dolphin-laundry.git")
            .unwrap();

        let scoped = h
            .vault
            .get_raw("git:https://github.com/Joshtri/landing-page-dolphin-laundry.git")
            .unwrap();
        assert_eq!(scoped.0, "joshtri");
        assert_eq!(scoped.1, "ghp_pin");
        // The host-level canonical slot is left untouched by a path pin.
        assert!(h.vault.get_raw("git:https://github.com").is_none());
        assert!(audit_has(&h, "credential.pin_path"));
    }

    #[test]
    fn pin_path_is_noop_without_profile_credential() {
        let h = harness();
        let pid = add_profile(&h);
        assert!(!h
            .svc
            .pin_path(pid, "github.com", "Joshtri/repo.git")
            .unwrap());
        assert!(h
            .vault
            .get_raw("git:https://github.com/Joshtri/repo.git")
            .is_none());
    }

    #[test]
    fn unpin_path_removes_scoped_target() {
        let h = harness();
        let pid = add_profile(&h);
        h.svc
            .create(Some(pid), "github.com", "joshtri".into(), &secret("t"))
            .unwrap();
        h.svc
            .pin_path(pid, "github.com", "Joshtri/repo.git")
            .unwrap();
        assert!(h
            .vault
            .get_raw("git:https://github.com/Joshtri/repo.git")
            .is_some());

        h.svc.unpin_path("github.com", "Joshtri/repo.git").unwrap();
        assert!(h
            .vault
            .get_raw("git:https://github.com/Joshtri/repo.git")
            .is_none());
    }

    #[test]
    fn parse_https_remote_extracts_host_and_path() {
        use crate::domain::credential::parse_https_remote;
        let (host, path) =
            parse_https_remote("https://github.com/Joshtri/landing-page-dolphin-laundry.git")
                .unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(path, "Joshtri/landing-page-dolphin-laundry.git");
    }

    #[test]
    fn parse_https_remote_strips_trailing_slash_and_userinfo() {
        use crate::domain::credential::parse_https_remote;
        let (host, path) = parse_https_remote(
            "https://joshtri@GitHub.com/Joshtri/landing-page-dolphin-laundry.git/",
        )
        .unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(path, "Joshtri/landing-page-dolphin-laundry.git");
    }

    #[test]
    fn parse_https_remote_rejects_ssh_and_host_only() {
        use crate::domain::credential::parse_https_remote;
        assert!(parse_https_remote("git@github.com:Joshtri/repo.git").is_none());
        assert!(parse_https_remote("https://github.com").is_none());
        assert!(parse_https_remote("file:///tmp/repo").is_none());
    }
}
