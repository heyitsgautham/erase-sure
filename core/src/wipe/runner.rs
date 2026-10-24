//! How the wipe engine runs external tools, with an optional timeout.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Upper bound for read-only probes such as `lsblk`, `umount` and `hdparm -I`.
///
/// Commands that change the device (overwrite passes, controller erases,
/// HPA/DCO resets) run without a limit, because killing one halfway leaves the
/// drive in an unknown state and a full pass takes minutes to hours.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Exit code and captured text of one external command.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandOutput {
    pub exit_code: i32,
    /// stdout, or stderr when stdout is empty (dd reports progress on stderr).
    pub output: String,
}

/// Runs the external tools a wipe needs (`lsblk`, `umount`, `hdparm`, `nvme`, `dd`).
///
/// Production code uses [`SystemCommandRunner`].
/// Tests inject a fake so no real device is ever touched.
pub trait CommandRunner: Send + Sync {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        timeout: Option<Duration>,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>>;
}

/// Spawns real processes.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        timeout: Option<Duration>,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Drain both pipes on their own threads so a chatty child cannot block on a full pipe.
        let mut stdout = child.stdout.take().ok_or("stdout not captured")?;
        let mut stderr = child.stderr.take().ok_or("stderr not captured")?;
        let stdout_reader = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf);
            buf
        });
        let stderr_reader = thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr.read_to_end(&mut buf);
            buf
        });

        let status = match timeout {
            None => child.wait()?,
            Some(limit) => {
                let started = Instant::now();
                loop {
                    if let Some(status) = child.try_wait()? {
                        break status;
                    }
                    if started.elapsed() >= limit {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(format!(
                            "Command timed out after {} seconds",
                            limit.as_secs_f32()
                        )
                        .into());
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
        };

        let stdout = stdout_reader.join().unwrap_or_default();
        let stderr = stderr_reader.join().unwrap_or_default();
        let output = if stdout.is_empty() {
            String::from_utf8_lossy(&stderr).to_string()
        } else {
            String::from_utf8_lossy(&stdout).to_string()
        };

        Ok(CommandOutput {
            exit_code: status.code().unwrap_or(-1),
            output,
        })
    }
}
