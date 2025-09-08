#[allow(unused_imports)] // These are needed for comprehensive integration testing
use securewipe::{
    device::{DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel},
    backup::{BackupOperations, EncryptedBackup},
    wipe::{WipeOperations, NistAlignedWipe, WipePolicy},
    cert::{CertificateOperations, Ed25519CertificateManager},
    logging::Logger,
};
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use securewipe::backup::{BackupOperations, EncryptedBackup, BackupManifest};
    use securewipe::wipe::{WipeOperations, NistAlignedWipe, WipePolicy};
    use securewipe::cert::{CertificateOperations, Ed25519CertificateManager};
    use securewipe::device::{DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel};
    use securewipe::logging::Logger;

    #[test]
    fn test_full_backup_wipe_cert_workflow() {
        let logger = Logger::new();
        
        // Test device discovery
        let discovery = LinuxDeviceDiscovery;
        let devices = discovery.discover_devices();
        assert!(devices.is_ok());
        
        // Test backup operation
        let backup = EncryptedBackup;
        let backup_result = backup.perform_backup(
            &["Documents".to_string()],
            "/mnt/backup",
        );
        assert!(backup_result.is_ok());
        
        let backup_data = backup_result.unwrap();
        assert_eq!(backup_data.encryption_method, "AES-256-CTR");
        
        // Test certificate creation for backup
        let cert_mgr = Ed25519CertificateManager;
        let backup_cert = cert_mgr.create_backup_certificate(&backup_data);
        assert!(backup_cert.is_ok());
        
        let backup_cert_data = backup_cert.unwrap();
        assert_eq!(backup_cert_data.cert_type, "backup");
        
        // Test wipe operation
        let wipe = NistAlignedWipe;
        let wipe_result = wipe.perform_wipe("/dev/sda", WipePolicy::Purge, false);
        assert!(wipe_result.is_ok());
        
        let wipe_data = wipe_result.unwrap();
        assert_eq!(wipe_data.device, "/dev/sda");
        
        // Test certificate creation for wipe with linkage
        let wipe_cert = cert_mgr.create_wipe_certificate(&wipe_data, Some(&backup_cert_data.cert_id));
        assert!(wipe_cert.is_ok());
        
        let wipe_cert_data = wipe_cert.unwrap();
        assert_eq!(wipe_cert_data.cert_type, "wipe");
        assert!(wipe_cert_data.linkage.is_some());
        
        logger.log_info("Integration test completed successfully");
    }
    
    #[test]
    fn test_backup_manifest_integrity() {
        let mut files = HashMap::new();
        files.insert("test/file1.txt".to_string(), "hash1".to_string());
        files.insert("test/file2.txt".to_string(), "hash2".to_string());
        
        let manifest = BackupManifest {
            files: files.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_files: 2,
            total_bytes: 2048,
        };
        
        // Test serialization and deserialization
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: BackupManifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.total_files, 2);
        assert_eq!(deserialized.total_bytes, 2048);
        assert_eq!(deserialized.files.len(), 2);
        assert_eq!(deserialized.files, files);
    }
    
    #[test]
    fn test_wipe_policy_validation() {
        // Test all wipe policies
        let policies = vec![WipePolicy::Clear, WipePolicy::Purge, WipePolicy::Destroy];
        
        for policy in policies {
            let json = serde_json::to_string(&policy).unwrap();
            let deserialized: WipePolicy = serde_json::from_str(&json).unwrap();
            
            // Verify round-trip serialization
            match (&policy, &deserialized) {
                (WipePolicy::Clear, WipePolicy::Clear) => (),
                (WipePolicy::Purge, WipePolicy::Purge) => (),
                (WipePolicy::Destroy, WipePolicy::Destroy) => (),
                _ => panic!("Policy serialization mismatch"),
            }
        }
    }
    
    #[test]
    fn test_risk_level_classification() {
        // Test risk level assignments
        let risk_levels = vec![RiskLevel::Critical, RiskLevel::High, RiskLevel::Safe];
        
        for risk in risk_levels {
            let json = serde_json::to_string(&risk).unwrap();
            let deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
            
            match (&risk, &deserialized) {
                (RiskLevel::Critical, RiskLevel::Critical) => (),
                (RiskLevel::High, RiskLevel::High) => (),
                (RiskLevel::Safe, RiskLevel::Safe) => (),
                _ => panic!("Risk level serialization mismatch"),
            }
        }
    }
    
    #[test]
    fn test_certificate_linkage() {
        let cert_mgr = Ed25519CertificateManager;
        
        // Create a backup certificate
        let backup_result = securewipe::backup::BackupResult {
            manifest: BackupManifest {
                files: HashMap::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                total_files: 0,
                total_bytes: 0,
            },
            destination: "/mnt/backup".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };
        
        let backup_cert = cert_mgr.create_backup_certificate(&backup_result).unwrap();
        
        // Create a wipe certificate with linkage
        let wipe_result = securewipe::wipe::WipeResult {
            device: "/dev/sda".to_string(),
            policy: WipePolicy::Purge,
            method: "controller_sanitize".to_string(),
            commands: vec![],
            verification_samples: 5,
            verification_passed: true,
            fallback_reason: None,
        };
        
        let wipe_cert = cert_mgr.create_wipe_certificate(&wipe_result, Some(&backup_cert.cert_id)).unwrap();
        
        // Verify linkage exists
        assert!(wipe_cert.linkage.is_some());
        
        if let Some(linkage) = &wipe_cert.linkage {
            assert!(linkage.get("backup_cert_id").is_some());
        }
    }
    
    #[test]
    fn test_logging_functionality() {
        let logger = Logger::new();
        
        // Test different log levels
        logger.log_info("Test info message");
        logger.log_error("Test error message");
        
        // Test JSON logging
        let test_data = serde_json::json!({
            "test": "integration",
            "module": "logging"
        });
        logger.log_json(&test_data);
        
        // Should not panic
    }
}