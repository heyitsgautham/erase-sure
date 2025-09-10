use securewipe::signer::{load_private_key, canonicalize_json, sign_certificate, verify_certificate_signature};
use securewipe::{BackupCertificate, WipeCertificate, CertificateSignature};
use serde_json::json;
use tempfile::NamedTempFile;
use std::io::Write;
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;

#[test]
fn test_end_to_end_backup_certificate_signing() {
    // Create a test private key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    // Write key to temporary file
    let mut key_file = NamedTempFile::new().unwrap();
    key_file.write_all(&signing_key.to_bytes()).unwrap();
    
    // Create a backup certificate
    let backup_cert = BackupCertificate {
        cert_id: "integration_test_backup_001".to_string(),
        cert_type: "backup".to_string(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        device: json!({
            "model": "Test SSD 1TB",
            "serial": "TEST123456789",
            "capacity_bytes": 1000204886016u64
        }),
        backup_summary: json!({
            "files": 1542,
            "bytes": 850394752u64,
            "duration_seconds": 120
        }),
        manifest_sha256: "a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        encryption_method: "AES-256-CTR".to_string(),
        signature: CertificateSignature {
            alg: "placeholder".to_string(),
            pubkey_id: "placeholder".to_string(),
            sig: "placeholder".to_string(),
        },
    };
    
    // Convert to JSON for signing
    let mut cert_value = serde_json::to_value(&backup_cert).unwrap();
    
    // Load private key
    let loaded_key = load_private_key(Some(key_file.path().to_path_buf())).unwrap();
    
    // Sign the certificate
    sign_certificate(&mut cert_value, &loaded_key, true).unwrap();
    
    // Verify signature exists and is properly formatted
    let signature_obj = cert_value.get("signature").unwrap();
    assert_eq!(signature_obj.get("alg").unwrap().as_str().unwrap(), "Ed25519");
    assert_eq!(signature_obj.get("pubkey_id").unwrap().as_str().unwrap(), "sih_root_v1");
    assert_eq!(signature_obj.get("canonicalization").unwrap().as_str().unwrap(), "RFC8785_JSON");
    
    let sig_b64 = signature_obj.get("sig").unwrap().as_str().unwrap();
    assert!(sig_b64.len() > 80); // Base64 encoded 64-byte signature should be ~88 chars
    
    // Verify the signature cryptographically
    let is_valid = verify_certificate_signature(&cert_value, verifying_key.as_bytes()).unwrap();
    assert!(is_valid, "Certificate signature should be valid");
}

#[test]
fn test_end_to_end_wipe_certificate_signing() {
    // Create a test private key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    // Write key to temporary file
    let mut key_file = NamedTempFile::new().unwrap();
    key_file.write_all(&signing_key.to_bytes()).unwrap();
    
    // Create a wipe certificate
    let wipe_cert = WipeCertificate {
        cert_id: "integration_test_wipe_001".to_string(),
        cert_type: "wipe".to_string(),
        created_at: "2023-01-01T01:00:00Z".to_string(),
        device: json!({
            "model": "Test SSD 1TB",
            "serial": "TEST123456789",
            "capacity_bytes": 1000204886016u64
        }),
        wipe_summary: json!({
            "policy": "PURGE",
            "method": "nvme_format_crypto_erase",
            "verification_samples": 128,
            "verification_passed": true,
            "duration_seconds": 45
        }),
        linkage: Some(json!({
            "backup_cert_id": "integration_test_backup_001"
        })),
        signature: CertificateSignature {
            alg: "placeholder".to_string(),
            pubkey_id: "placeholder".to_string(),
            sig: "placeholder".to_string(),
        },
    };
    
    // Convert to JSON for signing
    let mut cert_value = serde_json::to_value(&wipe_cert).unwrap();
    
    // Load private key
    let loaded_key = load_private_key(Some(key_file.path().to_path_buf())).unwrap();
    
    // Sign the certificate
    sign_certificate(&mut cert_value, &loaded_key, true).unwrap();
    
    // Verify signature exists and is properly formatted
    let signature_obj = cert_value.get("signature").unwrap();
    assert_eq!(signature_obj.get("alg").unwrap().as_str().unwrap(), "Ed25519");
    assert_eq!(signature_obj.get("pubkey_id").unwrap().as_str().unwrap(), "sih_root_v1");
    assert_eq!(signature_obj.get("canonicalization").unwrap().as_str().unwrap(), "RFC8785_JSON");
    
    // Verify the signature cryptographically  
    let is_valid = verify_certificate_signature(&cert_value, verifying_key.as_bytes()).unwrap();
    assert!(is_valid, "Certificate signature should be valid");
}

#[test]
fn test_canonicalization_determinism_complex_cert() {
    // Test with realistic certificate data to ensure canonicalization is truly deterministic
    let cert1 = json!({
        "wipe_summary": {
            "verification_passed": true,
            "policy": "PURGE",
            "method": "nvme_sanitize"
        },
        "cert_type": "wipe",
        "device": {
            "serial": "ABC123",
            "model": "Samsung SSD",
            "capacity_bytes": 1000000000u64
        },
        "cert_id": "test_001"
    });
    
    let cert2 = json!({
        "cert_id": "test_001", 
        "cert_type": "wipe",
        "device": {
            "capacity_bytes": 1000000000u64,
            "model": "Samsung SSD",
            "serial": "ABC123"
        },
        "wipe_summary": {
            "method": "nvme_sanitize",
            "policy": "PURGE", 
            "verification_passed": true
        }
    });
    
    let canonical1 = canonicalize_json(&cert1).unwrap();
    let canonical2 = canonicalize_json(&cert2).unwrap();
    
    assert_eq!(canonical1, canonical2, "Canonicalization must be deterministic regardless of field order");
    
    // Verify the canonical form has expected properties
    let canonical_str = String::from_utf8(canonical1).unwrap();
    assert!(!canonical_str.chars().any(char::is_whitespace), "Canonical form should have no whitespace");
    
    // Keys should be sorted at each level
    assert!(canonical_str.contains(r#""cert_id":"test_001""#));
    assert!(canonical_str.contains(r#""cert_type":"wipe""#)); 
    // Device keys should be sorted: capacity_bytes, model, serial
    let device_section = canonical_str.find(r#""device":"#).unwrap();
    let after_device = &canonical_str[device_section..];
    let capacity_pos = after_device.find("capacity_bytes").unwrap();
    let model_pos = after_device.find("model").unwrap(); 
    let serial_pos = after_device.find("serial").unwrap();
    assert!(capacity_pos < model_pos && model_pos < serial_pos, "Device keys should be sorted");
}

#[test]
fn test_env_var_key_loading() {
    use std::env;
    
    // Create a test private key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    
    // Write key to temporary file
    let mut key_file = NamedTempFile::new().unwrap();
    key_file.write_all(&signing_key.to_bytes()).unwrap();
    
    // Set environment variable
    env::set_var("SECUREWIPE_SIGN_KEY_PATH", key_file.path().to_str().unwrap());
    
    // Load key without providing path (should use env var)
    let loaded_key = load_private_key(None).unwrap();
    
    // Verify keys are equivalent by signing the same data
    let test_data = b"test message for key comparison";
    let orig_sig = signing_key.sign(test_data);
    let loaded_sig = loaded_key.sign(test_data);
    
    // They should produce identical signatures since they're the same key
    assert_eq!(orig_sig.to_bytes(), loaded_sig.to_bytes());
    
    // Clean up
    env::remove_var("SECUREWIPE_SIGN_KEY_PATH");
}

#[test]
fn test_signature_tampering_detection() {
    // Create a test private key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    // Create and sign a certificate
    let mut cert = json!({
        "cert_id": "tamper_test_001",
        "cert_type": "backup",
        "created_at": "2023-01-01T00:00:00Z",
        "data": "sensitive information"
    });
    
    sign_certificate(&mut cert, &signing_key, false).unwrap();
    
    // Verify original is valid
    let is_valid = verify_certificate_signature(&cert, verifying_key.as_bytes()).unwrap();
    assert!(is_valid, "Original certificate should be valid");
    
    // Tamper with the certificate data
    cert["data"] = json!("tampered information");
    
    // Signature should no longer be valid
    let is_valid_after_tamper = verify_certificate_signature(&cert, verifying_key.as_bytes()).unwrap();
    assert!(!is_valid_after_tamper, "Tampered certificate should be invalid");
}

#[test] 
fn test_cross_key_verification_failure() {
    // Create two different private keys
    let mut csprng = OsRng;
    let signing_key1 = SigningKey::generate(&mut csprng);
    let signing_key2 = SigningKey::generate(&mut csprng);
    let verifying_key2 = signing_key2.verifying_key();
    
    // Sign certificate with key 1
    let mut cert = json!({
        "cert_id": "cross_key_test",
        "cert_type": "backup"
    });
    
    sign_certificate(&mut cert, &signing_key1, false).unwrap();
    
    // Try to verify with key 2 - should fail
    let is_valid = verify_certificate_signature(&cert, verifying_key2.as_bytes()).unwrap();
    assert!(!is_valid, "Certificate signed with key1 should not verify with key2");
}
