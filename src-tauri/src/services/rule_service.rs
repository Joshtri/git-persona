use crate::domain::{
    audit::AuditEntry,
    ports::{AuditSink, ProfileStore, RuleResolver, RuleStore},
    rule::{
        EvaluationContext, Rule, RuleCondition, RuleMatch, RulePreviewInput, RulePreviewResult,
        RuleSummary,
    },
};
use crate::error::AppError;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// The Rule Engine — the decision layer that selects a profile for a repository.
///
/// It owns no identity mutation: it reads rules, evaluates them against
/// repository facts, and returns a [`RuleMatch`]. Execution stays entirely in the
/// existing profile-apply pipeline.
pub(crate) struct RuleService {
    store: Arc<dyn RuleStore>,
    profiles: Arc<dyn ProfileStore>,
    audit: Arc<dyn AuditSink>,
    /// The most recent rule that actually resolved a switch, for the dashboard.
    /// In-memory (resets on restart), mirroring `SmartSwitchStatus.last_switch`.
    last_match: Mutex<Option<RuleMatch>>,
}

impl RuleService {
    pub(crate) fn new(
        store: Arc<dyn RuleStore>,
        profiles: Arc<dyn ProfileStore>,
        audit: Arc<dyn AuditSink>,
    ) -> Self {
        Self {
            store,
            profiles,
            audit,
            last_match: Mutex::new(None),
        }
    }

    fn log(
        &self,
        action: &str,
        profile_id: Option<Uuid>,
        name: Option<String>,
    ) -> Result<(), AppError> {
        self.audit.append(&AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.into(),
            profile_id,
            repo_path: name,
        })
    }

    /// Rules ordered for evaluation and display: priority ascending.
    pub(crate) fn list(&self) -> Result<Vec<Rule>, AppError> {
        let mut rules = self.store.load_all()?;
        rules.sort_by_key(|r| r.priority);
        Ok(rules)
    }

    pub(crate) fn get(&self, id: Uuid) -> Result<Rule, AppError> {
        self.store
            .find(id)?
            .ok_or_else(|| AppError::NotFound(format!("rule {id}")))
    }

    fn validate(
        &self,
        name: &str,
        condition: &RuleCondition,
        target_profile_id: Uuid,
    ) -> Result<(), AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("rule name is required".into()));
        }
        if condition.value.trim().is_empty() {
            return Err(AppError::Validation("condition value is required".into()));
        }
        if !condition.subject.allows(condition.operator) {
            return Err(AppError::Validation(format!(
                "{} does not support the \"{}\" operator",
                condition.subject.label(),
                condition.operator.label()
            )));
        }
        if self.profiles.find(target_profile_id)?.is_none() {
            return Err(AppError::NotFound(format!("profile {target_profile_id}")));
        }
        Ok(())
    }

    fn next_priority(&self) -> Result<i32, AppError> {
        Ok(self
            .store
            .load_all()?
            .iter()
            .map(|r| r.priority)
            .max()
            .map_or(0, |p| p.saturating_add(1)))
    }

    pub(crate) fn create(
        &self,
        name: &str,
        condition: RuleCondition,
        target_profile_id: Uuid,
    ) -> Result<Rule, AppError> {
        self.validate(name, &condition, target_profile_id)?;
        let now = Utc::now();
        let rule = Rule {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            enabled: true,
            priority: self.next_priority()?,
            target_profile_id,
            condition,
            created_at: now,
            updated_at: now,
        };
        self.store.save(&rule)?;
        self.log(
            "rule.created",
            Some(target_profile_id),
            Some(rule.name.clone()),
        )?;
        Ok(rule)
    }

    pub(crate) fn update(
        &self,
        id: Uuid,
        name: &str,
        condition: RuleCondition,
        target_profile_id: Uuid,
    ) -> Result<Rule, AppError> {
        self.validate(name, &condition, target_profile_id)?;
        let mut rule = self.get(id)?;
        rule.name = name.trim().to_string();
        rule.condition = condition;
        rule.target_profile_id = target_profile_id;
        rule.updated_at = Utc::now();
        self.store.save(&rule)?;
        self.log(
            "rule.updated",
            Some(target_profile_id),
            Some(rule.name.clone()),
        )?;
        Ok(rule)
    }

    pub(crate) fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let rule = self.get(id)?;
        self.store.delete(id)?;
        self.log(
            "rule.deleted",
            Some(rule.target_profile_id),
            Some(rule.name),
        )?;
        Ok(())
    }

    pub(crate) fn duplicate(&self, id: Uuid) -> Result<Rule, AppError> {
        let source = self.get(id)?;
        let now = Utc::now();
        let rule = Rule {
            id: Uuid::new_v4(),
            name: format!("Copy of {}", source.name),
            enabled: source.enabled,
            priority: self.next_priority()?,
            target_profile_id: source.target_profile_id,
            condition: source.condition,
            created_at: now,
            updated_at: now,
        };
        self.store.save(&rule)?;
        self.log(
            "rule.created",
            Some(rule.target_profile_id),
            Some(rule.name.clone()),
        )?;
        Ok(rule)
    }

    pub(crate) fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<Rule, AppError> {
        let mut rule = self.get(id)?;
        rule.enabled = enabled;
        rule.updated_at = Utc::now();
        self.store.save(&rule)?;
        let action = if enabled {
            "rule.enabled"
        } else {
            "rule.disabled"
        };
        self.log(
            action,
            Some(rule.target_profile_id),
            Some(rule.name.clone()),
        )?;
        Ok(rule)
    }

    /// Enable or disable every rule at once (command-palette bulk action).
    pub(crate) fn set_all_enabled(&self, enabled: bool) -> Result<Vec<Rule>, AppError> {
        let mut rules = self.store.load_all()?;
        for rule in &mut rules {
            rule.enabled = enabled;
            rule.updated_at = Utc::now();
        }
        self.store.save_all(&rules)?;
        let action = if enabled {
            "rule.enabled"
        } else {
            "rule.disabled"
        };
        self.log(action, None, Some("all rules".into()))?;
        self.list()
    }

    /// Reassign priorities from an explicit ordering. Ids not present keep their
    /// relative order after the listed ones. Normalizes priorities to `0..n`.
    pub(crate) fn reorder(&self, ordered_ids: &[Uuid]) -> Result<Vec<Rule>, AppError> {
        let mut rules = self.store.load_all()?;
        let rank = |id: Uuid| ordered_ids.iter().position(|&x| x == id);
        rules.sort_by(|a, b| match (rank(a.id), rank(b.id)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.priority.cmp(&b.priority),
        });
        for (index, rule) in rules.iter_mut().enumerate() {
            rule.priority = i32::try_from(index).unwrap_or(i32::MAX);
        }
        self.store.save_all(&rules)?;
        self.list()
    }

    /// Resolve the first matching enabled rule for `ctx`, highest priority first.
    /// Pure read — never mutates state. Used by both preview and the resolver.
    pub(crate) fn evaluate(&self, ctx: &EvaluationContext) -> Result<Option<RuleMatch>, AppError> {
        let mut rules: Vec<Rule> = self
            .store
            .load_all()?
            .into_iter()
            .filter(|r| r.enabled)
            .collect();
        rules.sort_by_key(|r| r.priority);
        for rule in rules {
            if let Some(reason) = rule.condition.matches(ctx) {
                return Ok(Some(RuleMatch {
                    rule_id: rule.id,
                    rule_name: rule.name,
                    profile_id: rule.target_profile_id,
                    reason,
                }));
            }
        }
        Ok(None)
    }

    /// Evaluate a hypothetical repository. Never applies a switch.
    pub(crate) fn preview(&self, input: &RulePreviewInput) -> Result<RulePreviewResult, AppError> {
        let ctx =
            EvaluationContext::from_parts(&input.path, &input.name, input.remote_url.as_deref());
        Ok(RulePreviewResult {
            matched: self.evaluate(&ctx)?,
        })
    }

    pub(crate) fn summary(&self) -> Result<RuleSummary, AppError> {
        let rules = self.store.load_all()?;
        let active = u32::try_from(rules.iter().filter(|r| r.enabled).count()).unwrap_or(u32::MAX);
        let disabled =
            u32::try_from(rules.iter().filter(|r| !r.enabled).count()).unwrap_or(u32::MAX);
        Ok(RuleSummary {
            active,
            disabled,
            last_match: self
                .last_match
                .lock()
                .map_err(|_| AppError::Internal("rule last-match lock poisoned".into()))?
                .clone(),
        })
    }

    /// Serialize all rules (priority ascending) as pretty JSON for export.
    pub(crate) fn export(&self) -> Result<String, AppError> {
        let rules = self.list()?;
        let json =
            serde_json::to_string_pretty(&rules).map_err(|e| AppError::Internal(e.to_string()))?;
        self.log(
            "rule.exported",
            None,
            Some(format!("{} rules", rules.len())),
        )?;
        Ok(json)
    }

    /// Import rules from JSON. `replace` wipes the existing set; otherwise the
    /// imported rules are appended. Imported rules always receive fresh ids and
    /// re-normalized priorities so a set can never collide with the local one.
    pub(crate) fn import(&self, json: &str, replace: bool) -> Result<Vec<Rule>, AppError> {
        let parsed: Vec<Rule> = serde_json::from_str(json)
            .map_err(|e| AppError::Validation(format!("invalid rules file: {e}")))?;
        for rule in &parsed {
            if self.profiles.find(rule.target_profile_id)?.is_none() {
                return Err(AppError::NotFound(format!(
                    "rule \"{}\" targets a profile that does not exist on this machine",
                    rule.name
                )));
            }
            if !rule.condition.subject.allows(rule.condition.operator) {
                return Err(AppError::Validation(format!(
                    "rule \"{}\" has an unsupported operator for its subject",
                    rule.name
                )));
            }
        }

        let now = Utc::now();
        let mut existing = if replace {
            Vec::new()
        } else {
            self.store.load_all()?
        };
        let mut priority = existing
            .iter()
            .map(|r| r.priority)
            .max()
            .map_or(0, |p| p + 1);
        for mut rule in parsed {
            rule.id = Uuid::new_v4();
            rule.priority = priority;
            rule.created_at = now;
            rule.updated_at = now;
            priority = priority.saturating_add(1);
            existing.push(rule);
        }
        self.store.save_all(&existing)?;
        self.log(
            "rule.imported",
            None,
            Some(format!("{} rules", existing.len())),
        )?;
        self.list()
    }
}

impl RuleResolver for RuleService {
    fn resolve(&self, ctx: &EvaluationContext) -> Result<Option<RuleMatch>, AppError> {
        let matched = self.evaluate(ctx)?;
        if let Some(m) = &matched {
            if let Ok(mut slot) = self.last_match.lock() {
                *slot = Some(m.clone());
            }
        }
        Ok(matched)
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
        rule::{RuleOperator, RuleSubject},
        settings::AppSettings,
    };

    // ---- mock ports ------------------------------------------------------

    #[derive(Default)]
    struct MockRuleStore {
        items: Mutex<Vec<Rule>>,
    }
    impl RuleStore for MockRuleStore {
        fn load_all(&self) -> Result<Vec<Rule>, AppError> {
            Ok(self.items.lock().unwrap().clone())
        }
        fn find(&self, id: Uuid) -> Result<Option<Rule>, AppError> {
            Ok(self
                .items
                .lock()
                .unwrap()
                .iter()
                .find(|r| r.id == id)
                .cloned())
        }
        fn save(&self, rule: &Rule) -> Result<(), AppError> {
            let mut items = self.items.lock().unwrap();
            if let Some(slot) = items.iter_mut().find(|r| r.id == rule.id) {
                *slot = rule.clone();
            } else {
                items.push(rule.clone());
            }
            Ok(())
        }
        fn save_all(&self, rules: &[Rule]) -> Result<(), AppError> {
            *self.items.lock().unwrap() = rules.to_vec();
            Ok(())
        }
        fn delete(&self, id: Uuid) -> Result<(), AppError> {
            let mut items = self.items.lock().unwrap();
            let before = items.len();
            items.retain(|r| r.id != id);
            if items.len() == before {
                return Err(AppError::NotFound(format!("rule {id}")));
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
        svc: RuleService,
        store: Arc<MockRuleStore>,
        profiles: Arc<MockProfileStore>,
        audit: Arc<MockAudit>,
    }

    fn harness() -> Harness {
        let store = Arc::new(MockRuleStore::default());
        let profiles = Arc::new(MockProfileStore::default());
        let audit = Arc::new(MockAudit::default());
        let svc = RuleService::new(store.clone(), profiles.clone(), audit.clone());
        Harness {
            svc,
            store,
            profiles,
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

    fn cond(subject: RuleSubject, operator: RuleOperator, value: &str) -> RuleCondition {
        RuleCondition {
            subject,
            operator,
            value: value.into(),
        }
    }

    fn ctx(path: &str, name: &str, remote: Option<&str>) -> EvaluationContext {
        EvaluationContext::from_parts(path, name, remote)
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
    fn create_persists_rule_and_logs() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "Company",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/company/"),
                pid,
            )
            .unwrap();
        assert!(rule.enabled);
        assert_eq!(h.store.load_all().unwrap().len(), 1);
        assert!(actions(&h).contains(&"rule.created".to_string()));
    }

    #[test]
    fn create_rejects_unknown_profile() {
        let h = harness();
        let err = h
            .svc
            .create(
                "X",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                Uuid::new_v4(),
            )
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn create_rejects_illegal_operator_for_host() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let err = h
            .svc
            .create(
                "X",
                cond(
                    RuleSubject::RemoteHost,
                    RuleOperator::Contains,
                    "github.com",
                ),
                pid,
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn create_rejects_empty_value() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let err = h
            .svc
            .create(
                "X",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "  "),
                pid,
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn update_changes_fields() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        let updated = h
            .svc
            .update(
                rule.id,
                "B",
                cond(RuleSubject::RepoName, RuleOperator::StartsWith, "api-"),
                pid,
            )
            .unwrap();
        assert_eq!(updated.name, "B");
        assert_eq!(updated.condition.operator, RuleOperator::StartsWith);
        assert!(actions(&h).contains(&"rule.updated".to_string()));
    }

    #[test]
    fn delete_removes_rule() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        h.svc.delete(rule.id).unwrap();
        assert!(h.store.load_all().unwrap().is_empty());
        assert!(actions(&h).contains(&"rule.deleted".to_string()));
    }

    #[test]
    fn duplicate_copies_with_new_identity() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        let dup = h.svc.duplicate(rule.id).unwrap();
        assert_ne!(dup.id, rule.id);
        assert_eq!(dup.name, "Copy of A");
        assert!(dup.priority > rule.priority);
        assert_eq!(h.store.load_all().unwrap().len(), 2);
    }

    #[test]
    fn enable_disable_toggles_and_logs() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        let off = h.svc.set_enabled(rule.id, false).unwrap();
        assert!(!off.enabled);
        assert!(actions(&h).contains(&"rule.disabled".to_string()));
        let on = h.svc.set_enabled(rule.id, true).unwrap();
        assert!(on.enabled);
        assert!(actions(&h).contains(&"rule.enabled".to_string()));
    }

    #[test]
    fn priority_ordering_first_match_wins() {
        let h = harness();
        let work = add_profile(&h, "Work");
        let oss = add_profile(&h, "OSS");
        // Both match, but the lower-priority (created first) rule wins.
        let first = h
            .svc
            .create(
                "First",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/code/"),
                work,
            )
            .unwrap();
        h.svc
            .create(
                "Second",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/code/"),
                oss,
            )
            .unwrap();
        let m = h
            .svc
            .evaluate(&ctx("/home/code/x", "x", None))
            .unwrap()
            .unwrap();
        assert_eq!(m.profile_id, work);
        assert_eq!(m.rule_id, first.id);
    }

    #[test]
    fn disabled_rules_are_ignored() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let rule = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/code/"),
                pid,
            )
            .unwrap();
        h.svc.set_enabled(rule.id, false).unwrap();
        assert!(h
            .svc
            .evaluate(&ctx("/home/code/x", "x", None))
            .unwrap()
            .is_none());
    }

    #[test]
    fn no_rule_matched_returns_none() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/company/"),
                pid,
            )
            .unwrap();
        assert!(h
            .svc
            .evaluate(&ctx("/home/oss/x", "x", None))
            .unwrap()
            .is_none());
    }

    #[test]
    fn reorder_changes_first_match() {
        let h = harness();
        let work = add_profile(&h, "Work");
        let oss = add_profile(&h, "OSS");
        let a = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/code/"),
                work,
            )
            .unwrap();
        let b = h
            .svc
            .create(
                "B",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/code/"),
                oss,
            )
            .unwrap();
        h.svc.reorder(&[b.id, a.id]).unwrap();
        let m = h
            .svc
            .evaluate(&ctx("/home/code/x", "x", None))
            .unwrap()
            .unwrap();
        assert_eq!(m.profile_id, oss);
    }

    #[test]
    fn preview_never_updates_last_match() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/company/"),
                pid,
            )
            .unwrap();
        let result = h
            .svc
            .preview(&RulePreviewInput {
                path: "/home/company/api".into(),
                name: "api".into(),
                remote_url: None,
            })
            .unwrap();
        assert!(result.matched.is_some());
        // Preview must not mutate the dashboard's last-match.
        assert!(h.svc.summary().unwrap().last_match.is_none());
    }

    #[test]
    fn resolve_updates_last_match() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/company/"),
                pid,
            )
            .unwrap();
        let m = h
            .svc
            .resolve(&ctx("/home/company/api", "api", None))
            .unwrap();
        assert!(m.is_some());
        assert!(h.svc.summary().unwrap().last_match.is_some());
    }

    #[test]
    fn summary_counts_active_and_disabled() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        let a = h
            .svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        h.svc
            .create(
                "B",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "web"),
                pid,
            )
            .unwrap();
        h.svc.set_enabled(a.id, false).unwrap();
        let summary = h.svc.summary().unwrap();
        assert_eq!(summary.active, 1);
        assert_eq!(summary.disabled, 1);
    }

    #[test]
    fn export_import_round_trip() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoPath, RuleOperator::Contains, "/company/"),
                pid,
            )
            .unwrap();
        let json = h.svc.export().unwrap();

        // Import into a fresh service that shares the same profile store.
        let store2 = Arc::new(MockRuleStore::default());
        let svc2 = RuleService::new(
            store2.clone(),
            h.profiles.clone(),
            Arc::new(MockAudit::default()),
        );
        let imported = svc2.import(&json, true).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].name, "A");
        assert!(actions(&h).contains(&"rule.exported".to_string()));
    }

    #[test]
    fn import_merge_appends() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        let json = h.svc.export().unwrap();
        let merged = h.svc.import(&json, false).unwrap();
        assert_eq!(merged.len(), 2);
        // Priorities are re-normalized and unique.
        assert_ne!(merged[0].priority, merged[1].priority);
    }

    #[test]
    fn import_rejects_dangling_profile() {
        let h = harness();
        let pid = add_profile(&h, "Work");
        h.svc
            .create(
                "A",
                cond(RuleSubject::RepoName, RuleOperator::Equals, "api"),
                pid,
            )
            .unwrap();
        let json = h.svc.export().unwrap();

        // Import against a service with no matching profiles.
        let svc2 = RuleService::new(
            Arc::new(MockRuleStore::default()),
            Arc::new(MockProfileStore::default()),
            Arc::new(MockAudit::default()),
        );
        let err = svc2.import(&json, true).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
