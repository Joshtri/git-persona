use crate::domain::{
    ports::SshKeyGenerator,
    ssh::{GeneratedKey, SshAlgorithm},
};
use crate::error::AppError;
use ssh_key::private::{KeypairData, RsaKeypair};
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};
use std::path::Path;

const RSA_BITS: usize = 4096;

pub(crate) struct SshKeyPairGenerator;

impl SshKeyGenerator for SshKeyPairGenerator {
    fn generate(
        &self,
        algorithm: SshAlgorithm,
        output_dir: &Path,
        file_name: &str,
        comment: Option<&str>,
    ) -> Result<GeneratedKey, AppError> {
        let private_path = output_dir.join(file_name);
        let mut public_os = private_path.clone().into_os_string();
        public_os.push(".pub");
        let public_path = std::path::PathBuf::from(public_os);

        // Never clobber existing key material.
        if private_path.exists() {
            return Err(AppError::Conflict(format!(
                "a file already exists at {}",
                private_path.display()
            )));
        }
        std::fs::create_dir_all(output_dir).map_err(|e| AppError::Io(e.to_string()))?;

        let mut rng = OsRng;
        let mut key = match algorithm {
            SshAlgorithm::Ed25519 => PrivateKey::random(&mut rng, Algorithm::Ed25519)
                .map_err(|e| AppError::Crypto(e.to_string()))?,
            SshAlgorithm::Rsa => {
                let keypair = RsaKeypair::random(&mut rng, RSA_BITS)
                    .map_err(|e| AppError::Crypto(e.to_string()))?;
                PrivateKey::new(KeypairData::from(keypair), "")
                    .map_err(|e| AppError::Crypto(e.to_string()))?
            }
        };
        if let Some(c) = comment {
            key.set_comment(c);
        }

        // Private key: OpenSSH PEM, mode 0600 on Unix (handled by ssh-key).
        key.write_openssh_file(&private_path, LineEnding::LF)
            .map_err(|e| AppError::Io(e.to_string()))?;

        // Public key: standard `authorized_keys` line.
        let public_openssh = key
            .public_key()
            .to_openssh()
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        std::fs::write(&public_path, format!("{public_openssh}\n"))
            .map_err(|e| AppError::Io(e.to_string()))?;

        let fingerprint = key.public_key().fingerprint(HashAlg::Sha256).to_string();

        Ok(GeneratedKey {
            private_key_path: private_path.to_string_lossy().into_owned(),
            public_key_path: public_path.to_string_lossy().into_owned(),
            fingerprint,
            comment: comment.map(str::to_string),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn generates_ed25519_pair_with_fingerprint() {
        let tmp = tempfile::tempdir().unwrap();
        let out = SshKeyPairGenerator
            .generate(
                SshAlgorithm::Ed25519,
                tmp.path(),
                "id_ed25519",
                Some("me@host"),
            )
            .unwrap();
        assert!(std::path::Path::new(&out.private_key_path).is_file());
        assert!(std::path::Path::new(&out.public_key_path).is_file());
        assert!(out.fingerprint.starts_with("SHA256:"));

        // Written private key is a real OpenSSH key that round-trips.
        let reread = PrivateKey::read_openssh_file(std::path::Path::new(&out.private_key_path))
            .unwrap();
        assert_eq!(reread.algorithm(), Algorithm::Ed25519);
    }

    #[test]
    fn generates_rsa_pair_and_rereads() {
        let tmp = tempfile::tempdir().unwrap();
        let out = SshKeyPairGenerator
            .generate(SshAlgorithm::Rsa, tmp.path(), "id_rsa", Some("me@host"))
            .unwrap();
        assert!(std::path::Path::new(&out.private_key_path).is_file());
        assert!(std::path::Path::new(&out.public_key_path).is_file());
        assert!(out.fingerprint.starts_with("SHA256:"));
        let reread =
            PrivateKey::read_openssh_file(std::path::Path::new(&out.private_key_path)).unwrap();
        assert!(matches!(reread.algorithm(), Algorithm::Rsa { .. }));
    }

    #[test]
    fn refuses_to_overwrite_existing_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("id_ed25519"), b"existing").unwrap();
        let err = SshKeyPairGenerator
            .generate(SshAlgorithm::Ed25519, tmp.path(), "id_ed25519", None)
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
    }

    #[cfg(unix)]
    #[test]
    fn private_key_has_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let out = SshKeyPairGenerator
            .generate(SshAlgorithm::Ed25519, tmp.path(), "id_ed25519", None)
            .unwrap();
        let mode = std::fs::metadata(&out.private_key_path)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
