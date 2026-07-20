use crate::domain::{
    ports::SshKeyReader,
    ssh::{SshAlgorithm, SshKeyMetadata},
};
use crate::error::AppError;
use ssh_key::{Algorithm, HashAlg, PrivateKey};
use std::path::Path;

pub(crate) struct SshKeyFileReader;

/// Map an `ssh-key` algorithm to our supported subset. ED25519 and RSA only.
pub(crate) fn map_algorithm(algorithm: &Algorithm) -> Result<SshAlgorithm, AppError> {
    match algorithm {
        Algorithm::Ed25519 => Ok(SshAlgorithm::Ed25519),
        Algorithm::Rsa { .. } => Ok(SshAlgorithm::Rsa),
        other => Err(AppError::Crypto(format!(
            "unsupported algorithm: {}",
            other.as_str()
        ))),
    }
}

/// Reject private keys that are group/other-accessible on Unix, mirroring
/// OpenSSH's own refusal (`mode & 0o077`). No-op on Windows (ACL-based).
#[cfg(unix)]
fn check_permissions(private_key_path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(private_key_path).map_err(|e| AppError::Io(e.to_string()))?;
    let mode = meta.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(AppError::Validation(format!(
            "private key permissions are too open ({:o}); run `chmod 600` on it first",
            mode & 0o777
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn check_permissions(_private_key_path: &Path) -> Result<(), AppError> {
    Ok(())
}

impl SshKeyReader for SshKeyFileReader {
    fn read(&self, private_key_path: &Path) -> Result<SshKeyMetadata, AppError> {
        check_permissions(private_key_path)?;

        // Parses metadata only — encrypted keys parse fine without decryption,
        // and we never read the private scalar.
        let key = PrivateKey::read_openssh_file(private_key_path)
            .map_err(|e| AppError::Crypto(format!("could not parse key: {e}")))?;

        let algorithm = map_algorithm(&key.algorithm())?;
        let fingerprint = key.public_key().fingerprint(HashAlg::Sha256).to_string();

        let comment = {
            let c = key.comment().trim();
            (!c.is_empty()).then(|| c.to_string())
        };

        let pub_path = {
            let mut p = private_key_path.as_os_str().to_os_string();
            p.push(".pub");
            let candidate = std::path::PathBuf::from(p);
            candidate
                .is_file()
                .then(|| candidate.to_string_lossy().into_owned())
        };

        Ok(SshKeyMetadata {
            algorithm,
            fingerprint,
            comment,
            public_key_path: pub_path,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use ssh_key::rand_core::SeedableRng;
    use ssh_key::LineEnding;

    #[test]
    fn reads_algorithm_and_fingerprint_from_generated_key() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("id_ed25519");
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(42);
        let mut key = PrivateKey::random(&mut rng, Algorithm::Ed25519).unwrap();
        key.set_comment("tester@example.com");
        key.write_openssh_file(&path, LineEnding::LF).unwrap();

        let meta = SshKeyFileReader.read(&path).unwrap();
        assert_eq!(meta.algorithm, SshAlgorithm::Ed25519);
        assert!(meta.fingerprint.starts_with("SHA256:"));
        assert_eq!(meta.comment.as_deref(), Some("tester@example.com"));
    }

    #[test]
    fn rejects_unparseable_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("garbage");
        std::fs::write(&path, b"not a key").unwrap();
        let err = SshKeyFileReader.read(&path).unwrap_err();
        assert!(matches!(err, AppError::Crypto(_)));
    }
}
