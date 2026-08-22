use crate::domain::identity::Identity;
use crate::domain::ports::GitConfigBackend;
use crate::error::AppError;
use bstr::BStr;
use gix_config::File;
use std::path::{Path, PathBuf};

pub(crate) struct GixGitConfig;

fn home_gitconfig() -> Result<PathBuf, AppError> {
    dirs_next::home_dir()
        .map(|h| h.join(".gitconfig"))
        .ok_or_else(|| AppError::GitConfig("could not find home directory".into()))
}

/// Resolve the real `.git` directory for a repo root, following the `gitdir:`
/// pointer used by worktrees and submodules, so the local config we edit is the
/// one Git actually reads.
fn resolve_git_dir(git_root: &Path) -> Result<PathBuf, AppError> {
    let dotgit = git_root.join(".git");
    let meta = std::fs::symlink_metadata(&dotgit)
        .map_err(|e| AppError::GitConfig(format!("{}: {e}", dotgit.display())))?;
    if meta.is_dir() {
        return Ok(dotgit);
    }
    let content =
        std::fs::read_to_string(&dotgit).map_err(|e| AppError::GitConfig(e.to_string()))?;
    let rest = content
        .trim()
        .strip_prefix("gitdir:")
        .ok_or_else(|| {
            AppError::GitConfig(format!(
                "{} is not a valid git dir pointer",
                dotgit.display()
            ))
        })?
        .trim();
    let pointer = PathBuf::from(rest);
    Ok(if pointer.is_absolute() {
        pointer
    } else {
        git_root.join(pointer)
    })
}

fn open(path: &Path, source: gix_config::Source) -> Result<File<'static>, AppError> {
    if path.exists() {
        gix_config::File::from_path_no_includes(path.to_path_buf(), source)
            .map_err(|e| AppError::GitConfig(e.to_string()))
    } else {
        let meta = gix_config::file::Metadata::from(source);
        Ok(gix_config::File::new(meta))
    }
}

/// Set (or, when `value` is `None`, remove) a single key inside an open config
/// file, replacing any existing occurrences.
fn upsert(
    f: &mut File<'static>,
    section: &str,
    subsection: Option<&str>,
    key: &str,
    value: Option<&str>,
) -> Result<(), AppError> {
    // Owned key avoids lifetime conflict with File<'static>'s invariant 'event param.
    let value_name: gix_config::parse::section::ValueName<'static> = key
        .to_string()
        .try_into()
        .map_err(|_| AppError::GitConfig(format!("invalid key: {key}")))?;

    let subsection = subsection.map(BStr::new);
    let mut sec = f
        .section_mut_or_create_new(section, subsection)
        .map_err(|e| AppError::GitConfig(e.to_string()))?;

    while sec.remove(key).is_some() {}

    if let Some(v) = value {
        let normalized = gix_config::value::normalize_bstr(v);
        sec.push(value_name, Some(normalized.as_ref()));
    }
    Ok(())
}

fn write_file(path: &Path, f: &File<'static>) -> Result<(), AppError> {
    std::fs::write(path, f.to_string()).map_err(|e| AppError::GitConfig(e.to_string()))
}

fn get_global(section: &str, key: &str) -> Result<Option<String>, AppError> {
    let path = home_gitconfig()?;
    match open(&path, gix_config::Source::User) {
        Ok(f) => Ok(f
            .raw_value_by(section, None, key)
            .ok()
            .map(|v| String::from_utf8_lossy(v.as_ref()).into_owned())),
        Err(_) => Ok(None),
    }
}

fn set_global(section: &str, key: &str, value: Option<&str>) -> Result<(), AppError> {
    let path = home_gitconfig()?;
    let mut f = open(&path, gix_config::Source::User)?;
    upsert(&mut f, section, None, key, value)?;
    write_file(&path, &f)
}

/// Read a single key from a repository's **local** config only (no fallback).
/// `Ok(None)` when the repo, its config, or the key is absent — a missing local
/// value is a normal case, not an error.
fn get_local(git_root: &Path, section: &str, key: &str) -> Result<Option<String>, AppError> {
    let Ok(git_dir) = resolve_git_dir(git_root) else {
        return Ok(None);
    };
    let config_path = git_dir.join("config");
    match open(&config_path, gix_config::Source::Local) {
        Ok(f) => Ok(f
            .raw_value_by(section, None, key)
            .ok()
            .map(|v| String::from_utf8_lossy(v.as_ref()).into_owned())),
        Err(_) => Ok(None),
    }
}

impl GitConfigBackend for GixGitConfig {
    fn get_global_name(&self) -> Result<Option<String>, AppError> {
        get_global("user", "name")
    }

    fn get_global_email(&self) -> Result<Option<String>, AppError> {
        get_global("user", "email")
    }

    fn get_global_signing_key(&self) -> Result<Option<String>, AppError> {
        get_global("user", "signingkey")
    }

    fn set_global_name(&self, name: &str) -> Result<(), AppError> {
        set_global("user", "name", Some(name))
    }

    fn set_global_email(&self, email: &str) -> Result<(), AppError> {
        set_global("user", "email", Some(email))
    }

    fn set_global_signing_key(&self, key: Option<&str>) -> Result<(), AppError> {
        set_global("user", "signingkey", key)
    }

    fn get_local_name(&self, git_root: &Path) -> Result<Option<String>, AppError> {
        match get_local(git_root, "user", "name")? {
            Some(v) => Ok(Some(v)),
            None => self.get_global_name(),
        }
    }

    fn get_local_email(&self, git_root: &Path) -> Result<Option<String>, AppError> {
        match get_local(git_root, "user", "email")? {
            Some(v) => Ok(Some(v)),
            None => self.get_global_email(),
        }
    }

    fn set_local_identity(&self, git_root: &Path, identity: &Identity) -> Result<(), AppError> {
        let config_path = resolve_git_dir(git_root)?.join("config");
        // Open, rewrite all three keys, and persist once so the identity lands
        // atomically rather than through three separate reads/writes.
        let mut f = open(&config_path, gix_config::Source::Local)?;
        upsert(&mut f, "user", None, "name", Some(&identity.name))?;
        upsert(&mut f, "user", None, "email", Some(&identity.email))?;
        upsert(
            &mut f,
            "user",
            None,
            "signingkey",
            identity.signing_key.as_deref(),
        )?;
        write_file(&config_path, &f)
    }

    fn set_local_use_http_path(
        &self,
        git_root: &Path,
        host: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let config_path = resolve_git_dir(git_root)?.join("config");
        let mut f = open(&config_path, gix_config::Source::Local)?;
        let subsection = format!("https://{}", host.to_ascii_lowercase());
        let value = if enabled { Some("true") } else { None };
        upsert(
            &mut f,
            "credential",
            Some(&subsection),
            "useHttpPath",
            value,
        )?;
        write_file(&config_path, &f)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;

    fn identity(name: &str, email: &str, key: Option<&str>) -> Identity {
        Identity {
            name: name.into(),
            email: email.into(),
            signing_key: key.map(Into::into),
        }
    }

    #[test]
    fn set_local_identity_writes_repo_config() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir_all(&git).unwrap();

        GixGitConfig
            .set_local_identity(tmp.path(), &identity("Work", "work@co.com", Some("KEY1")))
            .unwrap();

        let written = fs::read_to_string(git.join("config")).unwrap();
        assert!(written.contains("name = Work"));
        assert!(written.contains("email = work@co.com"));
        assert!(written.contains("signingkey = KEY1"));
    }

    #[test]
    fn set_local_identity_replaces_previous_values() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(
            git.join("config"),
            "[user]\n\tname = Old\n\temail = old@x.com\n\tsigningkey = OLD\n",
        )
        .unwrap();

        GixGitConfig
            .set_local_identity(tmp.path(), &identity("New", "new@x.com", None))
            .unwrap();

        let written = fs::read_to_string(git.join("config")).unwrap();
        assert!(written.contains("name = New"));
        assert!(written.contains("email = new@x.com"));
        assert!(!written.contains("Old"));
        assert!(!written.contains("old@x.com"));
        // signing_key None clears the previous local signingkey.
        assert!(!written.contains("signingkey"));
    }

    #[test]
    fn set_local_use_http_path_writes_scoped_key() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir_all(&git).unwrap();

        GixGitConfig
            .set_local_use_http_path(tmp.path(), "github.com", true)
            .unwrap();

        let written = fs::read_to_string(git.join("config")).unwrap();
        assert!(written.contains("[credential \"https://github.com\"]"));
        assert!(written.contains("useHttpPath = true"));
    }

    #[test]
    fn set_local_use_http_path_disabled_removes_key() {
        let tmp = tempfile::tempdir().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(
            git.join("config"),
            "[credential \"https://github.com\"]\n\tuseHttpPath = true\n",
        )
        .unwrap();

        GixGitConfig
            .set_local_use_http_path(tmp.path(), "github.com", false)
            .unwrap();

        let written = fs::read_to_string(git.join("config")).unwrap();
        assert!(!written.contains("useHttpPath"));
    }

    #[test]
    fn set_local_identity_follows_gitdir_pointer() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real-git");
        fs::create_dir_all(&real).unwrap();
        let wt = tmp.path().join("wt");
        fs::create_dir_all(&wt).unwrap();
        fs::write(wt.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();

        GixGitConfig
            .set_local_identity(&wt, &identity("WT", "wt@x.com", None))
            .unwrap();

        let written = fs::read_to_string(real.join("config")).unwrap();
        assert!(written.contains("email = wt@x.com"));
    }
}
