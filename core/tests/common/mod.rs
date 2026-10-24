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
