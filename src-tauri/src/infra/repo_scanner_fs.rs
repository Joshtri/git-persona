use crate::domain::{ports::RepoScanner, repo::ScanProgress};
use crate::error::AppError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directory names never descended into during a scan.
const SKIP_DIRS: &[&str] = &["node_modules", "target", "dist", "build", ".git"];

/// Guard against pathological trees; scans stop past this depth.
const MAX_DEPTH: usize = 12;

/// Emit progress roughly every this many directories visited.
const PROGRESS_EVERY: usize = 64;

pub(crate) struct FsRepoScanner {
    skip: HashSet<&'static str>,
    max_depth: usize,
}

impl Default for FsRepoScanner {
    fn default() -> Self {
        Self {
            skip: SKIP_DIRS.iter().copied().collect(),
            max_depth: MAX_DEPTH,
        }
    }
}

fn is_git_repo(dir: &Path) -> bool {
    // `.git` is a directory in a normal clone, a file in worktrees/submodules.
    dir.join(".git").exists()
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Strip the Windows extended-length prefix (`\\?\`) that `canonicalize` adds,
/// so stored/displayed/copied paths look normal. No-op on other platforms.
fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        s.into_owned()
    }
}

impl FsRepoScanner {
    fn scan_root(
        &self,
        root: &Path,
        found: &mut Vec<String>,
        seen: &mut HashSet<PathBuf>,
        state: &mut ScanState<'_>,
    ) {
        // (path, depth) DFS stack. Symlinks are never pushed, so cycles are impossible.
        let mut stack: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

        while let Some((dir, depth)) = stack.pop() {
            state.tick();

            if is_git_repo(&dir) {
                if let Ok(canonical) = dir.canonicalize() {
                    if seen.insert(canonical.clone()) {
                        found.push(normalize_path(&canonical));
                        state.found_repos += 1;
                    }
                }
                // Do not descend into a repository — keeps scans fast and avoids
                // treating vendored/submodule checkouts as separate top-level repos.
                continue;
            }

            if depth >= self.max_depth {
                continue;
            }

            // Permission denied / transient IO: skip this directory, keep going.
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_dir() || file_type.is_symlink() {
                    continue;
                }
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if self.skip.contains(name.as_ref()) || is_hidden(&name) {
                    continue;
                }
                stack.push((entry.path(), depth + 1));
            }
        }
    }
}

struct ScanState<'a> {
    scanned_dirs: usize,
    found_repos: usize,
    on_progress: &'a (dyn Fn(ScanProgress) + Send + Sync),
}

impl ScanState<'_> {
    fn tick(&mut self) {
        self.scanned_dirs += 1;
        if self.scanned_dirs.is_multiple_of(PROGRESS_EVERY) {
            (self.on_progress)(ScanProgress {
                scanned_dirs: u32::try_from(self.scanned_dirs).unwrap_or(u32::MAX),
                found_repos: u32::try_from(self.found_repos).unwrap_or(u32::MAX),
            });
        }
    }
}

impl RepoScanner for FsRepoScanner {
    fn scan(
        &self,
        roots: &[String],
        on_progress: &(dyn Fn(ScanProgress) + Send + Sync),
    ) -> Result<Vec<String>, AppError> {
        let mut found: Vec<String> = Vec::new();
        let mut seen: HashSet<PathBuf> = HashSet::new();

        for root in roots {
            let path = PathBuf::from(root);
            if !path.is_dir() {
                return Err(AppError::Validation(format!(
                    "\"{root}\" is not a directory"
                )));
            }
            let mut state = ScanState {
                scanned_dirs: 0,
                found_repos: found.len(),
                on_progress,
            };
            self.scan_root(&path, &mut found, &mut seen, &mut state);
        }

        on_progress(ScanProgress {
            scanned_dirs: 0,
            found_repos: u32::try_from(found.len()).unwrap_or(u32::MAX),
        });
        Ok(found)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;

    fn mk_repo(base: &Path, rel: &str) {
        let dir = base.join(rel);
        fs::create_dir_all(dir.join(".git")).unwrap();
    }

    fn noop_progress() -> impl Fn(ScanProgress) + Send + Sync {
        |_| {}
    }

    #[test]
    fn detects_git_repositories() {
        let tmp = tempfile::tempdir().unwrap();
        mk_repo(tmp.path(), "alpha");
        mk_repo(tmp.path(), "nested/beta");
        let scanner = FsRepoScanner::default();
        let found = scanner
            .scan(
                &[tmp.path().to_string_lossy().into_owned()],
                &noop_progress(),
            )
            .unwrap();
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn skips_node_modules_and_build_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        mk_repo(tmp.path(), "app");
        // A repo buried inside node_modules must not be discovered.
        mk_repo(tmp.path(), "app/node_modules/some-pkg");
        mk_repo(tmp.path(), "app/target/vendored");
        let scanner = FsRepoScanner::default();
        let found = scanner
            .scan(
                &[tmp.path().to_string_lossy().into_owned()],
                &noop_progress(),
            )
            .unwrap();
        // Only "app" — its subtree is pruned once the repo root is found anyway.
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn does_not_descend_into_repositories() {
        let tmp = tempfile::tempdir().unwrap();
        mk_repo(tmp.path(), "outer");
        mk_repo(tmp.path(), "outer/inner");
        let scanner = FsRepoScanner::default();
        let found = scanner
            .scan(
                &[tmp.path().to_string_lossy().into_owned()],
                &noop_progress(),
            )
            .unwrap();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn deduplicates_overlapping_roots() {
        let tmp = tempfile::tempdir().unwrap();
        mk_repo(tmp.path(), "repo");
        let root = tmp.path().to_string_lossy().into_owned();
        let sub = tmp.path().join("repo").to_string_lossy().into_owned();
        let scanner = FsRepoScanner::default();
        let found = scanner.scan(&[root, sub], &noop_progress()).unwrap();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn normalize_strips_extended_length_prefix() {
        assert_eq!(
            normalize_path(Path::new(r"\\?\D:\code\project")),
            r"D:\code\project"
        );
        assert_eq!(
            normalize_path(Path::new(r"\\?\UNC\server\share\repo")),
            r"\\server\share\repo"
        );
        assert_eq!(
            normalize_path(Path::new("/home/alex/repo")),
            "/home/alex/repo"
        );
    }

    #[test]
    fn errors_on_non_directory_root() {
        let scanner = FsRepoScanner::default();
        let err = scanner
            .scan(
                &["/definitely/not/a/real/path/xyzzy".into()],
                &noop_progress(),
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
