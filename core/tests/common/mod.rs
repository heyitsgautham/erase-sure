#![allow(dead_code)] // Each test binary uses a different subset of these helpers.

//! Shared helpers for integration tests.
//!
//! Nothing here spawns a process or opens a block device.

use securewipe::wipe::{CommandOutput, CommandRunner, NistAlignedWipe};
use std::time::Duration;

/// Answers every wipe command without running it.
/// `dd` zero-fills its `of=` target, which tests always point at a temp file.
pub struct ZeroFillRunner;

impl CommandRunner for ZeroFillRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        _timeout: Option<Duration>,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let (exit_code, output) = match program {
            "lsblk" => (0, r#"{"blockdevices":[]}"#.to_string()),
            "dd" => {
                let target = args
                    .iter()
                    .find_map(|a| a.strip_prefix("of="))
                    .ok_or("dd without of=")?;
                let len = std::fs::metadata(target)?.len() as usize;
                std::fs::write(target, vec![0u8; len])?;
                (1, "No space left on device".to_string())
            }
            "umount" | "hdparm" => (0, String::new()),
            _ => (1, String::new()),
        };
        Ok(CommandOutput { exit_code, output })
    }
}

/// A wipe engine wired to [`ZeroFillRunner`].
pub fn fake_wipe_engine() -> NistAlignedWipe {
    NistAlignedWipe::with_runner(Box::new(ZeroFillRunner))
}

/// A 1 MiB temp file of non-zero bytes standing in for a disk.
pub fn disk_image(dir: &tempfile::TempDir) -> String {
    let path = dir.path().join("disk.img");
    std::fs::write(&path, vec![0x5Au8; 1024 * 1024]).unwrap();
    path.to_str().unwrap().to_string()
}

/// A throwaway Ed25519 key pair written as PEM files in a temp directory,
/// in the same formats `openssl genpkey -algorithm Ed25519` produces.
pub struct TestKeys {
    _dir: tempfile::TempDir,
    pub private: std::path::PathBuf,
    pub public: std::path::PathBuf,
}

impl TestKeys {
    pub fn generate() -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};
        use ed25519_dalek::SigningKey;

        // DER prefixes for an Ed25519 PKCS#8 private key and SPKI public key (RFC 8410).
        const PKCS8_PREFIX: [u8; 16] = [
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20,
        ];
        const SPKI_PREFIX: [u8; 12] = [
            0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
        ];

        let key = SigningKey::generate(&mut rand::rngs::OsRng);
        let pem = |label: &str, prefix: &[u8], body: &[u8]| {
            let der = [prefix, body].concat();
            format!(
                "-----BEGIN {label}-----\n{}\n-----END {label}-----\n",
                STANDARD.encode(der)
            )
        };

        let dir = tempfile::TempDir::new().unwrap();
        let private = dir.path().join("test_private.pem");
        let public = dir.path().join("test_public.pem");
        std::fs::write(&private, pem("PRIVATE KEY", &PKCS8_PREFIX, &key.to_bytes())).unwrap();
        std::fs::write(
            &public,
            pem("PUBLIC KEY", &SPKI_PREFIX, key.verifying_key().as_bytes()),
        )
        .unwrap();

        Self {
            _dir: dir,
            private,
            public,
        }
    }

    pub fn public_str(&self) -> &str {
        self.public.to_str().unwrap()
    }

    pub fn private_str(&self) -> &str {
        self.private.to_str().unwrap()
    }
}
