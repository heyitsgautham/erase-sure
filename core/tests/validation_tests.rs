use securewipe::*;

#[cfg(test)]
mod validation_tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile;

    #[test]
    fn test_all_modules_compile() {
        // Test that all modules can be imported and basic types work
        let _logger = Logger::new();
        let _discovery = LinuxDeviceDiscovery { enable_enrichment: false };
        let _backup = EncryptedBackup::new();
        let _wipe = NistAlignedWipe;
        let _cert_mgr = Ed25519CertificateManager;
    }

    #[test]
    fn test_enum_serialization_completeness() {
        // Test all enum variants serialize correctly
        let risk_levels = [RiskLevel::Critical, RiskLevel::High, RiskLevel::Safe];
        for risk in &risk_levels {
            let json = serde_json::to_string(risk).unwrap();
            let _deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
        }

        let wipe_policies = [WipePolicy::Clear, WipePolicy::Purge];
        for policy in &wipe_policies {
            let json = serde_json::to_string(policy).unwrap();
            let _deserialized: WipePolicy = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_struct_creation_completeness() {
        // Test all major structs can be created
        let device = Device {
            name: "/dev/test".to_string(),
            model: Some("Test Device".to_string()),
            serial: Some("TEST123".to_string()),
            capacity_bytes: 1000000000,
            bus: Some("SATA".to_string()),
            mountpoints: vec![],
            risk_level: RiskLevel::Safe,
        };

        let manifest = BackupManifest {
            files: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_files: 0,
            total_bytes: 0,
            manifest_sha256: "test_manifest_hash".to_string(),
        };

        let backup_result = BackupResult {
            backup_id: uuid::Uuid::new_v4().to_string(),
            manifest,
            destination: "/test".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };

        let wipe_result = WipeResult {
            device: "/dev/test".to_string(),
            policy: WipePolicy::Purge,
            method: "test".to_string(),
            commands: vec![],
            verification_samples: 5,
            verification_passed: true,
            fallback_reason: None,
        };

        let signature = CertificateSignature {
            alg: "Ed25519".to_string(),
            pubkey_id: "sih_root_v1".to_string(),
            sig: "test_sig".to_string(),
        };

        // Verify structs are created correctly
        assert_eq!(device.name, "/dev/test");
        assert_eq!(backup_result.encryption_method, "AES-256-CTR");
        assert_eq!(wipe_result.device, "/dev/test");
        assert_eq!(signature.alg, "Ed25519");
    }

    #[test]
    fn test_trait_implementations() {
        // Test that all traits are properly implemented
        let discovery = LinuxDeviceDiscovery { enable_enrichment: false };
        let result = discovery.discover_devices();
        // On non-Linux systems, device discovery will fail gracefully
        match result {
            Ok(_) => {
                // Success case - we're on Linux
            }
            Err(_) => {
                // Expected on non-Linux systems
            }
        }

        let backup = EncryptedBackup::new();
        let temp_source = tempfile::TempDir::new().unwrap();
        let temp_backup = tempfile::TempDir::new().unwrap();
        
        // Create a test file
        let test_file = temp_source.path().join("validation.txt");
        std::fs::write(&test_file, "validation test").unwrap();
        
        let result = backup.perform_backup(
            "/dev/test", 
            &[temp_source.path().to_str().unwrap().to_string()], 
            temp_backup.path().to_str().unwrap()
        );
        assert!(result.is_ok());

        let wipe = NistAlignedWipe;
        let result = wipe.perform_wipe("/dev/test", WipePolicy::Purge, false);
        assert!(result.is_ok());

        let cert_mgr = Ed25519CertificateManager;
        let backup_result = BackupResult {
            backup_id: uuid::Uuid::new_v4().to_string(),
            manifest: BackupManifest {
                files: HashMap::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                total_files: 0,
                total_bytes: 0,
                manifest_sha256: "test_manifest_hash".to_string(),
            },
            destination: "/test".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };
        let result = cert_mgr.create_backup_certificate(&backup_result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cert_verify_command_integration() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("securewipe").unwrap();
        
        // Test with valid signed certificate
        let output = cmd
            .args(&["cert", "verify", 
                   "--file", "test_data/valid_signed_cert.json",
                   "--pubkey", "../keys/dev_public.pem"])
            .output()
            .unwrap();
        
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(result["signature_valid"], true);
        assert_eq!(result["file"], "test_data/valid_signed_cert.json");
    }

    #[test]
    fn test_cert_verify_unsigned_certificate() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("securewipe").unwrap();
        
        // Test with unsigned certificate
        let output = cmd
            .args(&["cert", "verify", 
                   "--file", "test_data/unsigned_cert.json",
                   "--pubkey", "../keys/dev_public.pem"])
            .output()
            .unwrap();
        
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(result["signature_valid"], serde_json::Value::Null);
        assert_eq!(result["file"], "test_data/unsigned_cert.json");
    }

    #[test]
    fn test_cert_verify_invalid_signature() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("securewipe").unwrap();
        
        // Test with invalid signature
        let output = cmd
            .args(&["cert", "verify", 
                   "--file", "test_data/invalid_signed_cert.json",
                   "--pubkey", "../keys/dev_public.pem"])
            .output()
            .unwrap();
        
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(result["signature_valid"], false);
        assert_eq!(result["file"], "test_data/invalid_signed_cert.json");
    }

    #[test]
    fn test_cert_verify_wrong_pubkey_id() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("securewipe").unwrap();
        
        // Test with wrong pubkey_id
        let output = cmd
            .args(&["cert", "verify", 
                   "--file", "test_data/wrong_pubkey_id_cert.json",
                   "--pubkey", "../keys/dev_public.pem"])
            .output()
            .unwrap();
        
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(result["signature_valid"], false);
        assert_eq!(result["file"], "test_data/wrong_pubkey_id_cert.json");
    }

    #[test]
    fn test_cert_verify_missing_file() {
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("securewipe").unwrap();
        
        // Test with non-existent file
        let output = cmd
            .args(&["cert", "verify", 
                   "--file", "test_data/nonexistent.json",
                   "--pubkey", "../keys/dev_public.pem"])
            .output()
            .unwrap();
        
        assert!(!output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(result["signature_valid"], serde_json::Value::Null);
        assert_eq!(result["file"], "test_data/nonexistent.json");
        assert!(result.get("error").is_some());
    }
}