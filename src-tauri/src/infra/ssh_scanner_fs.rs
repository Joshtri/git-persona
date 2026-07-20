use crate::domain::ports::SshScanner;
use crate::error::AppError;
use std::io::Read;
use std::path::Path;

/// Filenames that live in `~/.ssh` but are never private keys.
const IGNORED_NAMES: &[&str] = &["known_hosts", "known_hosts.old", "authorized_keys", "config"];

pub(crate) struct FsSshScanner;

impl FsSshScanner {
    /// Cheap heuristic: a private key file begins with a PEM `-----BEGIN ... PRIVATE KEY-----`
    /// header. We read only the first 64 bytes — never the key body.
    fn looks_like_private_key(path: &Path) -> bool {
        let Ok(mut file) = std::fs::File::open(path) else {
            return false;
        };
        let mut head = [0u8; 64];
        let Ok(n) = file.read(&mut head) else {
            return false;
        };
        let prefix = String::from_utf8_lossy(head.get(..n).unwrap_or(&head));
        prefix.contains("PRIVATE KEY")
    }
}

impl SshScanner for FsSshScanner {
    fn scan(&self, ssh_dir: &Path) -> Result<Vec<String>, AppError> {
        if !ssh_dir.is_dir() {
            return Ok(vec![]);
        }
        let entries = std::fs::read_dir(ssh_dir).map_err(|e| AppError::Io(e.to_string()))?;
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let is_pub = path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pub"));
            if IGNORED_NAMES.contains(&name) || is_pub {
                continue;
            }
            if Self::looks_like_private_key(&path) {
                candidates.push(path.to_string_lossy().into_owned());
            }
        }
        candidates.sort();
        Ok(candidates)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::case_sensitive_file_extension_comparisons
)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(dir: &Path, name: &str, contents: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn discovers_private_keys_and_ignores_the_rest() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        write(dir, "id_ed25519", "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n");
        write(dir, "id_rsa", "-----BEGIN RSA PRIVATE KEY-----\ndef\n");
        write(dir, "id_ed25519.pub", "ssh-ed25519 AAAA comment");
        write(dir, "known_hosts", "github.com ssh-ed25519 AAAA");
        write(dir, "authorized_keys", "ssh-ed25519 AAAA");
        write(dir, "config", "Host x\n  HostName y");

        let found = FsSshScanner.scan(dir).unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|p| p.ends_with("id_ed25519")));
        assert!(found.iter().any(|p| p.ends_with("id_rsa")));
        assert!(!found.iter().any(|p| p.ends_with(".pub")));
    }

    #[test]
    fn missing_dir_returns_empty() {
        let found = FsSshScanner.scan(Path::new("/no/such/ssh/dir")).unwrap();
        assert!(found.is_empty());
    }
}
