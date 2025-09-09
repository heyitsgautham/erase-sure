use serde::{Deserialize, Serialize};
use std::process::Command;

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
    #[serde(rename = "DESTROY")]
    Destroy,
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
        _force_critical: bool,
    ) -> Result<WipeResult, Box<dyn std::error::Error>> {
        // Stub implementation - will implement actual NIST-aligned wiping
        Ok(WipeResult {
            device: device.to_string(),
            policy,
            method: "stub".to_string(),
            commands: vec![],
            verification_samples: 5,
            verification_passed: true,
            fallback_reason: None,
        })
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
        
        let policy = WipePolicy::Destroy;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, "\"DESTROY\"");
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