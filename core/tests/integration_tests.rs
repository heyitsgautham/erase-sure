#[allow(unused_imports)] // These are needed for comprehensive integration testing
use securewipe::{
    device::{DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel},
    backup::{BackupOperations, EncryptedBackup},
    wipe::{WipeOperations, NistAlignedWipe, WipePolicy},
    cert::{CertificateOperations, Ed25519CertificateManager},
    logging::Logger,
};
use std::collections::HashMap;
use tempfile;

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
        let discovery = LinuxDeviceDiscovery { enable_enrichment: false };
        let devices = discovery.discover_devices();
        // On non-Linux systems, this will fail gracefully
        match devices {
            Ok(_) => {
                // Success case - we're on Linux with lsblk
            }
            Err(e) => {
                // Expected on non-Linux systems - just log and continue with rest of test
                logger.log_info(&format!("Device discovery not available: {}", e));
            }
        }
        
        // Test backup operation
        let temp_source = tempfile::TempDir::new().unwrap();
        let temp_backup = tempfile::TempDir::new().unwrap();
        
        // Create test files
        let test_file = temp_source.path().join("test.txt");
        std::fs::write(&test_file, "integration test content").unwrap();
        
        let backup = EncryptedBackup::new();
        let backup_result = backup.perform_backup(
            "/dev/test",
            &[temp_source.path().to_str().unwrap().to_string()],
            temp_backup.path().to_str().unwrap(),
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
            manifest_sha256: "test_manifest_hash".to_string(),
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
            backup_id: uuid::Uuid::new_v4().to_string(),
            manifest: BackupManifest {
                files: HashMap::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                total_files: 0,
                total_bytes: 0,
                manifest_sha256: "test_manifest_hash".to_string(),
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

    #[test]
    fn test_cert_sign_integration() {
        use securewipe::signer::{load_private_key, sign_certificate};
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        use tempfile::NamedTempFile;
        use std::fs;
        use std::io::Write;
        
        // Generate test signing key
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let private_key_bytes = signing_key.to_bytes();
        
        // Write key to temporary file
        let mut temp_key_file = NamedTempFile::new().unwrap();
        temp_key_file.write_all(&private_key_bytes).unwrap();
        temp_key_file.flush().unwrap();
        
        // Create test certificate JSON
        let test_cert = serde_json::json!({
            "cert_id": "test_cert_001",
            "cert_type": "backup",
            "created_at": "2023-01-01T00:00:00Z",
            "device": {
                "model": "Test Drive",
                "serial": "ABC123"
            },
            "backup_summary": {
                "files": 100,
                "bytes": 1048576
            }
        });
        
        let mut temp_cert_file = NamedTempFile::new().unwrap();
        temp_cert_file.write_all(serde_json::to_string_pretty(&test_cert).unwrap().as_bytes()).unwrap();
        temp_cert_file.flush().unwrap();
        
        // Test loading private key
        let loaded_key = load_private_key(Some(temp_key_file.path().to_path_buf())).unwrap();
        
        // Test signing certificate
        let mut cert_value = test_cert.clone();
        let sign_result = sign_certificate(&mut cert_value, &loaded_key, false);
        assert!(sign_result.is_ok(), "Certificate signing should succeed");
        
        // Verify signature was added
        assert!(cert_value.get("signature").is_some(), "Signature should be present");
        
        let signature_obj = cert_value.get("signature").unwrap();
        assert_eq!(signature_obj.get("alg").unwrap().as_str().unwrap(), "Ed25519");
        assert_eq!(signature_obj.get("pubkey_id").unwrap().as_str().unwrap(), "sih_root_v1");
        assert_eq!(signature_obj.get("canonicalization").unwrap().as_str().unwrap(), "RFC8785_JSON");
        assert!(signature_obj.get("sig").is_some());
        
        // Test that signing already signed cert fails without force
        let sign_result_duplicate = sign_certificate(&mut cert_value, &loaded_key, false);
        assert!(sign_result_duplicate.is_err(), "Should fail when already signed without force");
        
        // Test that force signing works
        let sign_result_force = sign_certificate(&mut cert_value, &loaded_key, true);
        assert!(sign_result_force.is_ok(), "Force signing should succeed");
        
        // Test file-based signing workflow manually
        let mut unsigned_cert_file = NamedTempFile::new().unwrap();
        unsigned_cert_file.write_all(serde_json::to_string_pretty(&test_cert).unwrap().as_bytes()).unwrap();
        unsigned_cert_file.flush().unwrap();
        
        // Read file, sign, and write back (simulating what handle_cert_sign does)
        let cert_content = fs::read_to_string(unsigned_cert_file.path()).unwrap();
        let mut cert_value: serde_json::Value = serde_json::from_str(&cert_content).unwrap();
        
        let sign_result = sign_certificate(&mut cert_value, &loaded_key, false);
        assert!(sign_result.is_ok(), "File-based signing should succeed");
        
        // Write back to file
        let signed_json = serde_json::to_string_pretty(&cert_value).unwrap();
        fs::write(unsigned_cert_file.path(), signed_json).unwrap();
        
        // Verify the file was updated with signature
        let signed_content = fs::read_to_string(unsigned_cert_file.path()).unwrap();
        let signed_cert: serde_json::Value = serde_json::from_str(&signed_content).unwrap();
        assert!(signed_cert.get("signature").is_some(), "Signed certificate file should contain signature");
    }
}