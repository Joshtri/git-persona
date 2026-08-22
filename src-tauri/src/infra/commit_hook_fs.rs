use crate::domain::{
    commit_guard::{ExpectedIdentity, HookState},
    ports::CommitHookInstaller,
    settings::GuardMode,
};
use crate::error::AppError;
use bstr::BStr;
use gix_config::File;
use std::path::{Path, PathBuf};

/// Marks a `pre-commit` file as GitPersona-managed. Present on the second line so
/// the shebang stays first. Used to distinguish our hook from a foreign one and
/// to make install/repair/uninstall idempotent.
const SENTINEL: &str = "# gitpersona-commit-guard v1";
const HOOK_NAME: &str = "pre-commit";
const BACKUP_NAME: &str = "pre-commit.gitpersona-bak";

/// The managed hook script. POSIX `sh`, self-contained: it reads the expected
/// identity and mode from the repository's local Git config and compares against
/// the current `user.email`. Requires no running app process and touches no
/// secrets. A chained (backed-up) foreign hook runs first.
const HOOK_SCRIPT: &str = r#"#!/bin/sh
# gitpersona-commit-guard v1
# Managed by GitPersona. Verifies the active Git identity before a commit.
# Do not edit — reinstall from GitPersona to update.

_gp_dir=$(dirname "$0")
if [ -x "$_gp_dir/pre-commit.gitpersona-bak" ]; then
  "$_gp_dir/pre-commit.gitpersona-bak" "$@" || exit $?
fi

[ "$(git config --get gitpersona.guard 2>/dev/null)" = "1" ] || exit 0

expected_email=$(git config --get gitpersona.guardemail 2>/dev/null)
expected_name=$(git config --get gitpersona.guardname 2>/dev/null)
mode=$(git config --get gitpersona.guardmode 2>/dev/null)

# No expected identity recorded → allow (never block on the unknown).
[ -n "$expected_email" ] || exit 0

actual_email=$(git config user.email 2>/dev/null)
actual_name=$(git config user.name 2>/dev/null)

if [ "$actual_email" = "$expected_email" ]; then
  exit 0
fi

echo "GitPersona Commit Guard: Git identity does not match this repository's profile." 1>&2
echo "  expected: $expected_name <$expected_email>" 1>&2
echo "  current:  $actual_name <$actual_email>" 1>&2
if [ "$mode" = "block" ]; then
  echo "  Commit blocked. Switch to the correct GitPersona profile, then retry." 1>&2
  echo "  To override once: git commit --no-verify" 1>&2
  exit 1
fi
echo "  Warning only — the commit will proceed." 1>&2
exit 0
"#;

/// Filesystem-backed [`CommitHookInstaller`]. All writes go through `std::fs`
/// (no Tauri fs plugin, matching the other infra adapters), and the local-config
/// marker is written with `gix-config`.
pub(crate) struct FsCommitHookInstaller;

/// Resolve the real `.git` directory, following the `gitdir:` pointer used by
/// worktrees and submodules so the hook lands where Git actually looks.
fn resolve_git_dir(git_root: &Path) -> Result<PathBuf, AppError> {
    let dotgit = git_root.join(".git");
    let meta = std::fs::symlink_metadata(&dotgit)
        .map_err(|e| AppError::Io(format!("{}: {e}", dotgit.display())))?;
    if meta.is_dir() {
        return Ok(dotgit);
    }
    let content = std::fs::read_to_string(&dotgit).map_err(|e| AppError::Io(e.to_string()))?;
    let rest = content
        .trim()
        .strip_prefix("gitdir:")
        .ok_or_else(|| AppError::Io(format!("{} is not a git dir pointer", dotgit.display())))?
        .trim();
    let pointer = PathBuf::from(rest);
    Ok(if pointer.is_absolute() {
        pointer
    } else {
        git_root.join(pointer)
    })
}

fn config_path(git_dir: &Path) -> PathBuf {
    git_dir.join("config")
}

fn open_local(path: &Path) -> Result<File<'static>, AppError> {
    if path.exists() {
        gix_config::File::from_path_no_includes(path.to_path_buf(), gix_config::Source::Local)
            .map_err(|e| AppError::GitConfig(e.to_string()))
    } else {
        Ok(gix_config::File::new(gix_config::file::Metadata::from(
            gix_config::Source::Local,
        )))
    }
}

fn upsert(f: &mut File<'static>, key: &str, value: Option<&str>) -> Result<(), AppError> {
    let value_name: gix_config::parse::section::ValueName<'static> = key
        .to_string()
        .try_into()
        .map_err(|_| AppError::GitConfig(format!("invalid key: {key}")))?;
    let mut sec = f
        .section_mut_or_create_new("gitpersona", None::<&BStr>)
        .map_err(|e| AppError::GitConfig(e.to_string()))?;
    while sec.remove(key).is_some() {}
    if let Some(v) = value {
        let normalized = gix_config::value::normalize_bstr(v);
        sec.push(value_name, Some(normalized.as_ref()));
    }
    Ok(())
}

fn read_key(f: &File<'static>, key: &str) -> Option<String> {
    f.raw_value_by("gitpersona", None, key)
        .ok()
        .map(|v| String::from_utf8_lossy(v.as_ref()).into_owned())
}

/// Whether `core.hooksPath` is set (local or global). When it is, a hook in
/// `.git/hooks` would be ignored, so we must not pretend to have installed one.
fn hooks_path_overridden(git_dir: &Path) -> bool {
    // Local config only — a global override is the user's deliberate choice and
    // reading their home config here would broaden our surface; the local check
    // covers the case GitPersona itself could otherwise stomp.
    if let Ok(f) = open_local(&config_path(git_dir)) {
        if let Ok(v) = f.raw_value_by("core", None, "hookspath") {
            return !String::from_utf8_lossy(v.as_ref()).trim().is_empty();
        }
    }
    false
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| AppError::Io(e.to_string()))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn make_executable(_path: &Path) -> Result<(), AppError> {
    // On Windows the executable bit is not used; Git for Windows runs hooks via
    // its bundled `sh`, which keys off the shebang line.
    Ok(())
}

fn is_managed(hook_path: &Path) -> bool {
    std::fs::read_to_string(hook_path).is_ok_and(|c| c.contains(SENTINEL))
}

fn write_hook(hook_path: &Path) -> Result<(), AppError> {
    std::fs::write(hook_path, HOOK_SCRIPT).map_err(|e| AppError::Io(e.to_string()))?;
    make_executable(hook_path)
}

impl CommitHookInstaller for FsCommitHookInstaller {
    fn hook_state(&self, git_root: &Path) -> Result<HookState, AppError> {
        let git_dir = resolve_git_dir(git_root)?;
        if hooks_path_overridden(&git_dir) {
            return Ok(HookState::Unsupported);
        }
        let hook = git_dir.join("hooks").join(HOOK_NAME);
        if !hook.exists() {
            return Ok(HookState::NotInstalled);
        }
        if is_managed(&hook) {
            let backup = git_dir.join("hooks").join(BACKUP_NAME);
            return Ok(if backup.exists() {
                HookState::ManagedChained
            } else {
                HookState::Managed
            });
        }
        Ok(HookState::Foreign)
    }

    fn install(
        &self,
        git_root: &Path,
        expected: &ExpectedIdentity,
    ) -> Result<HookState, AppError> {
        let git_dir = resolve_git_dir(git_root)?;
        if hooks_path_overridden(&git_dir) {
            return Err(AppError::Unsupported(
                "this repository overrides core.hooksPath; install the Commit Guard hook manually"
                    .into(),
            ));
        }
        let hooks_dir = git_dir.join("hooks");
        std::fs::create_dir_all(&hooks_dir).map_err(|e| AppError::Io(e.to_string()))?;
        let hook = hooks_dir.join(HOOK_NAME);
        let backup = hooks_dir.join(BACKUP_NAME);

        // Preserve a pre-existing foreign hook by backing it up once, so the
        // managed hook can chain it. Never overwrite an existing backup (that
        // would clobber the user's original on a repair).
        let mut chained = backup.exists();
        if hook.exists() && !is_managed(&hook) && !backup.exists() {
            std::fs::rename(&hook, &backup).map_err(|e| AppError::Io(e.to_string()))?;
            make_executable(&backup)?;
            chained = true;
        }

        write_hook(&hook)?;
        write_marker(&git_dir, expected)?;

        Ok(if chained {
            HookState::ManagedChained
        } else {
            HookState::Managed
        })
    }

    fn uninstall(&self, git_root: &Path) -> Result<(), AppError> {
        let git_dir = resolve_git_dir(git_root)?;
        let hooks_dir = git_dir.join("hooks");
        let hook = hooks_dir.join(HOOK_NAME);
        let backup = hooks_dir.join(BACKUP_NAME);

        // Only remove a hook we own — never delete a foreign hook.
        if hook.exists() && is_managed(&hook) {
            std::fs::remove_file(&hook).map_err(|e| AppError::Io(e.to_string()))?;
        }
        // Restore a chained foreign hook.
        if backup.exists() {
            std::fs::rename(&backup, &hook).map_err(|e| AppError::Io(e.to_string()))?;
            make_executable(&hook)?;
        }
        clear_marker(&git_dir)?;
        Ok(())
    }

    fn read_marker(&self, git_root: &Path) -> Result<Option<ExpectedIdentity>, AppError> {
        let git_dir = resolve_git_dir(git_root)?;
        let f = open_local(&config_path(&git_dir))?;
        if read_key(&f, "guard").as_deref() != Some("1") {
            return Ok(None);
        }
        let email = match read_key(&f, "guardemail") {
            Some(e) if !e.is_empty() => e,
            _ => return Ok(None),
        };
        let name = read_key(&f, "guardname").unwrap_or_default();
        let mode = match read_key(&f, "guardmode").as_deref() {
            Some("block") => GuardMode::Block,
            _ => GuardMode::Warn,
        };
        Ok(Some(ExpectedIdentity { name, email, mode }))
    }
}

/// Write the expected-identity marker into the repository's local Git config.
fn write_marker(git_dir: &Path, expected: &ExpectedIdentity) -> Result<(), AppError> {
    let path = config_path(git_dir);
    let mut f = open_local(&path)?;
    upsert(&mut f, "guard", Some("1"))?;
    upsert(&mut f, "guardname", Some(&expected.name))?;
    upsert(&mut f, "guardemail", Some(&expected.email))?;
    upsert(
        &mut f,
        "guardmode",
        Some(match expected.mode {
            GuardMode::Warn => "warn",
            GuardMode::Block => "block",
        }),
    )?;
    std::fs::write(&path, f.to_string()).map_err(|e| AppError::Io(e.to_string()))
}

/// Remove the guard marker keys from the repository's local Git config.
fn clear_marker(git_dir: &Path) -> Result<(), AppError> {
    let path = config_path(git_dir);
    if !path.exists() {
        return Ok(());
    }
    let mut f = open_local(&path)?;
    for key in ["guard", "guardname", "guardemail", "guardmode"] {
        upsert(&mut f, key, None)?;
    }
    std::fs::write(&path, f.to_string()).map_err(|e| AppError::Io(e.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;

    fn init_repo(base: &Path) {
        fs::create_dir_all(base.join(".git").join("hooks")).unwrap();
    }

    fn expected() -> ExpectedIdentity {
        ExpectedIdentity {
            name: "Work".into(),
            email: "work@co.com".into(),
            mode: GuardMode::Block,
        }
    }

    #[test]
    fn fresh_install_is_managed_and_writes_marker() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let installer = FsCommitHookInstaller;

        assert_eq!(
            installer.hook_state(tmp.path()).unwrap(),
            HookState::NotInstalled
        );
        let state = installer.install(tmp.path(), &expected()).unwrap();
        assert_eq!(state, HookState::Managed);

        let hook = tmp.path().join(".git/hooks/pre-commit");
        let script = fs::read_to_string(&hook).unwrap();
        assert!(script.contains(SENTINEL));

        let marker = installer.read_marker(tmp.path()).unwrap().unwrap();
        assert_eq!(marker.email, "work@co.com");
        assert_eq!(marker.mode, GuardMode::Block);
    }

    #[test]
    fn install_over_foreign_hook_chains_and_backs_up() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let hook = tmp.path().join(".git/hooks/pre-commit");
        fs::write(&hook, "#!/bin/sh\necho existing\n").unwrap();

        let installer = FsCommitHookInstaller;
        assert_eq!(installer.hook_state(tmp.path()).unwrap(), HookState::Foreign);

        let state = installer.install(tmp.path(), &expected()).unwrap();
        assert_eq!(state, HookState::ManagedChained);

        // Our hook is now in place, and the original is preserved as a backup.
        assert!(is_managed(&hook));
        let backup = tmp.path().join(".git/hooks/pre-commit.gitpersona-bak");
        assert!(fs::read_to_string(&backup).unwrap().contains("echo existing"));
    }

    #[test]
    fn uninstall_restores_foreign_hook_and_clears_marker() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let hook = tmp.path().join(".git/hooks/pre-commit");
        fs::write(&hook, "#!/bin/sh\necho existing\n").unwrap();

        let installer = FsCommitHookInstaller;
        installer.install(tmp.path(), &expected()).unwrap();
        installer.uninstall(tmp.path()).unwrap();

        // The original foreign hook is restored; our marker is gone.
        assert!(fs::read_to_string(&hook).unwrap().contains("echo existing"));
        assert!(!is_managed(&hook));
        assert!(installer.read_marker(tmp.path()).unwrap().is_none());
        assert!(!tmp
            .path()
            .join(".git/hooks/pre-commit.gitpersona-bak")
            .exists());
    }

    #[test]
    fn uninstall_removes_solo_managed_hook() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let installer = FsCommitHookInstaller;
        installer.install(tmp.path(), &expected()).unwrap();
        installer.uninstall(tmp.path()).unwrap();
        assert!(!tmp.path().join(".git/hooks/pre-commit").exists());
        assert_eq!(
            installer.hook_state(tmp.path()).unwrap(),
            HookState::NotInstalled
        );
    }

    #[test]
    fn hooks_path_override_reports_unsupported() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        fs::write(
            tmp.path().join(".git/config"),
            "[core]\n\thooksPath = /custom/hooks\n",
        )
        .unwrap();
        let installer = FsCommitHookInstaller;
        assert_eq!(
            installer.hook_state(tmp.path()).unwrap(),
            HookState::Unsupported
        );
        assert!(installer.install(tmp.path(), &expected()).is_err());
    }

    #[test]
    fn repair_preserves_existing_backup() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let hook = tmp.path().join(".git/hooks/pre-commit");
        fs::write(&hook, "#!/bin/sh\necho original\n").unwrap();

        let installer = FsCommitHookInstaller;
        installer.install(tmp.path(), &expected()).unwrap();
        // Re-install (repair) must not clobber the preserved original.
        installer.install(tmp.path(), &expected()).unwrap();

        let backup = tmp.path().join(".git/hooks/pre-commit.gitpersona-bak");
        assert!(fs::read_to_string(&backup).unwrap().contains("echo original"));
    }
}
