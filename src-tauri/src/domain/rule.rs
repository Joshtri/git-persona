use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// The repository fact a rule condition inspects.
///
/// `RemoteHost` and `Owner` are derived by parsing the remote URL and only
/// support the `Equals` operator (see [`RuleSubject::allows`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum RuleSubject {
    RepoPath,
    RepoName,
    RemoteUrl,
    RemoteHost,
    Owner,
}

impl RuleSubject {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RepoPath => "Repository path",
            Self::RepoName => "Repository name",
            Self::RemoteUrl => "Remote URL",
            Self::RemoteHost => "Remote host",
            Self::Owner => "Owner",
        }
    }

    /// Which operators are legal for this subject. Host and owner are exact
    /// identifiers — substring matching on them would be surprising — so they are
    /// restricted to `Equals`.
    pub(crate) fn allows(self, operator: RuleOperator) -> bool {
        match self {
            Self::RemoteHost | Self::Owner => matches!(operator, RuleOperator::Equals),
            Self::RepoPath | Self::RepoName | Self::RemoteUrl => true,
        }
    }

    fn value(self, ctx: &EvaluationContext) -> Option<&str> {
        match self {
            Self::RepoPath => Some(ctx.path.as_str()),
            Self::RepoName => Some(ctx.name.as_str()),
            Self::RemoteUrl => ctx.remote_url.as_deref(),
            Self::RemoteHost => ctx.remote_host.as_deref(),
            Self::Owner => ctx.owner.as_deref(),
        }
    }
}

/// The comparison a rule condition applies. Deterministic string matching only —
/// no regular expressions, globbing, or scripting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum RuleOperator {
    Contains,
    StartsWith,
    EndsWith,
    Equals,
}

impl RuleOperator {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::StartsWith => "starts with",
            Self::EndsWith => "ends with",
            Self::Equals => "equals",
        }
    }

    /// Case-insensitive comparison. Repository paths are case-insensitive on
    /// Windows and hosts/owners are case-insensitive by convention, so folding
    /// case keeps matching predictable across machines.
    fn apply(self, haystack: &str, needle: &str) -> bool {
        let haystack = haystack.to_lowercase();
        let needle = needle.to_lowercase();
        match self {
            Self::Contains => haystack.contains(&needle),
            Self::StartsWith => haystack.starts_with(&needle),
            Self::EndsWith => haystack.ends_with(&needle),
            Self::Equals => haystack == needle,
        }
    }
}

/// A single declarative predicate: does `subject` `operator` `value` hold for a
/// repository?
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RuleCondition {
    pub(crate) subject: RuleSubject,
    pub(crate) operator: RuleOperator,
    pub(crate) value: String,
}

impl RuleCondition {
    /// Evaluate the condition against a repository context. Returns a
    /// human-readable reason on match, or `None` when it does not apply.
    pub(crate) fn matches(&self, ctx: &EvaluationContext) -> Option<String> {
        if self.value.trim().is_empty() {
            return None;
        }
        let field = self.subject.value(ctx)?;
        if self.operator.apply(field, self.value.trim()) {
            Some(format!(
                "{} {} \"{}\"",
                self.subject.label(),
                self.operator.label(),
                self.value.trim()
            ))
        } else {
            None
        }
    }
}

/// A rule mapping a repository condition to a target profile. Rules are the
/// decision layer: they only ever *select* a profile — execution stays in the
/// existing profile-apply pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct Rule {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    /// Lower number = higher priority. Evaluated ascending; first match wins.
    pub(crate) priority: i32,
    pub(crate) target_profile_id: Uuid,
    pub(crate) condition: RuleCondition,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

/// The outcome of a successful rule resolution. Carries display metadata only —
/// consumed by the preview, the dashboard summary, and the switching audit.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RuleMatch {
    pub(crate) rule_id: Uuid,
    pub(crate) rule_name: String,
    pub(crate) profile_id: Uuid,
    pub(crate) reason: String,
}

/// Input for the preview command — the user describes a hypothetical repository
/// and sees which rule (if any) would match. Never triggers a switch.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RulePreviewInput {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) remote_url: Option<String>,
}

/// The result of a preview evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RulePreviewResult {
    pub(crate) matched: Option<RuleMatch>,
}

/// Aggregate figures for the dashboard widget.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) struct RuleSummary {
    pub(crate) active: u32,
    pub(crate) disabled: u32,
    pub(crate) last_match: Option<RuleMatch>,
}

/// The set of repository facts a rule is evaluated against. Domain-internal —
/// built from a repository's path, name, and remote URL; never serialized.
pub(crate) struct EvaluationContext {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) remote_url: Option<String>,
    pub(crate) remote_host: Option<String>,
    pub(crate) owner: Option<String>,
}

impl EvaluationContext {
    /// Build a context, deriving `remote_host` and `owner` from the remote URL.
    pub(crate) fn from_parts(path: &str, name: &str, remote_url: Option<&str>) -> Self {
        let (remote_host, owner) = remote_url.map_or((None, None), parse_remote);
        Self {
            path: path.to_string(),
            name: name.to_string(),
            remote_url: remote_url.map(str::to_string),
            remote_host,
            owner,
        }
    }
}

/// Extract `(host, owner)` from a Git remote URL. Handles the two forms Git
/// emits: SCP-style SSH (`git@github.com:owner/repo.git`) and URL-style
/// (`https://github.com/owner/repo.git`, `ssh://git@host/owner/repo.git`).
/// Pure string parsing — no network, no `git` invocation.
pub(crate) fn parse_remote(url: &str) -> (Option<String>, Option<String>) {
    let url = url.trim();
    if url.is_empty() {
        return (None, None);
    }

    let non_empty = |s: &str| (!s.is_empty()).then(|| s.to_string());

    // SCP-like syntax has no scheme and uses `:` to separate host from path.
    if !url.contains("://") {
        return match url.split_once(':') {
            Some((authority, path)) => {
                let host = authority.rsplit('@').next().unwrap_or(authority);
                let owner = path.trim_start_matches('/').split('/').next().unwrap_or("");
                (non_empty(host), non_empty(owner))
            }
            None => (None, None),
        };
    }

    // scheme://[user@]host[:port]/owner/repo(.git)
    let after = url.split_once("://").map_or(url, |(_, rest)| rest);
    let (authority, path) = after.split_once('/').unwrap_or((after, ""));
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_port.split(':').next().unwrap_or(host_port);
    let owner = path.split('/').next().unwrap_or("");
    (non_empty(host), non_empty(owner))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn ctx(path: &str, name: &str, remote: Option<&str>) -> EvaluationContext {
        EvaluationContext::from_parts(path, name, remote)
    }

    #[test]
    fn parses_https_remote() {
        let (host, owner) = parse_remote("https://github.com/my-company/api.git");
        assert_eq!(host.as_deref(), Some("github.com"));
        assert_eq!(owner.as_deref(), Some("my-company"));
    }

    #[test]
    fn parses_scp_ssh_remote() {
        let (host, owner) = parse_remote("git@github.com:open-source/tool.git");
        assert_eq!(host.as_deref(), Some("github.com"));
        assert_eq!(owner.as_deref(), Some("open-source"));
    }

    #[test]
    fn parses_ssh_scheme_remote() {
        let (host, owner) = parse_remote("ssh://git@gitlab.com:22/group/proj.git");
        assert_eq!(host.as_deref(), Some("gitlab.com"));
        assert_eq!(owner.as_deref(), Some("group"));
    }

    #[test]
    fn empty_remote_yields_none() {
        assert_eq!(parse_remote("   "), (None, None));
    }

    #[test]
    fn path_contains_matches_case_insensitively() {
        let cond = RuleCondition {
            subject: RuleSubject::RepoPath,
            operator: RuleOperator::Contains,
            value: "/Company/".into(),
        };
        assert!(cond
            .matches(&ctx("/home/dev/company/api", "api", None))
            .is_some());
        assert!(cond
            .matches(&ctx("/home/dev/oss/api", "api", None))
            .is_none());
    }

    #[test]
    fn name_starts_with_matches() {
        let cond = RuleCondition {
            subject: RuleSubject::RepoName,
            operator: RuleOperator::StartsWith,
            value: "api-".into(),
        };
        assert!(cond.matches(&ctx("/x", "api-gateway", None)).is_some());
        assert!(cond.matches(&ctx("/x", "web-app", None)).is_none());
    }

    #[test]
    fn remote_host_equals_matches() {
        let cond = RuleCondition {
            subject: RuleSubject::RemoteHost,
            operator: RuleOperator::Equals,
            value: "github.com".into(),
        };
        let matched = cond.matches(&ctx("/x", "api", Some("git@github.com:acme/api.git")));
        assert!(matched.is_some());
    }

    #[test]
    fn owner_equals_matches() {
        let cond = RuleCondition {
            subject: RuleSubject::Owner,
            operator: RuleOperator::Equals,
            value: "open-source".into(),
        };
        assert!(cond
            .matches(&ctx(
                "/x",
                "tool",
                Some("git@github.com:open-source/tool.git")
            ))
            .is_some());
    }

    #[test]
    fn missing_field_does_not_match() {
        let cond = RuleCondition {
            subject: RuleSubject::RemoteUrl,
            operator: RuleOperator::Contains,
            value: "github".into(),
        };
        assert!(cond.matches(&ctx("/x", "api", None)).is_none());
    }

    #[test]
    fn empty_value_never_matches() {
        let cond = RuleCondition {
            subject: RuleSubject::RepoPath,
            operator: RuleOperator::Contains,
            value: "   ".into(),
        };
        assert!(cond.matches(&ctx("/anything", "x", None)).is_none());
    }

    #[test]
    fn host_and_owner_allow_only_equals() {
        assert!(RuleSubject::RemoteHost.allows(RuleOperator::Equals));
        assert!(!RuleSubject::RemoteHost.allows(RuleOperator::Contains));
        assert!(RuleSubject::Owner.allows(RuleOperator::Equals));
        assert!(!RuleSubject::Owner.allows(RuleOperator::StartsWith));
        assert!(RuleSubject::RepoPath.allows(RuleOperator::Contains));
    }
}
