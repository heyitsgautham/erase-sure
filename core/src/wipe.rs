use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}