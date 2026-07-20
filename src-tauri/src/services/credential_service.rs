use crate::domain::{
    audit::AuditEntry,
    credential::{is_supported_host, Credential, CredentialTxn, Protocol, Secret, VaultSnapshot},
    ports::{AuditSink, CredentialStore, CredentialVault, ProfileCredentialSync, ProfileStore},
};
use crate::error::AppError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// The canonical, Git-readable vault target for a host — exactly what the
/// `wincred` / GCM credential helpers look up (`git:https://github.com`).
fn canonical_target(host: &str, protocol: Protocol) -> String {
    format!("git:{}://{}", protocol.scheme(), host)
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
        };
        self.store.save(&credential)?;
        self.log("credential.create", profile_id, Some(host))?;
        Ok(credential)
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

    /// Promote a single credential's backing secret onto its canonical target,
    /// returning the canonical slot's prior contents for rollback.
    fn promote(&self, credential: &Credential) -> Result<VaultSnapshot, AppError> {
        let backing = backing_target(credential.id, &credential.host, credential.protocol);
        let canonical = canonical_target(&credential.host, credential.protocol);
        self.vault
            .promote(&backing, &canonical, &credential.username)
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
            Ok(())
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
    fn get_missing_errors() {
        let h = harness();
        let err = h.svc.get(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
