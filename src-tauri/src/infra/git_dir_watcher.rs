use crate::domain::ports::WorkspaceWatcher;
use crate::error::AppError;
use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Collapse a burst of Git operations (e.g. a rebase touching many refs) into a
/// single activation.
const DEBOUNCE: Duration = Duration::from_millis(400);

/// The `.git` entries whose change signals that a repository became "current":
/// `HEAD`/`ORIG_HEAD`/`MERGE_HEAD` move on checkout/merge/reset/rebase, and
/// `index` changes on commit/checkout. Deliberately excludes `objects/`,
/// `config`, `*.lock`, and `COMMIT_EDITMSG` so plain edits and object writes
/// never trigger a switch.
const RELEVANT_FILES: &[&str] = &["HEAD", "ORIG_HEAD", "MERGE_HEAD", "index"];

/// A native, event-driven [`WorkspaceWatcher`] backed by `notify` (uses
/// `ReadDirectoryChangesW` on Windows). Watches each repository's resolved
/// `.git` directory **non-recursively**, so `.git/objects` churn is never seen
/// and CPU stays flat even across a thousand repositories.
#[derive(Default)]
pub(crate) struct GitDirWatcher {
    debouncer: Mutex<Option<Debouncer<RecommendedWatcher>>>,
    paused: Arc<AtomicBool>,
    count: AtomicUsize,
}

impl GitDirWatcher {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// Resolve the real `.git` directory for a repo root, following the `gitdir:`
/// pointer used by worktrees and submodules. Returns `None` (skip this repo)
/// rather than failing the whole watch if the repo has since disappeared.
fn resolve_git_dir(git_root: &Path) -> Option<PathBuf> {
    let dotgit = git_root.join(".git");
    let meta = std::fs::symlink_metadata(&dotgit).ok()?;
    if meta.is_dir() {
        return Some(dotgit);
    }
    let content = std::fs::read_to_string(&dotgit).ok()?;
    let rest = content.trim().strip_prefix("gitdir:")?.trim();
    let pointer = PathBuf::from(rest);
    Some(if pointer.is_absolute() {
        pointer
    } else {
        git_root.join(pointer)
    })
}

fn is_relevant(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| RELEVANT_FILES.contains(&name))
}

impl WorkspaceWatcher for GitDirWatcher {
    fn start(
        &self,
        git_roots: &[String],
        on_activity: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<(), AppError> {
        // Map each resolvable `.git` dir back to its repo root once, up front.
        let mut targets: Vec<(PathBuf, String)> = Vec::with_capacity(git_roots.len());
        for root in git_roots {
            if let Some(git_dir) = resolve_git_dir(Path::new(root)) {
                targets.push((git_dir, root.clone()));
            }
        }

        let paused = Arc::clone(&self.paused);
        let map = targets.clone();
        let handler = move |result: DebounceEventResult| {
            if paused.load(Ordering::Relaxed) {
                return;
            }
            let Ok(events) = result else {
                return;
            };
            let mut activated: Vec<String> = Vec::new();
            for event in events {
                if !is_relevant(&event.path) {
                    continue;
                }
                let Some(parent) = event.path.parent() else {
                    continue;
                };
                for (git_dir, git_root) in &map {
                    if parent == git_dir.as_path() && !activated.iter().any(|r| r == git_root) {
                        activated.push(git_root.clone());
                    }
                }
            }
            for git_root in activated {
                on_activity(git_root);
            }
        };

        let mut debouncer =
            new_debouncer(DEBOUNCE, handler).map_err(|e| AppError::Io(e.to_string()))?;
        for (git_dir, _) in &targets {
            // A repo that vanished between resolve and watch is skipped, not fatal.
            let _ = debouncer
                .watcher()
                .watch(git_dir, RecursiveMode::NonRecursive);
        }

        self.paused.store(false, Ordering::Relaxed);
        self.count.store(targets.len(), Ordering::Relaxed);
        let mut slot = self
            .debouncer
            .lock()
            .map_err(|_| AppError::Internal("watcher lock poisoned".into()))?;
        // Dropping any previous debouncer here releases its OS watches.
        *slot = Some(debouncer);
        Ok(())
    }

    fn stop(&self) {
        if let Ok(mut slot) = self.debouncer.lock() {
            *slot = None;
        }
        self.count.store(0, Ordering::Relaxed);
        self.paused.store(false, Ordering::Relaxed);
    }

    fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    fn is_watching(&self) -> bool {
        self.debouncer.lock().is_ok_and(|slot| slot.is_some())
    }

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    fn watched_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::Duration;

    fn init_repo(base: &Path) {
        let git = base.join(".git");
        fs::create_dir_all(git.join("logs")).unwrap();
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(git.join("index"), b"x").unwrap();
    }

    #[test]
    fn resolves_plain_and_pointer_git_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let resolved = resolve_git_dir(tmp.path()).unwrap();
        assert_eq!(resolved, tmp.path().join(".git"));

        // worktree-style pointer file
        let real = tmp.path().join("real");
        fs::create_dir_all(&real).unwrap();
        let wt = tmp.path().join("wt");
        fs::create_dir_all(&wt).unwrap();
        fs::write(wt.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();
        assert_eq!(resolve_git_dir(&wt).unwrap(), real);
    }

    #[test]
    fn only_git_state_files_are_relevant() {
        assert!(is_relevant(Path::new("/repo/.git/HEAD")));
        assert!(is_relevant(Path::new("/repo/.git/index")));
        assert!(!is_relevant(Path::new("/repo/.git/config")));
        assert!(!is_relevant(Path::new("/repo/src/main.rs")));
        assert!(!is_relevant(Path::new("/repo/.git/index.lock")));
    }

    #[test]
    fn start_reports_head_change_and_pause_suppresses() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());
        let root = tmp.path().to_string_lossy().into_owned();

        let (tx, rx) = mpsc::channel::<String>();
        let watcher = GitDirWatcher::new();
        let cb: Arc<dyn Fn(String) + Send + Sync> = Arc::new(move |gr| {
            let _ = tx.send(gr);
        });
        watcher.start(std::slice::from_ref(&root), cb).unwrap();
        assert!(watcher.is_watching());
        assert_eq!(watcher.watched_count(), 1);

        // Simulate a checkout: HEAD changes.
        fs::write(
            tmp.path().join(".git").join("HEAD"),
            "ref: refs/heads/dev\n",
        )
        .unwrap();
        let got = rx.recv_timeout(Duration::from_secs(5)).expect("activation");
        assert_eq!(got, root);

        // While paused, further changes must not be reported.
        watcher.pause();
        assert!(watcher.is_paused());
        fs::write(tmp.path().join(".git").join("index"), b"y").unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(900)).is_err());

        watcher.stop();
        assert!(!watcher.is_watching());
        assert_eq!(watcher.watched_count(), 0);
    }
}
