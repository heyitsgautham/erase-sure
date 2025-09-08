use securewipe::*;

#[cfg(test)]
mod validation_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_all_modules_compile() {
        // Test that all modules can be imported and basic types work
        let _logger = Logger::new();
        let _discovery = LinuxDeviceDiscovery { enable_enrichment: false };
        let _backup = EncryptedBackup;
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

        let wipe_policies = [WipePolicy::Clear, WipePolicy::Purge, WipePolicy::Destroy];
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
        };

        let backup_result = BackupResult {
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

        let backup = EncryptedBackup;
        let result = backup.perform_backup(&[], "/backup");
        assert!(result.is_ok());

        let wipe = NistAlignedWipe;
        let result = wipe.perform_wipe("/dev/test", WipePolicy::Purge, false);
        assert!(result.is_ok());

        let cert_mgr = Ed25519CertificateManager;
        let backup_result = BackupResult {
            manifest: BackupManifest {
                files: HashMap::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                total_files: 0,
                total_bytes: 0,
            },
            destination: "/test".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };
        let result = cert_mgr.create_backup_certificate(&backup_result);
        assert!(result.is_ok());
    }
}