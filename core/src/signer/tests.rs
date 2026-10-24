use super::*;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde_json::json;

/// Base64 PKCS#8 DER for an Ed25519 key, the body of an openssl-style PEM.
fn pkcs8_der_base64(key: &SigningKey) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    const PKCS8_PREFIX: [u8; 16] = [
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04,
        0x20,
    ];
    STANDARD.encode([&PKCS8_PREFIX[..], &key.to_bytes()].concat())
}

#[test]
fn test_canonicalize_json_deterministic() {
    let obj1 = json!({
        "z_field": "value",
        "a_field": 42,
        "nested": {
            "second": 2,
            "first": 1
        }
    });

    let obj2 = json!({
        "a_field": 42,
        "nested": {
            "first": 1,
            "second": 2
        },
        "z_field": "value"
    });

    let canonical1 = canonicalize_json(&obj1).unwrap();
    let canonical2 = canonicalize_json(&obj2).unwrap();

    assert_eq!(
        canonical1, canonical2,
        "Canonicalization must be deterministic"
    );

    // Verify structure is preserved but sorted
    let canonical_str = String::from_utf8(canonical1).unwrap();
    assert!(canonical_str.contains(r#""a_field":42"#));
    assert!(canonical_str.contains(r#""z_field":"value""#));

    // Should not contain any whitespace
    assert!(!canonical_str.chars().any(char::is_whitespace));
}

#[test]
fn test_sign_certificate_roundtrip() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let mut cert = json!({
        "cert_id": "test_123",
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

    // Sign the certificate
    sign_certificate(&mut cert, &signing_key, false).unwrap();

    // Verify signature exists
    assert!(cert.get("signature").is_some());
    let sig_obj = cert.get("signature").unwrap();
    assert_eq!(sig_obj.get("alg").unwrap().as_str().unwrap(), "Ed25519");
    assert_eq!(
        sig_obj.get("pubkey_id").unwrap().as_str().unwrap(),
        "sih_root_v1"
    );
    assert_eq!(
        sig_obj.get("canonicalization").unwrap().as_str().unwrap(),
        "RFC8785_JSON"
    );

    // Verify signature is valid base64
    let sig_b64 = sig_obj.get("sig").unwrap().as_str().unwrap();
    let decoded = STANDARD.decode(sig_b64).unwrap();
    assert_eq!(decoded.len(), 64); // Ed25519 signatures are 64 bytes

    // Verify signature cryptographically
    let is_valid = verify_certificate_signature(&cert, verifying_key.as_bytes()).unwrap();
    assert!(is_valid, "Signature verification should succeed");
}

#[test]
fn test_sign_certificate_already_signed() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let mut cert = json!({
        "cert_id": "test_456",
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "existing_signature"
        }
    });

    // Should fail without force
    let result = sign_certificate(&mut cert, &signing_key, false);
    assert!(matches!(result.unwrap_err(), SignerError::AlreadySigned));

    // Should succeed with force
    let result = sign_certificate(&mut cert, &signing_key, true);
    assert!(result.is_ok());
}

#[test]
fn test_generate_test_keys() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    // Print keys for manual testing
    println!("\nTest keys generated:");
    println!("Signing key: {}", STANDARD.encode(signing_key.to_bytes()));
    println!(
        "Verifying key: {}",
        STANDARD.encode(verifying_key.to_bytes())
    );

    // Simple PEM format (just base64 of raw key bytes)
    let pubkey_pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        STANDARD.encode(verifying_key.to_bytes())
    );
    println!("Public key PEM:\n{}", pubkey_pem);

    // Test that keys work
    let mut test_cert = json!({
        "cert_id": "test_keygen",
        "test_field": "value"
    });

    sign_certificate(&mut test_cert, &signing_key, false).unwrap();
    let is_valid = verify_certificate_signature(&test_cert, verifying_key.as_bytes()).unwrap();
    assert!(is_valid);
}

#[test]
fn test_golden_canonicalization() {
    // Golden test with known expected output
    let test_obj = json!({
        "cert_type": "backup",
        "cert_id": "backup_001",
        "created_at": "2023-01-01T00:00:00Z"
    });

    let canonical = canonicalize_json(&test_obj).unwrap();
    let canonical_str = String::from_utf8(canonical).unwrap();

    // Expected canonical form (keys sorted, no whitespace)
    let expected =
        r#"{"cert_id":"backup_001","cert_type":"backup","created_at":"2023-01-01T00:00:00Z"}"#;
    assert_eq!(canonical_str, expected);
}

#[test]
fn test_load_private_key_pem_format() {
    use tempfile::NamedTempFile;

    // Test valid Ed25519 PEM format, built from a key generated for this test
    let der_base64 = pkcs8_der_base64(&SigningKey::generate(&mut OsRng));
    let valid_pem = format!("-----BEGIN PRIVATE KEY-----\n{der_base64}\n-----END PRIVATE KEY-----");

    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, valid_pem.as_bytes()).unwrap();

    let signing_key = load_private_key(Some(temp_file.path().to_path_buf())).unwrap();

    // Should be able to use the key for signing
    let test_data = b"test message";
    let _signature = signing_key.sign(test_data);

    // Test invalid PEM format (missing headers)
    let invalid_pem = der_base64;
    let mut temp_file2 = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file2, invalid_pem.as_bytes()).unwrap();

    let result = load_private_key(Some(temp_file2.path().to_path_buf()));
    assert!(matches!(
        result.unwrap_err(),
        SignerError::InvalidKeyFormat(_)
    ));

    // Test non-PEM file
    let non_pem = "This is not a PEM file";
    let mut temp_file3 = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file3, non_pem.as_bytes()).unwrap();

    let result3 = load_private_key(Some(temp_file3.path().to_path_buf()));
    assert!(matches!(
        result3.unwrap_err(),
        SignerError::InvalidKeyFormat(_)
    ));
}

#[test]
fn test_signature_verification_invalid_cases() {
    let cert_no_sig = json!({
        "cert_id": "test"
    });

    let dummy_pubkey = [0u8; 32];
    let result = verify_certificate_signature(&cert_no_sig, &dummy_pubkey);
    assert!(matches!(
        result.unwrap_err(),
        SignerError::SignatureError(_)
    ));

    let cert_bad_alg = json!({
        "cert_id": "test",
        "signature": {
            "alg": "RSA",
            "sig": "dGVzdA=="
        }
    });

    let result = verify_certificate_signature(&cert_bad_alg, &dummy_pubkey);
    assert!(matches!(
        result.unwrap_err(),
        SignerError::SignatureError(_)
    ));
}
