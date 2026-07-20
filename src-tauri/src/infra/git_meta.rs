use crate::domain::{ports::GitMetadataReader, repo::RepoMetadata};
use crate::error::AppError;
use std::path::{Path, PathBuf};

pub(crate) struct FsGitMetadataReader;

/// Resolve the real git directory for a repo root.
///
/// `.git` is a directory in a normal clone but a `gitdir: <path>` pointer file
/// in worktrees and submodules — this follows that pointer.
fn resolve_git_dir(git_root: &Path) -> Result<PathBuf, AppError> {
    let dotgit = git_root.join(".git");
    let meta = std::fs::symlink_metadata(&dotgit)
        .map_err(|e| AppError::Io(format!("{}: {e}", git_root.display())))?;
    if meta.is_dir() {
        return Ok(dotgit);
    }
    let content = std::fs::read_to_string(&dotgit).map_err(|e| AppError::Io(e.to_string()))?;
    let rest = content
        .trim()
        .strip_prefix("gitdir:")
        .ok_or_else(|| AppError::Validation("malformed .git pointer file".into()))?
        .trim();
    let p = PathBuf::from(rest);
    Ok(if p.is_absolute() { p } else { git_root.join(p) })
}

fn read_branch(git_dir: &Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    // "ref: refs/heads/<branch>" when on a branch; a raw SHA when detached.
    let rest = head.strip_prefix("ref:")?.trim();
    let branch = rest.strip_prefix("refs/heads/").unwrap_or(rest);
    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

fn read_remote_origin(git_dir: &Path) -> Option<String> {
    let config_path = git_dir.join("config");
    if !config_path.exists() {
        return None;
    }
    let file =
        gix_config::File::from_path_no_includes(config_path, gix_config::Source::Local).ok()?;
    file.string_by("remote", Some("origin".into()), "url")
        .map(|v| v.to_string())
}

impl GitMetadataReader for FsGitMetadataReader {
    fn read(&self, git_root: &Path) -> Result<RepoMetadata, AppError> {
        let git_dir = resolve_git_dir(git_root)?;
        Ok(RepoMetadata {
            active_branch: read_branch(&git_dir),
            remote_origin: read_remote_origin(&git_dir),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;

    fn init_repo(base: &Path, head: &str, config: Option<&str>) {
        let git = base.join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(git.join("HEAD"), head).unwrap();
        if let Some(cfg) = config {
            fs::write(git.join("config"), cfg).unwrap();
        }
    }

    #[test]
    fn reads_branch_and_remote() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(
            tmp.path(),
            "ref: refs/heads/main\n",
            Some("[remote \"origin\"]\n\turl = https://github.com/foo/bar.git\n"),
        );
        let meta = FsGitMetadataReader.read(tmp.path()).unwrap();
        assert_eq!(meta.active_branch.as_deref(), Some("main"));
        assert_eq!(
            meta.remote_origin.as_deref(),
            Some("https://github.com/foo/bar.git")
        );
    }

    #[test]
    fn nested_branch_name_is_preserved() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path(), "ref: refs/heads/feature/auth\n", None);
        let meta = FsGitMetadataReader.read(tmp.path()).unwrap();
        assert_eq!(meta.active_branch.as_deref(), Some("feature/auth"));
        assert_eq!(meta.remote_origin, None);
    }

    #[test]
    fn detached_head_has_no_branch() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(
            tmp.path(),
            "9f8e7d6c5b4a39281706f5e4d3c2b1a09f8e7d6c\n",
            None,
        );
        let meta = FsGitMetadataReader.read(tmp.path()).unwrap();
        assert_eq!(meta.active_branch, None);
    }

    #[test]
    fn resolves_gitdir_pointer_file() {
        let tmp = tempfile::tempdir().unwrap();
        // Real git dir lives elsewhere; the repo root has a `.git` file pointing to it.
        let real = tmp.path().join("realgit");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("HEAD"), "ref: refs/heads/wt\n").unwrap();
        let repo = tmp.path().join("wtree");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();
        let meta = FsGitMetadataReader.read(&repo).unwrap();
        assert_eq!(meta.active_branch.as_deref(), Some("wt"));
    }

    #[test]
    fn missing_git_dir_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let err = FsGitMetadataReader.read(tmp.path()).unwrap_err();
        assert!(matches!(err, AppError::Io(_)));
    }
}
