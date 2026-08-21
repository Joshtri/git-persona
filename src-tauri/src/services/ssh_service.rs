use crate::domain::{
    audit::AuditEntry,
    ports::{
        AuditSink, ProfileStore, SshConfigWriter, SshKeyGenerator, SshKeyReader, SshScanner,
        SshStore,
    },
    ssh::{GenerateParams, SshConfigEntry, SshKey},
};
use crate::error::AppError;
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_HOST_NAME: &str = "github.com";
const DEFAULT_USER: &str = "git";

pub(crate) struct SshService {
    store: Arc<dyn SshStore>,
    scanner: Arc<dyn SshScanner>,
    reader: Arc<dyn SshKeyReader>,
    generator: Arc<dyn SshKeyGenerator>,
    config: Arc<dyn SshConfigWriter>,
    profiles: Arc<dyn ProfileStore>,
    audit: Arc<dyn AuditSink>,
}

fn label_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| path.to_string())
}

/// Slugify a profile label into a safe `Host` alias: lowercase alphanumerics,
/// runs of anything else collapse to a single `-`.
fn slug(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !out.is_empty() && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-');
    if trimmed.is_empty() {
        "profile".to_string()
    } else {
        trimmed.to_string()
    }
}

impl SshService {
    pub(crate) fn new(
        store: Arc<dyn SshStore>,
        scanner: Arc<dyn SshScanner>,
        reader: Arc<dyn SshKeyReader>,
        generator: Arc<dyn SshKeyGenerator>,
        config: Arc<dyn SshConfigWriter>,
        profiles: Arc<dyn ProfileStore>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            scanner,
            reader,
            generator,
            config,
            profiles,
            audit,
        }
    }

    fn log(
        &self,
        action: &str,
        profile_id: Option<Uuid>,
        target: Option<String>,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path: target,
        })
    }

    pub(crate) fn list(&self) -> Result<Vec<SshKey>, AppError> {
        self.store.load_all()
    }

    pub(crate) fn get(&self, id: Uuid) -> Result<SshKey, AppError> {
        self.store
            .find(id)?
            .ok_or_else(|| AppError::NotFound(format!("ssh key {id}")))
    }

    /// Discover keys in `ssh_dir`, adding any not already tracked. Unsupported,
    /// unparseable, or insecure keys are silently skipped.
    pub(crate) fn scan(&self, ssh_dir: &Path) -> Result<Vec<SshKey>, AppError> {
        for path in self.scanner.scan(ssh_dir)? {
            if self.store.find_by_path(&path)?.is_some() {
                continue;
            }
            let Ok(meta) = self.reader.read(Path::new(&path)) else {
                continue;
            };
            let key = SshKey {
                id: Uuid::new_v4(),
                label: label_from_path(&path),
                algorithm: meta.algorithm,
                fingerprint: meta.fingerprint,
                private_key_path: path.clone(),
                public_key_path: meta.public_key_path,
                comment: meta.comment,
                assigned_profile_id: None,
                host_alias: None,
                host_name: None,
                created_at: Utc::now(),
                imported: false,
                last_used: None,
            };
            self.store.save(&key)?;
        }
        self.store.load_all()
    }

    pub(crate) fn import(
        &self,
        private_key_path: String,
        host_alias: Option<String>,
        host_name: Option<String>,
    ) -> Result<SshKey, AppError> {
        if self.store.find_by_path(&private_key_path)?.is_some() {
            return Err(AppError::Conflict("this key is already imported".into()));
        }
        let meta = self.reader.read(Path::new(&private_key_path))?;
        let key = SshKey {
            id: Uuid::new_v4(),
            label: label_from_path(&private_key_path),
            algorithm: meta.algorithm,
            fingerprint: meta.fingerprint,
            private_key_path: private_key_path.clone(),
            public_key_path: meta.public_key_path,
            comment: meta.comment,
            assigned_profile_id: None,
            host_alias,
            host_name,
            created_at: Utc::now(),
            imported: true,
            last_used: None,
        };
        self.store.save(&key)?;
        self.log("ssh.import", None, Some(private_key_path))?;
        Ok(key)
    }

    pub(crate) fn generate(&self, params: GenerateParams) -> Result<SshKey, AppError> {
        if params.label.trim().is_empty() {
            return Err(AppError::Validation("label is required".into()));
        }
        if params.file_name.trim().is_empty() {
            return Err(AppError::Validation("file name is required".into()));
        }
        let generated = self.generator.generate(
            params.algorithm,
            Path::new(&params.output_dir),
            &params.file_name,
            params.comment.as_deref(),
        )?;
        let key = SshKey {
            id: Uuid::new_v4(),
            label: params.label,
            algorithm: params.algorithm,
            fingerprint: generated.fingerprint,
            private_key_path: generated.private_key_path.clone(),
            public_key_path: Some(generated.public_key_path),
            comment: generated.comment,
            assigned_profile_id: None,
            host_alias: params.host_alias,
            host_name: params.host_name,
            created_at: Utc::now(),
            imported: false,
            last_used: None,
        };
        self.store.save(&key)?;
        self.log("ssh.generate", None, Some(generated.private_key_path))?;
        Ok(key)
    }

    /// Assign (or clear, with `None`) a key's profile. Enforces the 1:1 rule by
    /// clearing any other key currently pointing at the same profile.
    pub(crate) fn assign_profile(
        &self,
        id: Uuid,
        profile_id: Option<Uuid>,
    ) -> Result<SshKey, AppError> {
        let mut key = self.get(id)?;
        if let Some(pid) = profile_id {
            if self.profiles.find(pid)?.is_none() {
                return Err(AppError::NotFound(format!("profile {pid}")));
            }
            for mut other in self.store.load_all()? {
                if other.id != id && other.assigned_profile_id == Some(pid) {
                    other.assigned_profile_id = None;
                    self.store.save(&other)?;
                }
            }
        }
        key.assigned_profile_id = profile_id;
        self.store.save(&key)?;
        self.regenerate_config()?;
        self.log("ssh.assign", profile_id, Some(key.private_key_path.clone()))?;
        Ok(key)
    }

    /// Remove the *reference* to a key. Never deletes the key file from disk.
    pub(crate) fn remove(&self, id: Uuid) -> Result<(), AppError> {
        let key = self.get(id)?;
        if key.assigned_profile_id.is_some() {
            return Err(AppError::Conflict(
                "unassign this key from its profile before removing it".into(),
            ));
        }
        self.store.delete(id)?;
        self.regenerate_config()?;
        self.log("ssh.remove", None, Some(key.private_key_path))?;
        Ok(())
    }

    /// Record that a key was used and return it (the command reveals its path).
    pub(crate) fn mark_used(&self, id: Uuid) -> Result<SshKey, AppError> {
        let mut key = self.get(id)?;
        key.last_used = Some(Utc::now());
        self.store.save(&key)?;
        Ok(key)
    }

    pub(crate) fn config_preview(&self) -> Result<String, AppError> {
        self.config.read_raw()
    }

    /// Rebuild the managed `~/.ssh/config` block from all assigned keys.
    fn regenerate_config(&self) -> Result<(), AppError> {
        let mut entries: Vec<SshConfigEntry> = Vec::new();
        for key in self.store.load_all()? {
            let Some(pid) = key.assigned_profile_id else {
                continue;
            };
            let host_alias = match key.host_alias.as_ref() {
                Some(a) if !a.trim().is_empty() => a.clone(),
                _ => {
                    let label = self
                        .profiles
                        .find(pid)?
                        .map_or_else(|| key.label.clone(), |p| p.label);
                    slug(&label)
                }
            };
            let host_name = key
                .host_name
                .as_ref()
                .filter(|h| !h.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| DEFAULT_HOST_NAME.to_string());
            entries.push(SshConfigEntry {
                host_alias,
                host_name,
                user: DEFAULT_USER.to_string(),
                identity_file: key.private_key_path.clone(),
            });
        }
        self.config.write_managed_block(&entries)?;
        self.log("ssh.config.updated", None, None)
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
        identity::Identity,
        profile::Profile,
        settings::AppSettings,
        ssh::{GeneratedKey, SshAlgorithm, SshConfigEntry, SshKeyMetadata},
    };
    use std::sync::Mutex;

    // ---- mock ports ------------------------------------------------------

    #[derive(Default)]
    struct MockSshStore {
        keys: Mutex<Vec<SshKey>>,
    }
    impl SshStore for MockSshStore {
        fn load_all(&self) -> Result<Vec<SshKey>, AppError> {
            Ok(self.keys.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<SshKey>, AppError> {
            Ok(self
                .keys
                .lock()
                .unwrap()
                .iter()
                .find(|k| k.id == id)
                .cloned())
        }
        fn find_by_path(&self, path: &str) -> Result<Option<SshKey>, AppError> {
            Ok(self
                .keys
                .lock()
                .unwrap()
                .iter()
                .find(|k| k.private_key_path == path)
                .cloned())
        }
        fn save(&self, key: &SshKey) -> Result<(), AppError> {
            let mut ks = self.keys.lock().unwrap();
            if let Some(slot) = ks.iter_mut().find(|k| k.id == key.id) {
                *slot = key.clone();
            } else {
                ks.push(key.clone());
            }
            Ok(())
        }
        fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut ks = self.keys.lock().unwrap();
            let before = ks.len();
            ks.retain(|k| k.id != id);
            if ks.len() == before {
                return Err(AppError::NotFound(format!("ssh key {id}")));
            }
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

    /// Returns metadata for known paths; errors for anything else (unparseable).
    struct MockReader {
        table: Vec<(String, SshKeyMetadata)>,
    }
    impl SshKeyReader for MockReader {
        fn read(&self, path: &Path) -> Result<SshKeyMetadata, AppError> {
            let key = path.to_string_lossy().into_owned();
            self.table
                .iter()
                .find(|(p, _)| *p == key)
                .map(|(_, m)| m.clone())
                .ok_or_else(|| AppError::Crypto("unparseable".into()))
        }
    }

    struct MockGenerator;
    impl SshKeyGenerator for MockGenerator {
        fn generate(
            &self,
            _algorithm: SshAlgorithm,
            output_dir: &Path,
            file_name: &str,
            comment: Option<&str>,
        ) -> Result<GeneratedKey, AppError> {
            let priv_path = output_dir.join(file_name).to_string_lossy().into_owned();
            Ok(GeneratedKey {
                private_key_path: priv_path.clone(),
                public_key_path: format!("{priv_path}.pub"),
                fingerprint: "SHA256:generated".into(),
                comment: comment.map(str::to_string),
            })
        }
    }

    #[derive(Default)]
    struct MockConfig {
        entries: Mutex<Vec<SshConfigEntry>>,
        writes: Mutex<u32>,
    }
    impl SshConfigWriter for MockConfig {
        fn write_managed_block(&self, entries: &[SshConfigEntry]) -> Result<(), AppError> {
            *self.entries.lock().unwrap() = entries.to_vec();
            *self.writes.lock().unwrap() += 1;
            Ok(())
        }
        fn read_raw(&self) -> Result<String, AppError> {
            Ok(String::new())
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
        fn purge_before(&self, cutoff: chrono::DateTime<Utc>) -> Result<u32, AppError> {
            let mut entries = self.entries.lock().unwrap();
            let before = entries.len();
            entries.retain(|e| e.timestamp >= cutoff);
            Ok(u32::try_from(before - entries.len()).unwrap_or(u32::MAX))
        }
    }

    // ---- fixtures --------------------------------------------------------

    struct Harness {
        svc: SshService,
        store: Arc<MockSshStore>,
        profiles: Arc<MockProfileStore>,
        config: Arc<MockConfig>,
        audit: Arc<MockAudit>,
    }

    fn meta() -> SshKeyMetadata {
        SshKeyMetadata {
            algorithm: SshAlgorithm::Ed25519,
            fingerprint: "SHA256:abc".into(),
            comment: Some("me@host".into()),
            public_key_path: None,
        }
    }

    fn harness(found: Vec<&str>) -> Harness {
        let store = Arc::new(MockSshStore::default());
        let profiles = Arc::new(MockProfileStore::default());
        let config = Arc::new(MockConfig::default());
        let audit = Arc::new(MockAudit::default());
        let table = found.iter().map(|p| ((*p).to_string(), meta())).collect();
        let svc = SshService::new(
            store.clone(),
            Arc::new(MockScanner {
                found: found.into_iter().map(String::from).collect(),
            }),
            Arc::new(MockReader { table }),
            Arc::new(MockGenerator),
            config.clone(),
            profiles.clone(),
            audit.clone(),
        );
        Harness {
            svc,
            store,
            profiles,
            config,
            audit,
        }
    }

    fn add_profile(h: &Harness, label: &str) -> Uuid {
        let id = Uuid::new_v4();
        h.profiles.profiles.lock().unwrap().push(Profile {
            id,
            label: label.into(),
            identity: Identity {
                name: "N".into(),
                email: "e@x".into(),
                signing_key: None,
            },
            color: None,
        });
        id
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
    fn scan_discovers_and_is_idempotent() {
        let h = harness(vec!["/ssh/id_ed25519", "/ssh/id_rsa"]);
        let keys = h.svc.scan(Path::new("/ssh")).unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|k| !k.imported));
        // second scan adds nothing
        let again = h.svc.scan(Path::new("/ssh")).unwrap();
        assert_eq!(again.len(), 2);
    }

    #[test]
    fn scan_skips_unparseable() {
        let h = harness(vec!["/ssh/id_ed25519"]);
        // inject an extra candidate the reader doesn't know
        let svc = SshService::new(
            h.store.clone(),
            Arc::new(MockScanner {
                found: vec!["/ssh/id_ed25519".into(), "/ssh/garbage".into()],
            }),
            Arc::new(MockReader {
                table: vec![("/ssh/id_ed25519".to_string(), meta())],
            }),
            Arc::new(MockGenerator),
            h.config.clone(),
            h.profiles.clone(),
            h.audit.clone(),
        );
        let keys = svc.scan(Path::new("/ssh")).unwrap();
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn import_adds_and_audits() {
        let h = harness(vec!["/ssh/id_ed25519"]);
        let key = h.svc.import("/ssh/id_ed25519".into(), None, None).unwrap();
        assert!(key.imported);
        assert_eq!(key.fingerprint, "SHA256:abc");
        assert!(audit_has(&h, "ssh.import"));
    }

    #[test]
    fn import_rejects_duplicate() {
        let h = harness(vec!["/ssh/id_ed25519"]);
        h.svc.import("/ssh/id_ed25519".into(), None, None).unwrap();
        let err = h
            .svc
            .import("/ssh/id_ed25519".into(), None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn import_rejects_unparseable_path() {
        let h = harness(vec![]);
        let err = h.svc.import("/ssh/missing".into(), None, None).unwrap_err();
        assert!(matches!(err, AppError::Crypto(_)));
    }

    #[test]
    fn generate_creates_and_audits() {
        let h = harness(vec![]);
        let params = GenerateParams {
            label: "Work".into(),
            algorithm: SshAlgorithm::Ed25519,
            comment: Some("me@host".into()),
            output_dir: "/keys".into(),
            file_name: "work_ed25519".into(),
            host_alias: None,
            host_name: None,
        };
        let key = h.svc.generate(params).unwrap();
        assert_eq!(key.label, "Work");
        assert!(key.private_key_path.contains("work_ed25519"));
        assert!(key.public_key_path.is_some());
        assert!(audit_has(&h, "ssh.generate"));
    }

    #[test]
    fn assign_enforces_one_to_one_and_writes_config() {
        let h = harness(vec!["/ssh/a", "/ssh/b"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        let b = h.svc.import("/ssh/b".into(), None, None).unwrap();
        let pid = add_profile(&h, "Work");

        h.svc.assign_profile(a.id, Some(pid)).unwrap();
        h.svc.assign_profile(b.id, Some(pid)).unwrap();

        let keys = h.store.load_all().unwrap();
        let a_now = keys.iter().find(|k| k.id == a.id).unwrap();
        let b_now = keys.iter().find(|k| k.id == b.id).unwrap();
        assert_eq!(a_now.assigned_profile_id, None); // reassigned away
        assert_eq!(b_now.assigned_profile_id, Some(pid));

        // config now has exactly one entry, alias slugged from label
        let entries = h.config.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host_alias, "work");
        assert_eq!(entries[0].host_name, "github.com");
        assert!(audit_has(&h, "ssh.assign"));
        assert!(audit_has(&h, "ssh.config.updated"));
    }

    #[test]
    fn assign_uses_custom_host_fields() {
        let h = harness(vec!["/ssh/a"]);
        let a = h
            .svc
            .import(
                "/ssh/a".into(),
                Some("github-work".into()),
                Some("git.acme.io".into()),
            )
            .unwrap();
        let pid = add_profile(&h, "Work");
        h.svc.assign_profile(a.id, Some(pid)).unwrap();
        let entries = h.config.entries.lock().unwrap();
        assert_eq!(entries[0].host_alias, "github-work");
        assert_eq!(entries[0].host_name, "git.acme.io");
    }

    #[test]
    fn assign_unknown_profile_errors() {
        let h = harness(vec!["/ssh/a"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        let err = h
            .svc
            .assign_profile(a.id, Some(Uuid::new_v4()))
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn unassign_clears_config_entry() {
        let h = harness(vec!["/ssh/a"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        let pid = add_profile(&h, "Work");
        h.svc.assign_profile(a.id, Some(pid)).unwrap();
        h.svc.assign_profile(a.id, None).unwrap();
        assert!(h.config.entries.lock().unwrap().is_empty());
    }

    #[test]
    fn remove_refuses_when_assigned() {
        let h = harness(vec!["/ssh/a"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        let pid = add_profile(&h, "Work");
        h.svc.assign_profile(a.id, Some(pid)).unwrap();
        let err = h.svc.remove(a.id).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[test]
    fn remove_deletes_unassigned_reference() {
        let h = harness(vec!["/ssh/a"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        h.svc.remove(a.id).unwrap();
        assert!(h.store.load_all().unwrap().is_empty());
        assert!(audit_has(&h, "ssh.remove"));
    }

    #[test]
    fn mark_used_sets_last_used() {
        let h = harness(vec!["/ssh/a"]);
        let a = h.svc.import("/ssh/a".into(), None, None).unwrap();
        assert!(a.last_used.is_none());
        let used = h.svc.mark_used(a.id).unwrap();
        assert!(used.last_used.is_some());
    }

    #[test]
    fn get_missing_errors() {
        let h = harness(vec![]);
        let err = h.svc.get(Uuid::new_v4()).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
