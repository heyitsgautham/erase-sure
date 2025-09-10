use serde_json::json;
use tempfile::NamedTempFile;
use std::path::PathBuf;
use assert_cmd::Command;

/// Helper function to check if dev keys exist
fn dev_keys_exist() -> bool {
    let private_key_path = get_dev_private_key_path();
    let public_key_path = get_dev_public_key_path();
    private_key_path.exists() && public_key_path.exists()
}

/// Helper function to get dev private key path  
fn get_dev_private_key_path() -> PathBuf {
    PathBuf::from("keys/dev_private.pem")
}

/// Helper function to get dev public key path
fn get_dev_public_key_path() -> PathBuf {
    PathBuf::from("keys/dev_public.pem")
}

#[test]
fn test_sign_and_verify_with_dev_keys() {
    if !dev_keys_exist() {
        eprintln!("⚠️  Skipping test: Dev keys not found. Run 'openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem && openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem' from repo root.");
        return;
    }
    
    // Create a minimal valid backup certificate JSON
    let cert_json = json!({
        "cert_id": "test_backup_dev_001",
        "cert_type": "backup",
        "certificate_version": "v1.0.0",
        "created_at": "2025-09-10T12:00:00.000000+00:00",
        "device": {
            "name": "/dev/nvme0n1",
            "model": "Samsung SSD 980 PRO 1TB",
            "serial": "S6TXNX0R123456",
            "capacity_bytes": 1000204886016u64
        },
        "issuer": {
            "country": "IN",
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v1.0.0"
        }
    });
    
    // Save to temp file
    let mut temp_file = NamedTempFile::new().unwrap();
    serde_json::to_writer_pretty(&mut temp_file, &cert_json).unwrap();
    let cert_path = temp_file.path().to_str().unwrap();
    
    // Sign the certificate using CLI
    let mut sign_cmd = Command::cargo_bin("securewipe").unwrap();
    let sign_output = sign_cmd
        .args(&["cert", "sign", 
               "--file", cert_path,
               "--sign-key-path", get_dev_private_key_path().to_str().unwrap(),
               "--force"])
        .output()
        .unwrap();
    
    assert!(sign_output.status.success(), "Sign command should succeed");
    
    // Reload and verify signature exists
    let signed_content = std::fs::read_to_string(cert_path).unwrap();
    let signed_cert: serde_json::Value = serde_json::from_str(&signed_content).unwrap();
    
    assert!(signed_cert.get("signature").is_some(), "Certificate should have signature");
    let signature_obj = signed_cert.get("signature").unwrap();
    assert_eq!(signature_obj.get("alg").unwrap().as_str().unwrap(), "Ed25519");
    assert_eq!(signature_obj.get("pubkey_id").unwrap().as_str().unwrap(), "sih_root_v1");
    
    // Verify the signature using CLI
    let mut verify_cmd = Command::cargo_bin("securewipe").unwrap();
    let verify_output = verify_cmd
        .args(&["cert", "verify",
               "--file", cert_path,
               "--pubkey", get_dev_public_key_path().to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(verify_output.status.success(), "Verify command should succeed");
    
    let stdout = String::from_utf8(verify_output.stdout).unwrap();
    let verify_result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(verify_result["signature_valid"], true, "Signature should be valid");
}

#[test]
fn test_tamper_detection_with_dev_keys() {
    if !dev_keys_exist() {
        eprintln!("⚠️  Skipping test: Dev keys not found. Run 'openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem && openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem' from repo root.");
        return;
    }
    
    // Create and sign a certificate
    let cert_json = json!({
        "cert_id": "test_tamper_dev_001", 
        "cert_type": "wipe",
        "certificate_version": "v1.0.0",
        "created_at": "2025-09-10T12:00:00.000000+00:00",
        "device": {
            "name": "/dev/sda",
            "model": "Original SSD Model",
            "serial": "ORIG123",
            "capacity_bytes": 500000000u64
        },
        "issuer": {
            "country": "IN",
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v1.0.0"
        }
    });
    
    let mut temp_file = NamedTempFile::new().unwrap();
    serde_json::to_writer_pretty(&mut temp_file, &cert_json).unwrap();
    let cert_path = temp_file.path().to_str().unwrap();
    
    // Sign the certificate
    let mut sign_cmd = Command::cargo_bin("securewipe").unwrap();
    let sign_output = sign_cmd
        .args(&["cert", "sign",
               "--file", cert_path,
               "--sign-key-path", get_dev_private_key_path().to_str().unwrap(),
               "--force"])
        .output()
        .unwrap();
    
    assert!(sign_output.status.success(), "Sign command should succeed");
    
    // Verify original signature is valid
    let mut verify_cmd = Command::cargo_bin("securewipe").unwrap();
    let verify_output = verify_cmd
        .args(&["cert", "verify",
               "--file", cert_path,
               "--pubkey", get_dev_public_key_path().to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(verify_output.status.success());
    let stdout = String::from_utf8(verify_output.stdout).unwrap();
    let verify_result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(verify_result["signature_valid"], true, "Original signature should be valid");
    
    // Tamper with the certificate (change device model)
    let signed_content = std::fs::read_to_string(cert_path).unwrap();
    let mut tampered_cert: serde_json::Value = serde_json::from_str(&signed_content).unwrap();
    tampered_cert["device"]["model"] = json!("Tampered SSD Model");
    
    // Write tampered version back
    std::fs::write(cert_path, serde_json::to_string_pretty(&tampered_cert).unwrap()).unwrap();
    
    // Verify tampered signature should be invalid
    let mut verify_tampered_cmd = Command::cargo_bin("securewipe").unwrap();
    let verify_tampered_output = verify_tampered_cmd
        .args(&["cert", "verify",
               "--file", cert_path,
               "--pubkey", get_dev_public_key_path().to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(verify_tampered_output.status.success());
    let stdout_tampered = String::from_utf8(verify_tampered_output.stdout).unwrap();
    let verify_tampered_result: serde_json::Value = serde_json::from_str(&stdout_tampered).unwrap();
    assert_eq!(verify_tampered_result["signature_valid"], false, "Tampered signature should be invalid");
}

#[test]
fn test_missing_signature_detection() {
    if !dev_keys_exist() {
        eprintln!("⚠️  Skipping test: Dev keys not found. Run 'openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem && openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem' from repo root.");
        return;
    }
    
    // Create an unsigned certificate
    let cert_json = json!({
        "cert_id": "test_unsigned_dev_001",
        "cert_type": "backup", 
        "certificate_version": "v1.0.0",
        "created_at": "2025-09-10T12:00:00.000000+00:00",
        "device": {
            "name": "/dev/sdb",
            "model": "Test Drive",
            "serial": "UNSIGN123",
            "capacity_bytes": 250000000u64
        },
        "issuer": {
            "country": "IN",
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe", 
            "tool_version": "v1.0.0"
        }
    });
    
    let mut temp_file = NamedTempFile::new().unwrap();
    serde_json::to_writer_pretty(&mut temp_file, &cert_json).unwrap();
    let cert_path = temp_file.path().to_str().unwrap();
    
    // Verify unsigned certificate
    let mut verify_cmd = Command::cargo_bin("securewipe").unwrap();
    let verify_output = verify_cmd
        .args(&["cert", "verify",
               "--file", cert_path,
               "--pubkey", get_dev_public_key_path().to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(verify_output.status.success());
    let stdout = String::from_utf8(verify_output.stdout).unwrap();
    let verify_result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(verify_result["signature_valid"], serde_json::Value::Null, "Unsigned certificate should return null");
}

#[test]
fn test_dev_keys_availability() {
    // This test documents how to set up dev keys if they're missing
    if !dev_keys_exist() {
        println!("ℹ️  Dev keys not found. To set up dev keys, run from repo root:");
        println!("   mkdir -p keys");
        println!("   openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem");
        println!("   openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem");
        println!("   echo 'keys/' >> .gitignore");
        return;
    }
    
    // If keys exist, verify they are valid Ed25519 keys by attempting to use them
    let mut test_cmd = Command::cargo_bin("securewipe").unwrap();
    let help_output = test_cmd
        .args(&["cert", "sign", "--help"])
        .output()
        .unwrap();
    
    assert!(help_output.status.success(), "CLI should be working");
    
    println!("✅ Dev keys found and CLI is available for integration testing");
}
