use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

mod plan;
mod runner;

pub use plan::{plan_wipe, VerificationPlan, WipePlan};
pub use runner::{CommandOutput, CommandRunner, SystemCommandRunner, PROBE_TIMEOUT};

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

#[cfg(test)]
mod tests;
