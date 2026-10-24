use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Upper bound for read-only probes such as `lsblk`, `umount` and `hdparm -I`.
///
/// Commands that change the device (overwrite passes, controller erases,
/// HPA/DCO resets) run without a limit, because killing one halfway leaves the
/// drive in an unknown state and a full pass takes minutes to hours.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipePlan {
    pub device: String,
    pub risk: String,
    pub policy: WipePolicy,
    pub hpa_dco_clear: bool,
    pub main_method: String,
    pub verification: VerificationPlan,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub strategy: String,
    pub samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WipePolicy {
    #[serde(rename = "CLEAR")]
    Clear,
    #[serde(rename = "PURGE")]
    Purge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeCommand {
    pub command: String,
    pub exit_code: i32,
    pub elapsed_ms: u64,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeResult {
    pub device: String,
    pub policy: WipePolicy,
    pub method: String, // "controller_sanitize", "overwrite", etc.
    pub commands: Vec<WipeCommand>,
    pub verification_samples: usize,
    pub verification_passed: bool,
    pub fallback_reason: Option<String>,
}

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

pub trait WipeOperations {
    fn perform_wipe(
        &self,
        device: &str,
        policy: WipePolicy,
        force_critical: bool,
    ) -> Result<WipeResult, Box<dyn std::error::Error>>;
}

/// NIST SP 800-88 aligned wipe engine.
///
/// It tries a controller sanitize first and falls back to an overwrite pass,
/// then samples random sectors to verify the result.
pub struct NistAlignedWipe {
    runner: Box<dyn CommandRunner>,
}

impl Default for NistAlignedWipe {
    fn default() -> Self {
        Self::new()
    }
}

impl NistAlignedWipe {
    /// Engine that runs the real system tools.
    pub fn new() -> Self {
        Self::with_runner(Box::new(SystemCommandRunner))
    }

    /// Engine that sends every external command through `runner`.
    pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl WipeOperations for NistAlignedWipe {
    fn perform_wipe(
        &self,
        device: &str,
        policy: WipePolicy,
        _is_critical: bool,
    ) -> Result<WipeResult, Box<dyn std::error::Error>> {
        let mut commands = Vec::new();
        let method;
        let mut fallback_reason = None;

        info!(device, ?policy, "Starting NIST-aligned wipe");

        // Check if we have permission to write to the device
        self.check_device_permissions(device)?;

        // Step 1: Unmount all partitions on the device before wiping
        self.unmount_device(device, &mut commands)?;

        // Step 2: Try controller sanitize first, fallback to overwrite methods
        match self.try_controller_sanitize(device, &policy, &mut commands) {
            Ok(true) => {
                method = "controller_sanitize".to_string();
                info!(device, "Controller sanitize successful");
            }
            Ok(false) | Err(_) => {
                fallback_reason = Some("Controller sanitize not available or failed".to_string());
                method = "overwrite".to_string();

                match policy {
                    WipePolicy::Clear => {
                        self.perform_clear_wipe(device, &mut commands)?;
                    }
                    WipePolicy::Purge => {
                        self.perform_purge_wipe(device, &mut commands)?;
                    }
                }
            }
        }

        // Step 3: Verification sampling
        let verification_samples = match policy {
            WipePolicy::Clear => 32,
            WipePolicy::Purge => 128,
        };

        let verification_passed = self.verify_wipe(device, verification_samples)?;

        info!(
            device,
            verification_samples,
            verification_passed,
            method = %method,
            "Wipe finished"
        );

        Ok(WipeResult {
            device: device.to_string(),
            policy,
            method,
            commands,
            verification_samples,
            verification_passed,
            fallback_reason,
        })
    }
}

impl NistAlignedWipe {
    fn dd_completed_ok(&self, cmd: &WipeCommand) -> bool {
        // dd returns exit code 1 when it hits end of device with no count specified
        // and reports "No space left on device". Treat this as a successful full write.
        cmd.exit_code == 0
            || (cmd.exit_code == 1
                && cmd
                    .output
                    .to_lowercase()
                    .contains("no space left on device"))
    }

    fn unmount_device(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(device, "Unmounting all partitions");

        // Find all mounted partitions for this device
        let lsblk_result =
            self.probe("lsblk", &["-J", "-o", "NAME,MOUNTPOINT", device], commands)?;

        if lsblk_result.exit_code == 0 {
            if let Ok(lsblk_data) = serde_json::from_str::<serde_json::Value>(&lsblk_result.output)
            {
                if let Some(blockdevices) =
                    lsblk_data.get("blockdevices").and_then(|v| v.as_array())
                {
                    for device_info in blockdevices {
                        if let Some(mountpoint) =
                            device_info.get("mountpoint").and_then(|v| v.as_str())
                        {
                            if !mountpoint.is_empty() {
                                info!(device, mountpoint, "Unmounting");
                                self.probe("umount", &[mountpoint], commands)?;
                            }
                        }

                        if let Some(children) =
                            device_info.get("children").and_then(|v| v.as_array())
                        {
                            for partition in children {
                                if let Some(mountpoint) =
                                    partition.get("mountpoint").and_then(|v| v.as_str())
                                {
                                    if !mountpoint.is_empty() {
                                        info!(device, mountpoint, "Unmounting partition");
                                        self.probe("umount", &[mountpoint], commands)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also try to force unmount by device name patterns
        // This handles cases where the JSON parsing might fail
        for i in 1..=16 {
            let partition = format!("{}{}", device, i);
            if std::path::Path::new(&partition).exists() {
                info!(device, partition = %partition, "Force unmounting partition");
                self.probe("umount", &[&partition], commands)?;
            }
        }

        // Wait a moment for unmount to complete
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    fn check_device_permissions(&self, device: &str) -> Result<(), Box<dyn std::error::Error>> {
        match OpenOptions::new().write(true).open(device) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(format!(
                        "Permission denied: Cannot write to device {}. \
                         This application needs elevated privileges to perform disk wiping. \
                         Please run with sudo: 'sudo securewipe wipe --device {} --danger-allow-wipe'",
                        device, device
                    )
                    .into())
                } else {
                    Err(format!("Cannot access device {}: {}", device, e).into())
                }
            }
        }
    }

    fn try_controller_sanitize(
        &self,
        device: &str,
        policy: &WipePolicy,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Skip controller sanitize for USB devices as they often don't support it
        if device.contains("sd") && !device.contains("nvme") {
            info!(device, "Skipping controller sanitize for USB device");
            return Ok(false);
        }

        // Try NVMe sanitize first
        if device.contains("nvme") {
            if let Ok(nvme_result) = self.long_running("nvme", &["sanitize", device], commands) {
                if nvme_result.exit_code == 0 {
                    return Ok(true);
                }
            }
        }

        // Try SATA secure erase
        let method = match policy {
            WipePolicy::Clear => "secure-erase",
            WipePolicy::Purge => "secure-erase-enhanced",
        };

        match self.probe("hdparm", &["-I", device], commands) {
            Ok(identify_result) => {
                if identify_result.output.contains("Security")
                    && identify_result.output.contains("erase")
                {
                    // Set security password (required for secure erase)
                    if let Err(e) = self.long_running(
                        "hdparm",
                        &["--user-master", "u", "--security-set-pass", "p", device],
                        commands,
                    ) {
                        warn!(device, error = %e, "Failed to set ATA security password");
                        return Ok(false);
                    }

                    // Perform secure erase. This can run for hours, so it gets no timeout.
                    let erase_flag = format!("--security-{}", method);
                    if let Ok(erase_result) = self.long_running(
                        "hdparm",
                        &["--user-master", "u", &erase_flag, "p", device],
                        commands,
                    ) {
                        if erase_result.exit_code == 0 {
                            return Ok(true);
                        }
                    }
                }
            }
            Err(e) => {
                warn!(device, error = %e, "hdparm identify failed, likely unsupported on this device");
            }
        }

        Ok(false) // No controller sanitize available
    }

    fn perform_clear_wipe(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(device, "Performing CLEAR wipe (single zero pass)");

        let of = format!("of={}", device);
        let dd_result = self.long_running(
            "dd",
            &[
                "if=/dev/zero",
                &of,
                "bs=1M",
                "conv=fdatasync",
                "status=progress",
            ],
            commands,
        )?;

        if !self.dd_completed_ok(&dd_result) {
            return Err(format!(
                "Zero-fill failed: {} (exit {})",
                dd_result.output, dd_result.exit_code
            )
            .into());
        }

        Ok(())
    }

    fn perform_purge_wipe(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            device,
            "Performing PURGE wipe (HPA/DCO clear, then one random pass)"
        );

        // Step 1: Clear HPA/DCO if present
        self.clear_hpa_dco(device, commands)?;

        // Step 2: Single pass with random data
        let of = format!("of={}", device);
        let dd_result = self.long_running(
            "dd",
            &[
                "if=/dev/urandom",
                &of,
                "bs=1M",
                "conv=fdatasync",
                "status=progress",
            ],
            commands,
        )?;

        if !self.dd_completed_ok(&dd_result) {
            return Err(format!(
                "Random overwrite failed: {} (exit {})",
                dd_result.output, dd_result.exit_code
            )
            .into());
        }

        Ok(())
    }

    fn clear_hpa_dco(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(device, "Clearing HPA/DCO settings");

        let hpa_check = self.probe("hdparm", &["-N", device], commands)?;
        if hpa_check.output.contains("HPA") {
            self.long_running("hdparm", &["-N", "p", device], commands)?;
        }

        let dco_check = self.probe("hdparm", &["--dco-identify", device], commands)?;
        if dco_check.exit_code == 0 && dco_check.output.contains("DCO") {
            self.long_running("hdparm", &["--dco-restore", device], commands)?;
        }

        Ok(())
    }

    fn verify_wipe(
        &self,
        device: &str,
        sample_count: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!(
            device,
            sample_count, "Verifying wipe with random sector samples"
        );

        let mut file = OpenOptions::new().read(true).open(device)?;

        file.seek(SeekFrom::End(0))?;
        let device_size = file.stream_position()?;

        let sector_count = device_size / 512;
        if sector_count == 0 || sample_count == 0 {
            return Err(format!(
                "Cannot verify {}: size {} bytes holds no full 512-byte sector",
                device, device_size
            )
            .into());
        }

        let mut rng = rand::thread_rng();
        let mut verified_count = 0;

        for _ in 0..sample_count {
            let offset = (rng.next_u64() % sector_count) * 512;

            file.seek(SeekFrom::Start(offset))?;
            let mut buffer = [0u8; 512];
            file.read_exact(&mut buffer)?;

            // Check if sector appears to be wiped (mostly zeros or random-looking)
            let zero_count = buffer.iter().filter(|&&b| b == 0).count();
            let is_likely_wiped = zero_count > 400 || self.appears_random(&buffer);

            if is_likely_wiped {
                verified_count += 1;
            }
        }

        // Consider verification passed if >95% of samples look wiped
        let success_threshold = (sample_count * 95) / 100;
        let passed = verified_count >= success_threshold;

        info!(
            device,
            verified_count, sample_count, passed, "Verification sampling complete"
        );

        Ok(passed)
    }

    fn appears_random(&self, data: &[u8]) -> bool {
        // Simple randomness check: count byte-to-byte transitions.
        // More than 30% transitions suggests random data.
        let transitions = data.windows(2).filter(|w| w[0] != w[1]).count();
        (transitions * 100) / data.len() > 30
    }

    /// Run a short read-only probe with [`PROBE_TIMEOUT`].
    fn probe(
        &self,
        command: &str,
        args: &[&str],
        commands: &mut Vec<WipeCommand>,
    ) -> Result<WipeCommand, Box<dyn std::error::Error>> {
        self.execute_command(command, args, Some(PROBE_TIMEOUT), commands)
    }

    /// Run a command that changes the device, with no timeout.
    fn long_running(
        &self,
        command: &str,
        args: &[&str],
        commands: &mut Vec<WipeCommand>,
    ) -> Result<WipeCommand, Box<dyn std::error::Error>> {
        self.execute_command(command, args, None, commands)
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[&str],
        timeout: Option<Duration>,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<WipeCommand, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let result = self.runner.run(command, args, timeout)?;

        let cmd_record = WipeCommand {
            command: format!("{} {}", command, args.join(" ")),
            exit_code: result.exit_code,
            elapsed_ms: start_time.elapsed().as_millis() as u64,
            output: result.output,
        };

        info!(
            command = %cmd_record.command,
            exit_code = cmd_record.exit_code,
            elapsed_ms = cmd_record.elapsed_ms,
            "Executed command"
        );

        commands.push(cmd_record.clone());
        Ok(cmd_record)
    }
}

/// Plan a wipe operation without performing destructive actions
pub fn plan_wipe(
    device: &str,
    policy: Option<WipePolicy>,
    is_critical: bool,
    iso_mode: bool,
    mock_hdparm: Option<&str>,
    mock_nvme: Option<&str>,
) -> WipePlan {
    let policy = policy.unwrap_or(WipePolicy::Purge);
    let mut hpa_dco_clear = false;
    let mut main_method = "overwrite".to_string();
    let risk = if is_critical {
        "CRITICAL".to_string()
    } else {
        "SAFE".to_string()
    };
    let mut blocked = false;
    let mut reason = None;

    // Guard rails: block CRITICAL unless ISO mode
    if is_critical && !iso_mode {
        blocked = true;
        reason =
            Some("CRITICAL disk wipe blocked unless running from bootable ISO mode".to_string());
    }

    // Probe controller capabilities (non-fatal)
    let hdparm_output = if let Some(mock) = mock_hdparm {
        Some(mock.to_string())
    } else {
        Command::new("hdparm")
            .arg("-I")
            .arg(device)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
    };

    let nvme_output = if let Some(mock) = mock_nvme {
        Some(mock.to_string())
    } else {
        Command::new("nvme")
            .arg("id-ctrl")
            .arg(device)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
    };

    // Determine method based on controller capabilities
    if let Some(ref hdparm) = hdparm_output {
        if hdparm.contains("sanitize") || hdparm.contains("Security") {
            main_method = "controller_sanitize".to_string();
        }
        if hdparm.contains("HPA") || hdparm.contains("DCO") {
            hpa_dco_clear = true;
        }
    }

    if let Some(ref nvme) = nvme_output {
        if nvme.contains("sanitize") {
            main_method = "controller_sanitize".to_string();
        }
    }

    let verification = VerificationPlan {
        strategy: "random_sectors".to_string(),
        samples: 128,
    };

    WipePlan {
        device: device.to_string(),
        risk,
        policy,
        hpa_dco_clear,
        main_method,
        verification,
        blocked,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    /// Stands in for the system tools. It never spawns a process.
    ///
    /// `dd` is simulated against the target path, which in these tests is always
    /// a temp file, so the verification pass reads what a real overwrite would leave.
    #[derive(Default)]
    struct FakeRunner {
        calls: Arc<Mutex<Vec<String>>>,
        lsblk_json: String,
        hdparm_identify: String,
        nvme_sanitize_exit: i32,
        dd_exit: i32,
        dd_output: String,
        dd_writes: bool,
    }

    impl FakeRunner {
        fn overwriting() -> Self {
            Self {
                lsblk_json: r#"{"blockdevices":[{"name":"loop0","mountpoint":null}]}"#.into(),
                nvme_sanitize_exit: 1,
                dd_exit: 1,
                dd_output: "dd: error writing: No space left on device".into(),
                dd_writes: true,
                ..Default::default()
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &self,
            program: &str,
            args: &[&str],
            _timeout: Option<Duration>,
        ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{} {}", program, args.join(" ")));
            let reply = |exit_code: i32, output: &str| {
                Ok(CommandOutput {
                    exit_code,
                    output: output.to_string(),
                })
            };
            match program {
                "lsblk" => reply(0, &self.lsblk_json),
                "umount" => reply(0, ""),
                "nvme" => reply(self.nvme_sanitize_exit, ""),
                "hdparm" if args.first() == Some(&"-I") => reply(0, &self.hdparm_identify),
                "hdparm" => reply(0, ""),
                "dd" => {
                    if self.dd_writes {
                        let source = args.iter().find_map(|a| a.strip_prefix("if=")).unwrap();
                        let target = args.iter().find_map(|a| a.strip_prefix("of=")).unwrap();
                        let len = std::fs::metadata(target)?.len() as usize;
                        let mut data = vec![0u8; len];
                        if source == "/dev/urandom" {
                            rand::thread_rng().fill_bytes(&mut data);
                        }
                        std::fs::write(target, data)?;
                    }
                    reply(self.dd_exit, &self.dd_output)
                }
                other => Err(format!("unexpected command in test: {}", other).into()),
            }
        }
    }

    /// A 64 KiB temp file of long byte runs, which looks neither zeroed nor
    /// random to the verifier.
    fn image_with_data(name: &str) -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        for _ in 0..512 {
            file.write_all(&[b'A'; 64]).unwrap();
            file.write_all(&[b'B'; 64]).unwrap();
        }
        (dir, path.to_str().unwrap().to_string())
    }

    fn run_wipe(
        runner: FakeRunner,
        device: &str,
        policy: WipePolicy,
    ) -> (Result<WipeResult, Box<dyn std::error::Error>>, Vec<String>) {
        let calls = runner.calls.clone();
        let engine = NistAlignedWipe::with_runner(Box::new(runner));
        let result = engine.perform_wipe(device, policy, false);
        let calls = calls.lock().unwrap().clone();
        (result, calls)
    }

    #[test]
    fn clear_overwrites_with_zeros_and_verifies() {
        let (_dir, image) = image_with_data("disk.img");
        let (result, calls) = run_wipe(FakeRunner::overwriting(), &image, WipePolicy::Clear);
        let result = result.unwrap();

        assert_eq!(result.method, "overwrite");
        assert_eq!(result.verification_samples, 32);
        assert!(result.verification_passed);
        assert!(result.fallback_reason.is_some());
        assert!(std::fs::read(&image).unwrap().iter().all(|&b| b == 0));
        let dd = format!(
            "dd if=/dev/zero of={} bs=1M conv=fdatasync status=progress",
            image
        );
        assert!(calls.contains(&dd), "calls were {:?}", calls);
        assert!(!calls.iter().any(|c| c.starts_with("hdparm -N")));
    }

    #[test]
    fn purge_clears_hpa_dco_then_writes_random_data() {
        let (_dir, image) = image_with_data("disk.img");
        let (result, calls) = run_wipe(FakeRunner::overwriting(), &image, WipePolicy::Purge);
        let result = result.unwrap();

        assert_eq!(result.method, "overwrite");
        assert_eq!(result.verification_samples, 128);
        assert!(result.verification_passed);
        let hpa = calls
            .iter()
            .position(|c| c.starts_with("hdparm -N"))
            .unwrap();
        let dd = calls
            .iter()
            .position(|c| c.starts_with("dd if=/dev/urandom"))
            .unwrap();
        assert!(hpa < dd, "HPA/DCO must be cleared before the overwrite");
        assert!(!std::fs::read(&image).unwrap().iter().all(|&b| b == 0));
    }

    #[test]
    fn verification_fails_when_overwrite_left_data_behind() {
        let (_dir, image) = image_with_data("disk.img");
        let runner = FakeRunner {
            dd_writes: false,
            dd_exit: 0,
            ..FakeRunner::overwriting()
        };
        let (result, _) = run_wipe(runner, &image, WipePolicy::Clear);

        assert!(!result.unwrap().verification_passed);
    }

    #[test]
    fn dd_failure_aborts_the_wipe() {
        let (_dir, image) = image_with_data("disk.img");
        let runner = FakeRunner {
            dd_writes: false,
            dd_exit: 1,
            dd_output: "dd: error writing: Input/output error".into(),
            ..FakeRunner::overwriting()
        };
        let (result, _) = run_wipe(runner, &image, WipePolicy::Clear);

        let err = result.unwrap_err().to_string();
        assert!(err.contains("Zero-fill failed"), "{}", err);
        assert!(err.contains("Input/output error"), "{}", err);
    }

    #[test]
    fn mounted_partitions_are_unmounted_before_writing() {
        let (_dir, image) = image_with_data("disk.img");
        let runner = FakeRunner {
            lsblk_json: r#"{"blockdevices":[{"name":"loop0","mountpoint":"/mnt/a",
                "children":[{"name":"loop0p1","mountpoint":"/mnt/b"},
                            {"name":"loop0p2","mountpoint":null}]}]}"#
                .into(),
            ..FakeRunner::overwriting()
        };
        let (result, calls) = run_wipe(runner, &image, WipePolicy::Clear);
        result.unwrap();

        let first_dd = calls.iter().position(|c| c.starts_with("dd ")).unwrap();
        let umount_a = calls.iter().position(|c| c == "umount /mnt/a").unwrap();
        let umount_b = calls.iter().position(|c| c == "umount /mnt/b").unwrap();
        assert!(umount_a < first_dd && umount_b < first_dd);
        assert_eq!(calls.iter().filter(|c| c.starts_with("umount")).count(), 2);
    }

    #[test]
    fn nvme_sanitize_success_skips_overwrite() {
        let (_dir, image) = image_with_data("nvme0n1.img");
        let runner = FakeRunner {
            nvme_sanitize_exit: 0,
            dd_writes: false,
            ..FakeRunner::overwriting()
        };
        let (result, calls) = run_wipe(runner, &image, WipePolicy::Purge);
        let result = result.unwrap();

        assert_eq!(result.method, "controller_sanitize");
        assert!(result.fallback_reason.is_none());
        assert!(calls.contains(&format!("nvme sanitize {}", image)));
        assert!(!calls.iter().any(|c| c.starts_with("dd ")));
    }

    #[test]
    fn missing_device_is_rejected_before_any_command_runs() {
        let dir = tempfile::TempDir::new().unwrap();
        let missing = dir.path().join("absent.img");
        let (result, calls) = run_wipe(
            FakeRunner::overwriting(),
            missing.to_str().unwrap(),
            WipePolicy::Clear,
        );

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot access device"));
        assert!(calls.is_empty());
    }

    #[test]
    fn verification_rejects_a_device_smaller_than_one_sector() {
        let dir = tempfile::TempDir::new().unwrap();
        let tiny = dir.path().join("tiny.img");
        std::fs::write(&tiny, [0u8; 100]).unwrap();
        let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::overwriting()));

        let err = engine
            .verify_wipe(tiny.to_str().unwrap(), 8)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no full 512-byte sector"), "{}", err);
    }

    #[test]
    fn dd_completed_ok_accepts_end_of_device() {
        let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::default()));
        let cmd = |exit_code: i32, output: &str| WipeCommand {
            command: "dd".into(),
            exit_code,
            elapsed_ms: 0,
            output: output.into(),
        };

        assert!(engine.dd_completed_ok(&cmd(0, "")));
        assert!(engine.dd_completed_ok(&cmd(
            1,
            "dd: error writing '/dev/sdz': No space left on device"
        )));
        assert!(!engine.dd_completed_ok(&cmd(1, "dd: Permission denied")));
        assert!(!engine.dd_completed_ok(&cmd(2, "No space left on device")));
    }

    #[test]
    fn appears_random_separates_noise_from_patterns() {
        let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::default()));
        let mut noise = [0u8; 512];
        rand::thread_rng().fill_bytes(&mut noise);

        assert!(engine.appears_random(&noise));
        assert!(!engine.appears_random(&[0xAA; 512]));
        assert!(!engine.appears_random(&[0u8; 512]));
    }

    #[cfg(unix)]
    #[test]
    fn system_runner_captures_exit_code_and_output() {
        let out = SystemCommandRunner
            .run("sh", &["-c", "echo hello; exit 3"], Some(PROBE_TIMEOUT))
            .unwrap();
        assert_eq!(out.exit_code, 3);
        assert_eq!(out.output.trim(), "hello");

        let err_only = SystemCommandRunner
            .run("sh", &["-c", "echo oops >&2"], None)
            .unwrap();
        assert_eq!(err_only.output.trim(), "oops");
    }

    #[cfg(unix)]
    #[test]
    fn system_runner_kills_a_probe_that_overruns_its_timeout() {
        let started = Instant::now();
        let err = SystemCommandRunner
            .run("sleep", &["5"], Some(Duration::from_millis(200)))
            .unwrap_err();

        assert!(err.to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[cfg(unix)]
    #[test]
    fn system_runner_waits_for_long_commands_without_a_timeout() {
        let out = SystemCommandRunner
            .run("sh", &["-c", "sleep 0.3; echo done"], None)
            .unwrap();
        assert_eq!(out.exit_code, 0);
        assert_eq!(out.output.trim(), "done");
    }

    #[test]
    fn test_wipe_policy_serialization() {
        assert_eq!(
            serde_json::to_string(&WipePolicy::Purge).unwrap(),
            "\"PURGE\""
        );
        assert_eq!(
            serde_json::to_string(&WipePolicy::Clear).unwrap(),
            "\"CLEAR\""
        );
    }

    #[test]
    fn test_wipe_policy_deserialization() {
        let purge: WipePolicy = serde_json::from_str("\"PURGE\"").unwrap();
        let clear: WipePolicy = serde_json::from_str("\"CLEAR\"").unwrap();
        assert_eq!(purge, WipePolicy::Purge);
        assert_eq!(clear, WipePolicy::Clear);
        assert!(serde_json::from_str::<WipePolicy>("\"DESTROY\"").is_err());
    }

    #[test]
    fn test_wipe_result_serialization() {
        let result = WipeResult {
            device: "/dev/sda".to_string(),
            policy: WipePolicy::Purge,
            method: "controller_sanitize".to_string(),
            commands: vec![],
            verification_samples: 5,
            verification_passed: true,
            fallback_reason: Some("Controller sanitize not supported".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: WipeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.device, "/dev/sda");
        assert!(deserialized.fallback_reason.is_some());
    }

    // plan_wipe tests always pass mock probe output so they never run hdparm or nvme.

    #[test]
    fn test_plan_wipe_safe_disk() {
        let plan = plan_wipe(
            "/dev/sda",
            Some(WipePolicy::Purge),
            false,
            false,
            Some("sanitize Security HPA DCO"),
            Some(""),
        );
        assert_eq!(plan.device, "/dev/sda");
        assert_eq!(plan.risk, "SAFE");
        assert_eq!(plan.policy, WipePolicy::Purge);
        assert_eq!(plan.main_method, "controller_sanitize");
        assert!(plan.hpa_dco_clear);
        assert!(!plan.blocked);
        assert_eq!(plan.verification.strategy, "random_sectors");
        assert_eq!(plan.verification.samples, 128);
    }

    #[test]
    fn test_plan_wipe_critical_blocked() {
        let plan = plan_wipe(
            "/dev/sda",
            Some(WipePolicy::Clear),
            true,
            false,
            Some(""),
            Some(""),
        );
        assert!(plan.blocked);
        assert_eq!(plan.risk, "CRITICAL");
        assert!(plan.reason.unwrap().contains("CRITICAL disk wipe blocked"));
    }

    #[test]
    fn test_plan_wipe_critical_iso_mode_allowed() {
        let plan = plan_wipe(
            "/dev/sda",
            Some(WipePolicy::Clear),
            true,
            true,
            Some(""),
            Some(""),
        );
        assert!(!plan.blocked);
        assert_eq!(plan.risk, "CRITICAL");
        assert!(plan.reason.is_none());
    }

    #[test]
    fn test_plan_wipe_nvme_sanitize() {
        let plan = plan_wipe(
            "/dev/nvme0n1",
            None, // default to PURGE
            false,
            false,
            Some(""),
            Some("sanitize capabilities"),
        );
        assert_eq!(plan.main_method, "controller_sanitize");
        assert_eq!(plan.policy, WipePolicy::Purge);
    }

    #[test]
    fn test_plan_wipe_fallback_overwrite() {
        let plan = plan_wipe(
            "/dev/sdb",
            Some(WipePolicy::Clear),
            false,
            false,
            Some("basic drive info"),
            Some(""),
        );
        assert_eq!(plan.main_method, "overwrite");
        assert!(!plan.hpa_dco_clear);
    }

    #[test]
    fn test_plan_serialization() {
        let plan = plan_wipe(
            "/dev/sda",
            Some(WipePolicy::Purge),
            false,
            false,
            Some("Security sanitize HPA DCO"),
            Some(""),
        );
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: WipePlan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.device, "/dev/sda");
        assert_eq!(deserialized.main_method, "controller_sanitize");
        assert!(deserialized.hpa_dco_clear);
    }
}
