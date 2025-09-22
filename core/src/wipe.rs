use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs::OpenOptions;
use std::time::Instant;
use rand::RngCore;

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

#[allow(dead_code)] // MVP: Implementation pending
pub trait WipeOperations {
    fn perform_wipe(
        &self,
        device: &str,
        policy: WipePolicy,
        force_critical: bool,
    ) -> Result<WipeResult, Box<dyn std::error::Error>>;
}

#[allow(dead_code)] // MVP: Implementation pending
pub struct NistAlignedWipe;

impl WipeOperations for NistAlignedWipe {
    fn perform_wipe(
        &self,
        device: &str,
        policy: WipePolicy,
        _is_critical: bool,
    ) -> Result<WipeResult, Box<dyn std::error::Error>> {
        let mut commands = Vec::new();
        let mut method = String::new();
        let mut fallback_reason = None;

        println!("Starting NIST-aligned wipe on {}", device);

        // Step 0: Unmount all partitions on the device before wiping
        self.unmount_device(device, &mut commands)?;

        // Step 1: Try controller sanitize first, fallback to overwrite methods
        match self.try_controller_sanitize(device, &policy, &mut commands) {
            Ok(true) => {
                method = "controller_sanitize".to_string();
                println!("Controller sanitize successful");
            }
            Ok(false) | Err(_) => {
                // Fallback to overwrite methods
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

        // Step 2: Verification sampling
        let verification_samples = match policy {
            WipePolicy::Clear => 32,
            WipePolicy::Purge => 128,
        };
        
        let verification_passed = self.verify_wipe(device, verification_samples)?;
        
        println!("Wipe verification: {} samples, result: {}", 
                verification_samples, 
                if verification_passed { "PASSED" } else { "FAILED" });

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
        cmd.exit_code == 0 || (cmd.exit_code == 1 && cmd.output.to_lowercase().contains("no space left on device"))
    }

    fn unmount_device(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Unmounting all partitions on {}", device);
        
        // Find all mounted partitions for this device
        let lsblk_result = self.execute_command("lsblk", &["-J", "-o", "NAME,MOUNTPOINT", device], commands)?;
        
        if lsblk_result.exit_code == 0 {
            // Parse lsblk JSON output to find mounted partitions
            if let Ok(lsblk_data) = serde_json::from_str::<serde_json::Value>(&lsblk_result.output) {
                if let Some(blockdevices) = lsblk_data.get("blockdevices").and_then(|v| v.as_array()) {
                    for device_info in blockdevices {
                        // Check main device mountpoint
                        if let Some(mountpoint) = device_info.get("mountpoint").and_then(|v| v.as_str()) {
                            if !mountpoint.is_empty() {
                                println!("Unmounting {}", mountpoint);
                                let _umount_result = self.execute_command("umount", &[mountpoint], commands)?;
                            }
                        }
                        
                        // Check partition mountpoints
                        if let Some(children) = device_info.get("children").and_then(|v| v.as_array()) {
                            for partition in children {
                                if let Some(mountpoint) = partition.get("mountpoint").and_then(|v| v.as_str()) {
                                    if !mountpoint.is_empty() {
                                        println!("Unmounting partition at {}", mountpoint);
                                        let _umount_result = self.execute_command("umount", &[mountpoint], commands)?;
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
                println!("Force unmounting partition {}", partition);
                let _umount_result = self.execute_command("umount", &[&partition], commands)?;
            }
        }
        
        // Wait a moment for unmount to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }

    fn try_controller_sanitize(
        &self,
        device: &str,
        policy: &WipePolicy,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Try NVMe sanitize first
        if device.contains("nvme") {
            let nvme_result = self.execute_command("nvme", &["sanitize", device], commands)?;
            if nvme_result.exit_code == 0 {
                return Ok(true);
            }
        }

        // Try SATA secure erase
        let method = match policy {
            WipePolicy::Clear => "secure-erase",
            WipePolicy::Purge => "secure-erase-enhanced",
        };

        // Check if secure erase is supported
        let identify_result = self.execute_command("hdparm", &["-I", device], commands)?;
        if identify_result.output.contains("Security") && identify_result.output.contains("erase") {
            // Set security password (required for secure erase)
            let _set_pass = self.execute_command("hdparm", &["--user-master", "u", "--security-set-pass", "p", device], commands)?;
            
            // Perform secure erase
            let erase_result = self.execute_command("hdparm", &["--user-master", "u", &format!("--security-{}", method), "p", device], commands)?;
            
            return Ok(erase_result.exit_code == 0);
        }

        Ok(false) // No controller sanitize available
    }

    fn perform_clear_wipe(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Performing CLEAR wipe (single zero pass)");
        
        // Single pass with zeros
        let dd_result = self.execute_command(
            "dd",
            &[
                "if=/dev/zero",
                &format!("of={}", device),
                "bs=1M",
                "conv=fdatasync",
                "status=progress"
            ],
            commands,
        )?;

        if !self.dd_completed_ok(&dd_result) {
            return Err(format!("Zero-fill failed: {} (exit {})", dd_result.output, dd_result.exit_code).into());
        }

        Ok(())
    }

    fn perform_purge_wipe(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Performing PURGE wipe (HPA/DCO clear + random pass + verification)");
        
        // Step 1: Clear HPA/DCO if present
        self.clear_hpa_dco(device, commands)?;
        
        // Step 2: Single pass with random data
        let dd_result = self.execute_command(
            "dd",
            &[
                "if=/dev/urandom",
                &format!("of={}", device),
                "bs=1M",
                "conv=fdatasync",
                "status=progress"
            ],
            commands,
        )?;

        if !self.dd_completed_ok(&dd_result) {
            return Err(format!("Random overwrite failed: {} (exit {})", dd_result.output, dd_result.exit_code).into());
        }

        Ok(())
    }

    fn clear_hpa_dco(
        &self,
        device: &str,
        commands: &mut Vec<WipeCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Clearing HPA/DCO settings");
        
        // Check and disable HPA
        let hpa_check = self.execute_command("hdparm", &["-N", device], commands)?;
        if hpa_check.output.contains("HPA") {
            let _hpa_disable = self.execute_command("hdparm", &["-N", "p", device], commands)?;
        }

        // Check and disable DCO
        let dco_check = self.execute_command("hdparm", &["--dco-identify", device], commands)?;
        if dco_check.exit_code == 0 && dco_check.output.contains("DCO") {
            let _dco_restore = self.execute_command("hdparm", &["--dco-restore", device], commands)?;
        }

        Ok(())
    }

    fn verify_wipe(
        &self,
        device: &str,
        sample_count: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        println!("Verifying wipe with {} random samples", sample_count);
        
        let mut file = OpenOptions::new().read(true).open(device)?;
        
        // Get device size
        file.seek(SeekFrom::End(0))?;
        let device_size = file.stream_position()?;
        
        if device_size == 0 {
            return Err("Cannot determine device size".into());
        }

        let mut rng = rand::thread_rng();
        let mut verified_count = 0;
        
        for _ in 0..sample_count {
            // Random sector to check
            let offset = (rng.next_u64() % (device_size / 512)) * 512;
            
            // Read 512 bytes
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
        
        println!("Verification: {}/{} samples passed ({}%)", 
                verified_count, sample_count, 
                (verified_count * 100) / sample_count);
        
        Ok(passed)
    }

    fn appears_random(&self, data: &[u8]) -> bool {
        // Simple randomness check: count bit transitions
        let mut transitions = 0;
        for i in 1..data.len() {
            if data[i] != data[i-1] {
                transitions += 1;
            }
        }
        
        // Random data should have many transitions
        // More than 30% transitions suggests randomness
        (transitions * 100) / data.len() > 30
    }

    fn execute_command(
        &self,
        command: &str,
        args: &[&str],
        commands: &mut Vec<WipeCommand>,
    ) -> Result<WipeCommand, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let output = Command::new(command)
            .args(args)
            .output()?;
            
        let elapsed = start_time.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);
        let output_str = if output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };

        let cmd_record = WipeCommand {
            command: format!("{} {}", command, args.join(" ")),
            exit_code,
            elapsed_ms: elapsed.as_millis() as u64,
            output: output_str,
        };

        println!("Executed: {} (exit: {}, time: {}ms)", 
                cmd_record.command, cmd_record.exit_code, cmd_record.elapsed_ms);

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
    let risk = if is_critical { "CRITICAL".to_string() } else { "SAFE".to_string() };
    let mut blocked = false;
    let mut reason = None;

    // Guard rails: block CRITICAL unless ISO mode
    if is_critical && !iso_mode {
        blocked = true;
        reason = Some("CRITICAL disk wipe blocked unless running from bootable ISO mode".to_string());
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
    
    #[test]
    fn test_wipe_operations_trait() {
        let wipe = NistAlignedWipe;
        let result = wipe.perform_wipe("/dev/sda", WipePolicy::Purge, false);
        assert!(result.is_ok());
        
        if let Ok(wipe_result) = result {
            assert_eq!(wipe_result.device, "/dev/sda");
            assert!(matches!(wipe_result.policy, WipePolicy::Purge));
            assert_eq!(wipe_result.verification_samples, 5);
            assert!(wipe_result.verification_passed);
        }
    }
    
    #[test]
    fn test_wipe_policy_serialization() {
        let policy = WipePolicy::Purge;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"PURGE\"");
        
        let policy = WipePolicy::Clear;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"CLEAR\"");
        
        let policy = WipePolicy::Purge;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"PURGE\"");
    }
    
    #[test]
    fn test_wipe_policy_deserialization() {
        let json = "\"PURGE\"";
        let policy: WipePolicy = serde_json::from_str(json).unwrap();
        matches!(policy, WipePolicy::Purge);
        
        let json = "\"CLEAR\"";
        let policy: WipePolicy = serde_json::from_str(json).unwrap();
        matches!(policy, WipePolicy::Clear);
    }
    
    #[test]
    fn test_wipe_command_creation() {
        let command = WipeCommand {
            command: "hdparm --secure-erase /dev/sda".to_string(),
            exit_code: 0,
            elapsed_ms: 15000,
            output: "Success".to_string(),
        };
        
        assert_eq!(command.exit_code, 0);
        assert_eq!(command.elapsed_ms, 15000);
        assert!(command.command.contains("hdparm"));
    }
    
    #[test]
    fn test_wipe_result_with_fallback() {
        let wipe = NistAlignedWipe;
        let result = wipe.perform_wipe("/dev/sdb", WipePolicy::Clear, false);
        assert!(result.is_ok());
        
        if let Ok(wipe_result) = result {
            assert_eq!(wipe_result.device, "/dev/sdb");
            assert!(wipe_result.fallback_reason.is_none()); // Stub doesn't set fallback
        }
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
        
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
        
        let deserialized: WipeResult = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.device, "/dev/sda");
        assert!(deserialized.fallback_reason.is_some());
    }
    
    #[test]
    fn test_plan_wipe_safe_disk() {
        let plan = plan_wipe(
            "/dev/sda",
            Some(WipePolicy::Purge),
            false,
            false,
            Some("sanitize Security HPA DCO"),
            None,
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
            None,
            None,
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
            None,
            None,
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
            None,
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
            None,
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
            None,
        );
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: WipePlan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.device, "/dev/sda");
        assert_eq!(deserialized.main_method, "controller_sanitize");
        assert!(deserialized.hpa_dco_clear);
    }
}