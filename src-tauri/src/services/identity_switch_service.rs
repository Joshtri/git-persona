use crate::domain::{
    audit::AuditEntry,
    identity_switch::{ResolvedRepo, SmartSwitchStatus, SwitchEvent, SwitchStatus},
    ports::{
        AuditSink, IdentityResolver, IdentitySwitcher, ProfileStore, RepoStore, RuleResolver,
        SwitchObserver, WorkspaceWatcher,
    },
    rule::EvaluationContext,
    settings::SmartSwitchingSettings,
};
use crate::error::AppError;
use crate::services::profile_service::ProfileService;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

fn locked<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, AppError> {
    m.lock()
        .map_err(|_| AppError::Internal("smart-switch lock poisoned".into()))
}

// ---- adapters ------------------------------------------------------------

/// [`IdentityResolver`] backed by the repository store.
pub(crate) struct RepoIdentityResolver {
    repos: Arc<dyn RepoStore>,
}

impl RepoIdentityResolver {
    pub(crate) fn new(repos: Arc<dyn RepoStore>) -> Self {
        Self { repos }
    }
}

impl IdentityResolver for RepoIdentityResolver {
    fn resolve(&self, git_root: &str) -> Result<Option<ResolvedRepo>, AppError> {
        Ok(self
            .repos
            .find_by_git_root(git_root)?
            .map(|r| ResolvedRepo {
                repo_id: r.id,
                repo_name: r.name,
                profile_id: r.active_profile_id,
                remote_origin: r.remote_origin,
            }))
    }

    fn watchable_roots(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .repos
            .load_all()?
            .into_iter()
            .map(|r| r.git_root)
            .collect())
    }
}

/// [`IdentitySwitcher`] that delegates to the existing profile-apply pipeline.
/// This is the sole bridge from the orchestrator into identity mutation — the
/// orchestrator never edits Git config or the credential vault itself.
pub(crate) struct ProfileIdentitySwitcher {
    profiles: Arc<ProfileService>,
    store: Arc<dyn ProfileStore>,
}

impl ProfileIdentitySwitcher {
    pub(crate) fn new(profiles: Arc<ProfileService>, store: Arc<dyn ProfileStore>) -> Self {
        Self { profiles, store }
    }
}

impl IdentitySwitcher for ProfileIdentitySwitcher {
    fn active_profile_id(&self) -> Result<Option<Uuid>, AppError> {
        self.store.get_active_id()
    }

    fn profile_label(&self, id: Uuid) -> Result<Option<String>, AppError> {
        Ok(self.store.find(id)?.map(|p| p.label))
    }

    fn apply(&self, profile_id: Uuid) -> Result<(), AppError> {
        self.profiles.apply(profile_id)
    }

    fn apply_local(&self, git_root: &str, profile_id: Uuid) -> Result<(), AppError> {
        self.profiles
            .apply_local(std::path::Path::new(git_root), profile_id)
    }
}

// ---- orchestrator --------------------------------------------------------

/// The Smart Identity Switching orchestrator. It owns no I/O of its own: it
/// coordinates the watcher, the resolver, and the existing profile-apply
/// pipeline, records activity, and reports outcomes to the UI.
pub(crate) struct IdentitySwitchService {
    resolver: Arc<dyn IdentityResolver>,
    switcher: Arc<dyn IdentitySwitcher>,
    watcher: Arc<dyn WorkspaceWatcher>,
    observer: Arc<dyn SwitchObserver>,
    settings: Arc<dyn ProfileStore>,
    rules: Arc<dyn RuleResolver>,
    audit: Arc<dyn AuditSink>,
    started_at: Mutex<Option<DateTime<Utc>>>,
    last_switch: Mutex<Option<SwitchEvent>>,
}

impl IdentitySwitchService {
    pub(crate) fn new(
        resolver: Arc<dyn IdentityResolver>,
        switcher: Arc<dyn IdentitySwitcher>,
        watcher: Arc<dyn WorkspaceWatcher>,
        observer: Arc<dyn SwitchObserver>,
        settings: Arc<dyn ProfileStore>,
        rules: Arc<dyn RuleResolver>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            resolver,
            switcher,
            watcher,
            observer,
            settings,
            rules,
            audit,
            started_at: Mutex::new(None),
            last_switch: Mutex::new(None),
        }
    }

    /// Resolve the profile a repository should use: the Rule Engine decides
    /// first (highest-priority match wins), falling back to the manual
    /// repository assignment when no rule matches. Records `rule.matched` when a
    /// rule drives the decision. Returns `None` only when neither a rule nor a
    /// manual assignment applies.
    fn resolve_profile(
        &self,
        resolved: &ResolvedRepo,
        git_root: &str,
    ) -> Result<Option<Uuid>, AppError> {
        let ctx = EvaluationContext::from_parts(
            git_root,
            &resolved.repo_name,
            resolved.remote_origin.as_deref(),
        );
        if let Some(m) = self.rules.resolve(&ctx)? {
            self.record("rule.matched", Some(m.profile_id), git_root)?;
            return Ok(Some(m.profile_id));
        }
        Ok(resolved.profile_id)
    }

    fn config(&self) -> Result<SmartSwitchingSettings, AppError> {
        Ok(self.settings.load_settings()?.smart_switching)
    }

    /// Bring the watcher in line with the persisted settings: watching every
    /// tracked repository when enabled, stopped otherwise. Safe to call
    /// repeatedly — used on launch, on settings change, and on restart.
    pub(crate) fn reconcile(self: &Arc<Self>) -> Result<(), AppError> {
        if self.config()?.enabled {
            let roots = self.resolver.watchable_roots()?;
            let weak = Arc::downgrade(self);
            let on_activity: Arc<dyn Fn(String) + Send + Sync> = Arc::new(move |git_root| {
                if let Some(this) = weak.upgrade() {
                    this.handle_activity(&git_root);
                }
            });
            self.watcher.start(&roots, on_activity)?;
            // Pin every tracked repo up front so the correct author is already in
            // local config before the first commit — not applied a commit late.
            self.pin_all();
            let mut started = locked(&self.started_at)?;
            if started.is_none() {
                *started = Some(Utc::now());
            }
        } else {
            self.watcher.stop();
            *locked(&self.started_at)? = None;
        }
        self.emit_status();
        Ok(())
    }

    /// Rebuild the watch set from the current repository list (e.g. after a scan).
    pub(crate) fn restart(self: &Arc<Self>) -> Result<(), AppError> {
        self.reconcile()
    }

    /// Persist the master toggle and reconcile the watcher.
    pub(crate) fn set_enabled(self: &Arc<Self>, enabled: bool) -> Result<(), AppError> {
        let mut settings = self.settings.load_settings()?;
        settings.smart_switching.enabled = enabled;
        self.settings.save_settings(&settings)?;
        self.reconcile()
    }

    /// Start watching on launch when the user opted in and the feature is on.
    pub(crate) fn maybe_start_on_launch(self: &Arc<Self>) -> Result<(), AppError> {
        if self.config()?.start_on_launch {
            self.reconcile()?;
        }
        Ok(())
    }

    pub(crate) fn pause(&self) {
        self.watcher.pause();
        self.emit_status();
    }

    pub(crate) fn resume(&self) {
        self.watcher.resume();
        self.emit_status();
    }

    pub(crate) fn status(&self) -> Result<SmartSwitchStatus, AppError> {
        let settings = self.config()?;
        Ok(SmartSwitchStatus {
            enabled: settings.enabled,
            watching: self.watcher.is_watching(),
            paused: self.watcher.is_paused(),
            repos_monitored: u32::try_from(self.watcher.watched_count()).unwrap_or(u32::MAX),
            last_switch: locked(&self.last_switch)?.clone(),
            started_at: *locked(&self.started_at)?,
        })
    }

    /// Apply the pending switch for a repository (used by the confirm-before-switch
    /// flow). Re-resolves so a stale assignment can't be applied.
    pub(crate) fn confirm(&self, git_root: &str) -> Result<(), AppError> {
        let resolved = self
            .resolver
            .resolve(git_root)?
            .ok_or_else(|| AppError::NotFound(format!("repository {git_root}")))?;
        let profile_id = self
            .resolve_profile(&resolved, git_root)?
            .ok_or_else(|| AppError::Validation("repository has no assigned profile".into()))?;
        self.apply_switch(&resolved, profile_id, git_root)
    }

    /// Record that the user dismissed a pending switch.
    pub(crate) fn cancel(&self, git_root: &str) -> Result<(), AppError> {
        self.record("identity.switch_cancelled", None, git_root)
    }

    /// Proactively pin the resolved profile into a single repository's local Git
    /// config, so the identity is already correct *before* the next commit —
    /// independent of when the watcher would otherwise react. A no-op when the
    /// repository is untracked or resolves to no profile. Never touches global
    /// config or the active marker.
    pub(crate) fn pin_repo(&self, git_root: &str) -> Result<(), AppError> {
        let Some(resolved) = self.resolver.resolve(git_root)? else {
            return Ok(());
        };
        let Some(profile_id) = self.resolve_profile(&resolved, git_root)? else {
            return Ok(());
        };
        self.switcher.apply_local(git_root, profile_id)
    }

    /// Pin every tracked repository's resolved identity into its local config.
    /// Best-effort: a failure on one repository (missing, bare, permission) must
    /// not abort the rest. Used on launch/reconcile so first commits are correct
    /// even for repositories driven only by a rule. Also pins each repo's
    /// path-scoped HTTPS credential when its profile owns one.
    pub(crate) fn pin_all(&self) {
        let Ok(roots) = self.resolver.watchable_roots() else {
            return;
        };
        for root in roots {
            let _ = self.pin_repo(&root);
        }
    }

    /// Entry point invoked from the watcher thread. Errors are swallowed so a
    /// transient failure (repo removed, key missing, Git busy) never crashes the
    /// watcher or the app.
    fn handle_activity(&self, git_root: &str) {
        let _ = self.process(git_root);
    }

    fn process(&self, git_root: &str) -> Result<(), AppError> {
        let settings = self.config()?;
        if !settings.enabled {
            return Ok(());
        }
        let Some(resolved) = self.resolver.resolve(git_root)? else {
            // Not a tracked repository — nothing to do.
            return Ok(());
        };
        let Some(profile_id) = self.resolve_profile(&resolved, git_root)? else {
            self.record("identity.no_assignment", None, git_root)?;
            self.emit_switch(SwitchStatus::NoAssignment, &resolved, None, git_root)?;
            return Ok(());
        };
        if self.switcher.active_profile_id()? == Some(profile_id) {
            self.record("identity.same_profile", Some(profile_id), git_root)?;
            return Ok(());
        }
        if settings.confirm_before_switch {
            self.emit_switch(
                SwitchStatus::PendingConfirmation,
                &resolved,
                Some(profile_id),
                git_root,
            )?;
            return Ok(());
        }
        self.apply_switch(&resolved, profile_id, git_root)
    }

    fn apply_switch(
        &self,
        resolved: &ResolvedRepo,
        profile_id: Uuid,
        git_root: &str,
    ) -> Result<(), AppError> {
        // The existing pipeline switches git config + HTTPS credentials + active
        // marker atomically, rolling back on failure. SSH follows via the managed
        // `~/.ssh/config` block.
        self.switcher.apply(profile_id)?;
        // Pin the identity into this repo's local config so the *next* commit
        // here reads the right author even before global switching would react.
        // Best-effort: the global switch above already succeeded, so a local-pin
        // failure (a moved repo, a missing backing credential) must never abort
        // or hide the switch — otherwise the engine silently appears to stop.
        let _ = self.switcher.apply_local(git_root, profile_id);
        self.record("identity.auto_switch", Some(profile_id), git_root)?;
        let event =
            self.build_event(SwitchStatus::Switched, resolved, Some(profile_id), git_root)?;
        *locked(&self.last_switch)? = Some(event.clone());
        self.observer.switched(&event);
        self.emit_status();
        Ok(())
    }

    fn emit_switch(
        &self,
        status: SwitchStatus,
        resolved: &ResolvedRepo,
        profile_id: Option<Uuid>,
        git_root: &str,
    ) -> Result<(), AppError> {
        let event = self.build_event(status, resolved, profile_id, git_root)?;
        self.observer.switched(&event);
        Ok(())
    }

    fn build_event(
        &self,
        status: SwitchStatus,
        resolved: &ResolvedRepo,
        profile_id: Option<Uuid>,
        git_root: &str,
    ) -> Result<SwitchEvent, AppError> {
        let profile_label = match profile_id {
            Some(id) => self.switcher.profile_label(id)?,
            None => None,
        };
        Ok(SwitchEvent {
            status,
            git_root: git_root.to_string(),
            repo_id: Some(resolved.repo_id),
            repo_name: Some(resolved.repo_name.clone()),
            profile_id,
            profile_label,
            at: Utc::now(),
        })
    }

    fn emit_status(&self) {
        if let Ok(status) = self.status() {
            self.observer.status_changed(&status);
        }
    }

    fn record(
        &self,
        action: &str,
        profile_id: Option<Uuid>,
        git_root: &str,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path: Some(git_root.to_string()),
        })
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
    use crate::domain::{profile::Profile, rule::RuleMatch, settings::AppSettings};

    // ---- mock ports ------------------------------------------------------

    #[derive(Default)]
    struct MockResolver {
        by_root: Mutex<Vec<(String, ResolvedRepo)>>,
        roots: Mutex<Vec<String>>,
    }
    impl MockResolver {
        fn add(&self, git_root: &str, repo_id: Uuid, name: &str, profile_id: Option<Uuid>) {
            self.by_root.lock().unwrap().push((
                git_root.into(),
                ResolvedRepo {
                    repo_id,
                    repo_name: name.into(),
                    profile_id,
                    remote_origin: None,
                },
            ));
            self.roots.lock().unwrap().push(git_root.into());
        }
    }
    impl IdentityResolver for MockResolver {
        fn resolve(&self, git_root: &str) -> Result<Option<ResolvedRepo>, AppError> {
            Ok(self
                .by_root
                .lock()
                .unwrap()
                .iter()
                .find(|(r, _)| r == git_root)
                .map(|(_, v)| ResolvedRepo {
                    repo_id: v.repo_id,
                    repo_name: v.repo_name.clone(),
                    profile_id: v.profile_id,
                    remote_origin: v.remote_origin.clone(),
                }))
        }
        fn watchable_roots(&self) -> Result<Vec<String>, AppError> {
            Ok(self.roots.lock().unwrap().clone())
        }
    }

    #[derive(Default)]
    struct MockSwitcher {
        active: Mutex<Option<Uuid>>,
        applied: Mutex<Vec<Uuid>>,
        pinned: Mutex<Vec<(String, Uuid)>>,
        labels: Mutex<Vec<(Uuid, String)>>,
        /// When set, `apply_local` fails — simulating a moved repo or a missing
        /// backing credential. The switch itself must survive this.
        fail_local: Mutex<bool>,
    }
    impl IdentitySwitcher for MockSwitcher {
        fn active_profile_id(&self) -> Result<Option<Uuid>, AppError> {
            Ok(*self.active.lock().unwrap())
        }
        fn profile_label(&self, id: Uuid) -> Result<Option<String>, AppError> {
            Ok(self
                .labels
                .lock()
                .unwrap()
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, l)| l.clone()))
        }
        fn apply(&self, profile_id: Uuid) -> Result<(), AppError> {
            self.applied.lock().unwrap().push(profile_id);
            *self.active.lock().unwrap() = Some(profile_id);
            Ok(())
        }
        fn apply_local(&self, git_root: &str, profile_id: Uuid) -> Result<(), AppError> {
            if *self.fail_local.lock().unwrap() {
                return Err(AppError::Internal("local pin failed".into()));
            }
            self.pinned
                .lock()
                .unwrap()
                .push((git_root.to_string(), profile_id));
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockWatcher {
        watching: Mutex<bool>,
        paused: Mutex<bool>,
        count: Mutex<usize>,
        starts: Mutex<u32>,
    }
    impl WorkspaceWatcher for MockWatcher {
        fn start(
            &self,
            git_roots: &[String],
            _on_activity: Arc<dyn Fn(String) + Send + Sync>,
        ) -> Result<(), AppError> {
            *self.watching.lock().unwrap() = true;
            *self.paused.lock().unwrap() = false;
            *self.count.lock().unwrap() = git_roots.len();
            *self.starts.lock().unwrap() += 1;
            Ok(())
        }
        fn stop(&self) {
            *self.watching.lock().unwrap() = false;
            *self.count.lock().unwrap() = 0;
        }
        fn pause(&self) {
            *self.paused.lock().unwrap() = true;
        }
        fn resume(&self) {
            *self.paused.lock().unwrap() = false;
        }
        fn is_watching(&self) -> bool {
            *self.watching.lock().unwrap()
        }
        fn is_paused(&self) -> bool {
            *self.paused.lock().unwrap()
        }
        fn watched_count(&self) -> usize {
            *self.count.lock().unwrap()
        }
    }

    /// When `forced` is set, every resolution returns that profile (simulating a
    /// matching rule). Otherwise it matches nothing, so the manual assignment
    /// stands — the backward-compatible path.
    #[derive(Default)]
    struct MockRuleResolver {
        forced: Mutex<Option<Uuid>>,
    }
    impl RuleResolver for MockRuleResolver {
        fn resolve(&self, _ctx: &EvaluationContext) -> Result<Option<RuleMatch>, AppError> {
            Ok(self.forced.lock().unwrap().map(|profile_id| RuleMatch {
                rule_id: Uuid::new_v4(),
                rule_name: "forced".into(),
                profile_id,
                reason: "test".into(),
            }))
        }
    }

    #[derive(Default)]
    struct MockObserver {
        events: Mutex<Vec<SwitchEvent>>,
        statuses: Mutex<Vec<SmartSwitchStatus>>,
    }
    impl SwitchObserver for MockObserver {
        fn switched(&self, event: &SwitchEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
        fn status_changed(&self, status: &SmartSwitchStatus) {
            self.statuses.lock().unwrap().push(status.clone());
        }
    }

    struct MockSettingsStore {
        settings: Mutex<AppSettings>,
    }
    impl ProfileStore for MockSettingsStore {
        fn load_all(&self) -> Result<Vec<Profile>, AppError> {
            Ok(vec![])
        }
        fn find(&self, _id: Uuid) -> Result<Option<Profile>, AppError> {
            Ok(None)
        }
        fn save(&self, _profile: &Profile) -> Result<(), AppError> {
            Ok(())
        }
        fn delete(&self, _id: Uuid) -> Result<(), AppError> {
            Ok(())
        }
        fn load_settings(&self) -> Result<AppSettings, AppError> {
            Ok(self.settings.lock().unwrap().clone())
        }
        fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
            *self.settings.lock().unwrap() = settings.clone();
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
        svc: Arc<IdentitySwitchService>,
        resolver: Arc<MockResolver>,
        switcher: Arc<MockSwitcher>,
        watcher: Arc<MockWatcher>,
        observer: Arc<MockObserver>,
        rules: Arc<MockRuleResolver>,
        audit: Arc<MockAudit>,
    }

    fn harness(smart: SmartSwitchingSettings) -> Harness {
        let resolver = Arc::new(MockResolver::default());
        let switcher = Arc::new(MockSwitcher::default());
        let watcher = Arc::new(MockWatcher::default());
        let observer = Arc::new(MockObserver::default());
        let rules = Arc::new(MockRuleResolver::default());
        let audit = Arc::new(MockAudit::default());
        let settings = AppSettings {
            smart_switching: smart,
            ..AppSettings::default()
        };
        let store = Arc::new(MockSettingsStore {
            settings: Mutex::new(settings),
        });
        let svc = Arc::new(IdentitySwitchService::new(
            resolver.clone(),
            switcher.clone(),
            watcher.clone(),
            observer.clone(),
            store,
            rules.clone(),
            audit.clone(),
        ));
        Harness {
            svc,
            resolver,
            switcher,
            watcher,
            observer,
            rules,
            audit,
        }
    }

    fn enabled() -> SmartSwitchingSettings {
        SmartSwitchingSettings {
            enabled: true,
            ..SmartSwitchingSettings::default()
        }
    }

    fn actions(h: &Harness) -> Vec<String> {
        h.audit
            .read_all()
            .unwrap()
            .into_iter()
            .map(|e| e.action)
            .collect()
    }

    // ---- tests -----------------------------------------------------------

    #[test]
    fn switches_when_assigned_profile_differs() {
        let h = harness(enabled());
        let pid = Uuid::new_v4();
        h.switcher.labels.lock().unwrap().push((pid, "Work".into()));
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));

        h.svc.handle_activity("/repo/a");

        assert_eq!(h.switcher.applied.lock().unwrap().as_slice(), &[pid]);
        // The repo is also pinned locally so its next commit reads the right author.
        assert_eq!(
            h.switcher.pinned.lock().unwrap().as_slice(),
            &[("/repo/a".to_string(), pid)]
        );
        assert!(actions(&h).contains(&"identity.auto_switch".to_string()));
        let events = h.observer.events.lock().unwrap();
        assert_eq!(events.last().unwrap().status, SwitchStatus::Switched);
        assert_eq!(
            events.last().unwrap().profile_label.as_deref(),
            Some("Work")
        );
    }

    #[test]
    fn local_pin_failure_does_not_suppress_switch() {
        // A failing local pin (moved repo, missing backing credential) must not
        // abort the switch: the global identity still changes and the UI still
        // gets a Switched event. Regression guard for the "engine stopped
        // switching" bug caused by a hard `?` on apply_local.
        let h = harness(enabled());
        let pid = Uuid::new_v4();
        h.switcher.labels.lock().unwrap().push((pid, "Work".into()));
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));
        *h.switcher.fail_local.lock().unwrap() = true;

        h.svc.handle_activity("/repo/a");

        assert_eq!(h.switcher.applied.lock().unwrap().as_slice(), &[pid]);
        assert!(actions(&h).contains(&"identity.auto_switch".to_string()));
        let events = h.observer.events.lock().unwrap();
        assert_eq!(events.last().unwrap().status, SwitchStatus::Switched);
    }

    #[test]
    fn rule_match_overrides_manual_assignment() {
        let h = harness(enabled());
        let manual = Uuid::new_v4();
        let rule_profile = Uuid::new_v4();
        h.switcher
            .labels
            .lock()
            .unwrap()
            .push((rule_profile, "Rule".into()));
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(manual));
        *h.rules.forced.lock().unwrap() = Some(rule_profile);

        h.svc.handle_activity("/repo/a");

        // The rule's profile wins over the manual assignment.
        assert_eq!(
            h.switcher.applied.lock().unwrap().as_slice(),
            &[rule_profile]
        );
        assert!(actions(&h).contains(&"rule.matched".to_string()));
    }

    #[test]
    fn rule_match_applies_without_manual_assignment() {
        let h = harness(enabled());
        let rule_profile = Uuid::new_v4();
        // No manual assignment — the CTO success scenario.
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", None);
        *h.rules.forced.lock().unwrap() = Some(rule_profile);

        h.svc.handle_activity("/repo/a");

        assert_eq!(
            h.switcher.applied.lock().unwrap().as_slice(),
            &[rule_profile]
        );
        assert!(actions(&h).contains(&"rule.matched".to_string()));
    }

    #[test]
    fn falls_back_to_manual_assignment_when_no_rule_matches() {
        let h = harness(enabled());
        let manual = Uuid::new_v4();
        h.switcher
            .labels
            .lock()
            .unwrap()
            .push((manual, "Work".into()));
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(manual));
        // rules.forced stays None → no rule matches; manual assignment applies.

        h.svc.handle_activity("/repo/a");

        assert_eq!(h.switcher.applied.lock().unwrap().as_slice(), &[manual]);
        assert!(!actions(&h).contains(&"rule.matched".to_string()));
    }

    #[test]
    fn no_switch_when_already_active() {
        let h = harness(enabled());
        let pid = Uuid::new_v4();
        *h.switcher.active.lock().unwrap() = Some(pid);
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));

        h.svc.handle_activity("/repo/a");

        assert!(h.switcher.applied.lock().unwrap().is_empty());
        assert!(actions(&h).contains(&"identity.same_profile".to_string()));
    }

    #[test]
    fn records_no_assignment() {
        let h = harness(enabled());
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", None);

        h.svc.handle_activity("/repo/a");

        assert!(h.switcher.applied.lock().unwrap().is_empty());
        assert!(actions(&h).contains(&"identity.no_assignment".to_string()));
        assert_eq!(
            h.observer.events.lock().unwrap().last().unwrap().status,
            SwitchStatus::NoAssignment
        );
    }

    #[test]
    fn confirm_before_switch_defers_apply() {
        let h = harness(SmartSwitchingSettings {
            enabled: true,
            confirm_before_switch: true,
            ..SmartSwitchingSettings::default()
        });
        let pid = Uuid::new_v4();
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));

        h.svc.handle_activity("/repo/a");
        assert!(h.switcher.applied.lock().unwrap().is_empty());
        assert_eq!(
            h.observer.events.lock().unwrap().last().unwrap().status,
            SwitchStatus::PendingConfirmation
        );

        // Confirming applies it.
        h.svc.confirm("/repo/a").unwrap();
        assert_eq!(h.switcher.applied.lock().unwrap().as_slice(), &[pid]);
        assert!(actions(&h).contains(&"identity.auto_switch".to_string()));
    }

    #[test]
    fn cancel_records_activity() {
        let h = harness(enabled());
        h.svc.cancel("/repo/a").unwrap();
        assert!(actions(&h).contains(&"identity.switch_cancelled".to_string()));
    }

    #[test]
    fn disabled_ignores_activity() {
        let h = harness(SmartSwitchingSettings::default());
        let pid = Uuid::new_v4();
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));
        h.svc.handle_activity("/repo/a");
        assert!(h.switcher.applied.lock().unwrap().is_empty());
        assert!(actions(&h).is_empty());
    }

    #[test]
    fn reconcile_starts_when_enabled_and_stops_when_disabled() {
        let h = harness(enabled());
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", None);
        h.resolver.add("/repo/b", Uuid::new_v4(), "b", None);

        h.svc.reconcile().unwrap();
        assert!(h.watcher.is_watching());
        assert_eq!(h.svc.status().unwrap().repos_monitored, 2);

        h.svc.set_enabled(false).unwrap();
        assert!(!h.watcher.is_watching());
        assert!(!h.svc.status().unwrap().enabled);
    }

    #[test]
    fn reconcile_pins_tracked_repos_up_front() {
        let h = harness(enabled());
        let pid = Uuid::new_v4();
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(pid));
        h.resolver.add("/repo/b", Uuid::new_v4(), "b", None);

        h.svc.reconcile().unwrap();

        // Only the assigned repo is pinned; the unassigned one is left untouched.
        assert_eq!(
            h.switcher.pinned.lock().unwrap().as_slice(),
            &[("/repo/a".to_string(), pid)]
        );
    }

    #[test]
    fn pin_repo_uses_rule_over_manual_assignment() {
        let h = harness(enabled());
        let manual = Uuid::new_v4();
        let rule_profile = Uuid::new_v4();
        h.resolver.add("/repo/a", Uuid::new_v4(), "a", Some(manual));
        *h.rules.forced.lock().unwrap() = Some(rule_profile);

        h.svc.pin_repo("/repo/a").unwrap();

        assert_eq!(
            h.switcher.pinned.lock().unwrap().as_slice(),
            &[("/repo/a".to_string(), rule_profile)]
        );
    }

    #[test]
    fn pause_and_resume_toggle_status() {
        let h = harness(enabled());
        h.svc.reconcile().unwrap();
        h.svc.pause();
        assert!(h.svc.status().unwrap().paused);
        h.svc.resume();
        assert!(!h.svc.status().unwrap().paused);
    }
}
