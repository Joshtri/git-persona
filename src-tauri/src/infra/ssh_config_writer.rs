use crate::domain::{ports::SshConfigWriter, ssh::SshConfigEntry};
use crate::error::AppError;
use std::fmt::Write as _;
use std::path::PathBuf;

const START: &str = "# >>> GitPersona Managed >>>";
const END: &str = "# <<< GitPersona Managed <<<";

pub(crate) struct FsSshConfigWriter {
    config_path: PathBuf,
}

impl FsSshConfigWriter {
    pub(crate) fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Render the managed block from `entries`, or `None` when there are no
    /// assigned keys (the block is then removed entirely).
    fn render_block(entries: &[SshConfigEntry]) -> Option<String> {
        if entries.is_empty() {
            return None;
        }
        let mut s = String::new();
        s.push_str(START);
        s.push('\n');
        s.push_str("# Managed by GitPersona — do not edit inside this block.\n");
        for e in entries {
            // `write!` to a String is infallible.
            let _ = write!(
                s,
                "\nHost {}\n    HostName {}\n    User {}\n    IdentityFile {}\n",
                e.host_alias, e.host_name, e.user, e.identity_file
            );
        }
        s.push('\n');
        s.push_str(END);
        Some(s)
    }

    /// Remove an existing managed block (between and including the markers),
    /// returning everything else untouched.
    fn strip_block(content: &str) -> String {
        let Some((before, rest)) = content.split_once(START) else {
            return content.to_string();
        };
        let Some((_managed, after)) = rest.split_once(END) else {
            return content.to_string();
        };
        let before = before.trim_end();
        let after = after.trim_start_matches(['\n', '\r']);
        match (before.is_empty(), after.is_empty()) {
            (true, _) => after.to_string(),
            (false, true) => format!("{before}\n"),
            (false, false) => format!("{before}\n\n{after}"),
        }
    }

    fn merge(existing: &str, entries: &[SshConfigEntry]) -> String {
        let stripped = Self::strip_block(existing);
        let Some(block) = Self::render_block(entries) else {
            let trimmed = stripped.trim_end();
            return if trimmed.is_empty() {
                String::new()
            } else {
                format!("{trimmed}\n")
            };
        };
        let head = stripped.trim_end();
        if head.is_empty() {
            format!("{block}\n")
        } else {
            format!("{head}\n\n{block}\n")
        }
    }

    fn read_existing(&self) -> Result<String, AppError> {
        match std::fs::read_to_string(&self.config_path) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(AppError::Io(e.to_string())),
        }
    }

    /// Write `contents` to the config path via a temp file + rename so a partial
    /// write can never corrupt the user's config.
    fn atomic_write(&self, contents: &str) -> Result<(), AppError> {
        let parent = self
            .config_path
            .parent()
            .ok_or_else(|| AppError::Io("invalid config path".into()))?;
        std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }

        let tmp = self.config_path.with_extension("gitpersona.tmp");
        std::fs::write(&tmp, contents).map_err(|e| AppError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| AppError::Io(e.to_string()))?;
        }
        std::fs::rename(&tmp, &self.config_path).map_err(|e| AppError::Io(e.to_string()))
    }
}

impl SshConfigWriter for FsSshConfigWriter {
    fn write_managed_block(&self, entries: &[SshConfigEntry]) -> Result<(), AppError> {
        let existing = self.read_existing()?;
        let merged = Self::merge(&existing, entries);
        self.atomic_write(&merged)
    }

    fn read_raw(&self) -> Result<String, AppError> {
        self.read_existing()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn entry(alias: &str, file: &str) -> SshConfigEntry {
        SshConfigEntry {
            host_alias: alias.into(),
            host_name: "github.com".into(),
            user: "git".into(),
            identity_file: file.into(),
        }
    }

    #[test]
    fn appends_block_and_preserves_user_config() {
        let user_config = "Host myserver\n    HostName example.com\n    User admin\n";
        let merged = FsSshConfigWriter::merge(user_config, &[entry("github-work", "~/.ssh/work")]);
        assert!(merged.contains("Host myserver"));
        assert!(merged.contains("HostName example.com"));
        assert!(merged.contains(START));
        assert!(merged.contains("Host github-work"));
        assert!(merged.contains("IdentityFile ~/.ssh/work"));
        assert!(merged.contains(END));
        // user content comes before the managed block
        assert!(merged.find("Host myserver").unwrap() < merged.find(START).unwrap());
    }

    #[test]
    fn replaces_only_managed_block() {
        let first = FsSshConfigWriter::merge("Host keep\n    HostName keep.com\n", &[entry("a", "~/.ssh/a")]);
        let second = FsSshConfigWriter::merge(&first, &[entry("b", "~/.ssh/b")]);
        assert!(second.contains("Host keep"));
        assert!(second.contains("Host b"));
        assert!(!second.contains("Host a"));
        // exactly one managed block
        assert_eq!(second.matches(START).count(), 1);
        assert_eq!(second.matches(END).count(), 1);
    }

    #[test]
    fn empty_entries_removes_block_but_keeps_user_config() {
        let with_block = FsSshConfigWriter::merge("Host keep\n    HostName keep.com\n", &[entry("a", "~/.ssh/a")]);
        let cleared = FsSshConfigWriter::merge(&with_block, &[]);
        assert!(cleared.contains("Host keep"));
        assert!(!cleared.contains(START));
        assert!(!cleared.contains("Host a"));
    }

    #[test]
    fn round_trips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let writer = FsSshConfigWriter::new(tmp.path().join("config"));
        writer
            .write_managed_block(&[entry("github-work", "~/.ssh/work")])
            .unwrap();
        let raw = writer.read_raw().unwrap();
        assert!(raw.contains("Host github-work"));
        assert_eq!(raw.matches(START).count(), 1);
    }
}
