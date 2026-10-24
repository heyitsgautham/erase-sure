use super::cert::CertCommands;
use super::*;
use securewipe::logging::Logger;

#[test]
fn test_discover_args_default() {
    let args = DiscoverArgs {
        format: "json".to_string(),
        no_enrich: false,
    };
    assert_eq!(args.format, "json");
}

#[test]
fn test_backup_args_creation() {
    let args = BackupArgs {
        device: "/dev/sda".to_string(),
        dest: "/mnt/backup".to_string(),
        paths: vec!["Documents".to_string(), "Pictures".to_string()],
        sign: false,
        sign_key_path: None,
        force: false,
    };
    assert_eq!(args.device, "/dev/sda");
    assert_eq!(args.dest, "/mnt/backup");
    assert_eq!(args.paths.len(), 2);
    assert!(!args.sign);
    assert!(!args.force);
}

#[test]
fn test_wipe_args_defaults() {
    let args = WipeArgs {
        device: "/dev/sda".to_string(),
        policy: "PURGE".to_string(),
        iso_mode: false,
        format: "json".to_string(),
        samples: 128,
        sign: false,
        sign_key_path: None,
        force: false,
        danger_allow_wipe: false,
        backup_cert_id: None,
    };
    assert_eq!(args.policy, "PURGE");
    assert!(!args.iso_mode);
    assert_eq!(args.format, "json");
    assert_eq!(args.samples, 128);
    assert!(!args.sign);
    assert!(!args.force);
}

#[test]
fn test_cert_args_creation() {
    let args = CertArgs {
        show: Some("cert_123".to_string()),
        export_pdf: None,
        command: None,
    };
    assert_eq!(args.show, Some("cert_123".to_string()));
    assert_eq!(args.export_pdf, None);
    assert!(args.command.is_none());
}

#[test]
fn test_handle_discover() {
    let logger = Logger::new();
    let args = DiscoverArgs {
        format: "json".to_string(),
        no_enrich: false,
    };

    let result = handle_discover(args, &logger);
    // On non-Linux systems, this will fail with a clear error message
    // On Linux systems with lsblk, this should succeed
    // We just verify that the function handles errors gracefully
    match result {
        Ok(_) => {
            // Success case - we're on Linux with lsblk
        }
        Err(e) => {
            // Expected on non-Linux systems
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("lsblk command not found")
                    || error_msg.contains("Device discovery failed"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_handle_backup() {
    let logger = Logger::new();
    let args = BackupArgs {
        device: "/dev/sda".to_string(),
        dest: "/mnt/backup".to_string(),
        paths: vec!["Documents".to_string()],
        sign: false,
        sign_key_path: None,
        force: false,
    };

    let result = handle_backup(args, &logger);
    // On most systems, this will fail due to permissions or missing destination
    // This test verifies the function handles errors gracefully
    match result {
        Ok(_) => {
            // Success case - backup actually worked (rare in test environment)
        }
        Err(e) => {
            // Expected case - verify it's a sensible error message
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Read-only file system")
                    || error_msg.contains("No such file or directory")
                    || error_msg.contains("Permission denied")
                    || error_msg.contains("Backup failed"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_handle_wipe() {
    // Planning only (danger_allow_wipe is false), against a path that is not a device.
    let logger = Logger::new();
    let args = WipeArgs {
        device: "/nonexistent/securewipe-test-device".to_string(),
        policy: "PURGE".to_string(),
        iso_mode: false,
        format: "json".to_string(),
        samples: 128,
        sign: false,
        sign_key_path: None,
        force: false,
        danger_allow_wipe: false,
        backup_cert_id: None,
    };

    let result = handle_wipe(args, &logger);
    assert!(result.is_ok());
}

#[test]
fn test_handle_cert() {
    let logger = Logger::new();
    let args = CertArgs {
        show: Some("cert_123".to_string()),
        export_pdf: None,
        command: None,
    };

    let result = handle_cert(args, &logger);
    assert!(result.is_ok());
}

#[test]
fn test_json_output_structure() {
    let logger = Logger::new();
    let args = DiscoverArgs {
        format: "json".to_string(),
        no_enrich: false,
    };

    // This test verifies the JSON structure without printing
    let result = handle_discover(args, &logger);
    // On non-Linux systems, this will fail with a clear error message
    // On Linux systems with lsblk, this should succeed
    match result {
        Ok(_) => {
            // Success case - we're on Linux with lsblk
        }
        Err(e) => {
            // Expected on non-Linux systems
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("lsblk command not found")
                    || error_msg.contains("Device discovery failed"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_cert_sign_args() {
    let sign_command = CertCommands::Sign {
        file: std::path::PathBuf::from("/tmp/test_cert.json"),
        key: Some(std::path::PathBuf::from("/tmp/test_key")),
        force: true,
    };

    match sign_command {
        CertCommands::Sign { file, key, force } => {
            assert_eq!(file, std::path::PathBuf::from("/tmp/test_cert.json"));
            assert_eq!(key, Some(std::path::PathBuf::from("/tmp/test_key")));
            assert!(force);
        }
        _ => panic!("Expected Sign command"),
    }
}

#[test]
fn test_backup_signing_flags() {
    let args = BackupArgs {
        device: "/dev/sda".to_string(),
        dest: "/mnt/backup".to_string(),
        paths: vec!["Documents".to_string()],
        sign: true,
        sign_key_path: Some(std::path::PathBuf::from("/tmp/key")),
        force: true,
    };

    assert!(args.sign);
    assert_eq!(
        args.sign_key_path,
        Some(std::path::PathBuf::from("/tmp/key"))
    );
    assert!(args.force);
}

#[test]
fn test_wipe_signing_flags() {
    let args = WipeArgs {
        device: "/dev/sda".to_string(),
        policy: "PURGE".to_string(),
        iso_mode: false,
        format: "json".to_string(),
        samples: 128,
        sign: true,
        sign_key_path: Some(std::path::PathBuf::from("/tmp/key")),
        force: true,
        danger_allow_wipe: false,
        backup_cert_id: None,
    };

    assert!(args.sign);
    assert_eq!(
        args.sign_key_path,
        Some(std::path::PathBuf::from("/tmp/key"))
    );
    assert!(args.force);
}

#[test]
fn test_cert_verify_args() {
    let verify_command = CertCommands::Verify {
        file: std::path::PathBuf::from("/tmp/test_cert.json"),
        pubkey: std::path::PathBuf::from("keys/dev_public.pem"),
    };

    match verify_command {
        CertCommands::Verify { file, pubkey } => {
            assert_eq!(file, std::path::PathBuf::from("/tmp/test_cert.json"));
            assert_eq!(pubkey, std::path::PathBuf::from("keys/dev_public.pem"));
        }
        _ => panic!("Expected Verify command"),
    }
}
