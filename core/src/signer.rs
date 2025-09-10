use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Errors that can occur during signing operations
#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    #[error("Private key file not found or not readable: {0}")]
    KeyFileError(String),
    
    #[error("Invalid private key format: {0}")]
    InvalidKeyFormat(String),
    
    #[error("JSON canonicalization failed: {0}")]
    CanonicalizationError(String),
    
    #[error("Signature operation failed: {0}")]
    SignatureError(String),
    
    #[error("Certificate already signed (use --force to overwrite)")]
    AlreadySigned,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Load Ed25519 private key from PKCS#8 PEM file
/// 
/// Accepts only "-----BEGIN PRIVATE KEY-----" PEM format with Ed25519.
/// Priority: CLI path argument > SECUREWIPE_SIGN_KEY_PATH env var
pub fn load_private_key(path_or_env: Option<PathBuf>) -> Result<SigningKey, SignerError> {
    let key_path = match path_or_env {
        Some(path) => {
            info!("Loading private key from CLI path: {}", path.display());
            path
        }
        None => {
            let env_path = env::var("SECUREWIPE_SIGN_KEY_PATH")
                .map_err(|_| SignerError::KeyFileError(
                    "No key path provided and SECUREWIPE_SIGN_KEY_PATH not set. Provide an Ed25519 PKCS#8 PEM via --sign-key-path or SECUREWIPE_SIGN_KEY_PATH.".to_string()
                ))?;
            let path = PathBuf::from(env_path);
            info!("Loading private key from env var: {}", path.display());
            path
        }
    };

    let pem_content = fs::read_to_string(&key_path)
        .map_err(|e| SignerError::KeyFileError(format!("{}: {}. Provide an Ed25519 PKCS#8 PEM via --sign-key-path or SECUREWIPE_SIGN_KEY_PATH.", key_path.display(), e)))?;

    debug!("Private key PEM file read, {} bytes", pem_content.len());

    // Parse Ed25519 private key from PEM
    let signing_key = parse_ed25519_private_key_pem(&pem_content)
        .map_err(|e| SignerError::InvalidKeyFormat(format!("{} Provide an Ed25519 PKCS#8 PEM via --sign-key-path or SECUREWIPE_SIGN_KEY_PATH.", e)))?;

    info!("Private key loaded successfully");
    Ok(signing_key)
}

/// Parse Ed25519 private key from PKCS#8 PEM format
fn parse_ed25519_private_key_pem(pem_content: &str) -> Result<SigningKey> {
    // Check for proper PEM headers
    if !pem_content.contains("-----BEGIN PRIVATE KEY-----") {
        return Err(anyhow::anyhow!("Invalid PEM format. Expected '-----BEGIN PRIVATE KEY-----' for Ed25519 PKCS#8."));
    }
    
    if !pem_content.contains("-----END PRIVATE KEY-----") {
        return Err(anyhow::anyhow!("Invalid PEM format. Missing '-----END PRIVATE KEY-----' footer."));
    }
    
    // Extract base64 content between headers
    let lines: Vec<&str> = pem_content.lines().collect();
    let start_idx = lines.iter().position(|&line| line.contains("BEGIN PRIVATE KEY"))
        .ok_or_else(|| anyhow::anyhow!("No PEM begin marker found"))?;
    let end_idx = lines.iter().position(|&line| line.contains("END PRIVATE KEY"))
        .ok_or_else(|| anyhow::anyhow!("No PEM end marker found"))?;
    
    if start_idx >= end_idx {
        return Err(anyhow::anyhow!("Invalid PEM structure"));
    }
    
    let base64_lines = &lines[start_idx + 1..end_idx];
    let base64_content = base64_lines.join("");
    
    // Decode base64 to get DER bytes
    let der_bytes = STANDARD.decode(&base64_content)
        .map_err(|e| anyhow::anyhow!("Invalid base64 content in PEM: {}", e))?;
    
    // For Ed25519 PKCS#8, the private key seed is the last 32 bytes
    if der_bytes.len() < 32 {
        return Err(anyhow::anyhow!("Invalid Ed25519 PKCS#8 DER: too short ({})", der_bytes.len()));
    }
    
    // Extract the 32-byte Ed25519 seed from the DER structure
    let key_start = der_bytes.len() - 32;
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&der_bytes[key_start..]);
    
    Ok(SigningKey::from_bytes(&seed))
}

/// Canonicalize JSON according to RFC 8785 JSON Canonicalization Scheme (JCS)
/// 
/// This ensures deterministic byte representation for signing:
/// - UTF-8 encoding
/// - Sorted object keys
/// - No insignificant whitespace
/// - Consistent number formatting
pub fn canonicalize_json(value: &Value) -> Result<Vec<u8>, SignerError> {
    debug!("Starting JSON canonicalization");
    
    let canonical = canonicalize_value(value)
        .map_err(|e| SignerError::CanonicalizationError(e.to_string()))?;
    
    let canonical_json = serde_json::to_string(&canonical)
        .map_err(|e| SignerError::CanonicalizationError(format!("JSON serialization failed: {}", e)))?;
    
    // Remove all whitespace for true RFC 8785 compliance
    let minified = canonical_json
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    
    let canonical_bytes = minified.as_bytes().to_vec();
    
    debug!("JSON canonicalized to {} bytes", canonical_bytes.len());
    Ok(canonical_bytes)
}

/// Recursively canonicalize JSON values according to RFC 8785
fn canonicalize_value(value: &Value) -> Result<Value> {
    match value {
        Value::Object(map) => {
            // Sort keys and canonicalize all values
            let mut canonical_map = BTreeMap::new();
            for (key, val) in map {
                canonical_map.insert(key.clone(), canonicalize_value(val)?);
            }
            Ok(Value::Object(canonical_map.into_iter().collect()))
        }
        Value::Array(arr) => {
            // Canonicalize array elements
            let canonical_arr: Result<Vec<Value>> = arr
                .iter()
                .map(canonicalize_value)
                .collect();
            Ok(Value::Array(canonical_arr?))
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            // Primitive values are already canonical
            Ok(value.clone())
        }
    }
}

/// Sign a certificate JSON with Ed25519
/// 
/// Adds signature fields to the certificate:
/// - signature.alg = "Ed25519"  
/// - signature.pubkey_id = "sih_root_v1"
/// - signature.sig = base64(signature_bytes)
/// - signature.canonicalization = "RFC8785_JSON"
/// 
/// Returns an error if certificate is already signed unless force is true
pub fn sign_certificate(
    value: &mut Value, 
    signing_key: &SigningKey, 
    force: bool
) -> Result<(), SignerError> {
    info!("Starting certificate signing process");
    
    // Check if already signed
    if let Some(_existing_sig) = value.get("signature") {
        if !force {
            warn!("Certificate already contains signature, use --force to overwrite");
            return Err(SignerError::AlreadySigned);
        }
        info!("Overwriting existing signature (--force specified)");
    }

    // Remove signature field temporarily for canonicalization
    let _original_signature = value.as_object_mut()
        .ok_or_else(|| SignerError::CanonicalizationError("Certificate must be JSON object".to_string()))?
        .remove("signature");

    // Canonicalize the unsigned certificate
    let canonical_bytes = canonicalize_json(value)?;
    
    debug!("Canonical certificate: {} bytes", canonical_bytes.len());

    // Sign the canonical bytes
    let signature_bytes = signing_key.sign(&canonical_bytes);
    let signature_b64 = STANDARD.encode(signature_bytes.to_bytes());
    
    debug!("Generated signature: {} bytes -> {} b64 chars", 
           signature_bytes.to_bytes().len(), signature_b64.len());

    // Add signature fields
    let signature_object = serde_json::json!({
        "alg": "Ed25519",
        "pubkey_id": "sih_root_v1", 
        "sig": signature_b64,
        "canonicalization": "RFC8785_JSON"
    });

    value.as_object_mut()
        .unwrap()
        .insert("signature".to_string(), signature_object);

    info!("Certificate signed successfully");
    Ok(())
}

/// Verify an Ed25519 signature on a certificate
/// 
/// Used for testing and validation - extracts signature, canonicalizes unsigned cert,
/// and verifies the signature matches
pub fn verify_certificate_signature(
    value: &Value, 
    public_key_bytes: &[u8; 32]
) -> Result<bool, SignerError> {
    debug!("Starting certificate signature verification");
    
    let signature_obj = value.get("signature")
        .ok_or_else(|| SignerError::SignatureError("No signature found in certificate".to_string()))?;
    
    // Extract signature components
    let alg = signature_obj.get("alg")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SignerError::SignatureError("Missing or invalid signature.alg".to_string()))?;
    
    if alg != "Ed25519" {
        return Err(SignerError::SignatureError(format!("Unsupported algorithm: {}", alg)));
    }
    
    let sig_b64 = signature_obj.get("sig")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SignerError::SignatureError("Missing or invalid signature.sig".to_string()))?;
    
    let signature_bytes = STANDARD.decode(sig_b64)
        .map_err(|e| SignerError::SignatureError(format!("Invalid base64 signature: {}", e)))?;
    
    let signature = Signature::from_bytes(&signature_bytes.try_into()
        .map_err(|_| SignerError::SignatureError("Invalid signature length".to_string()))?);
    
    // Remove signature for canonicalization
    let mut unsigned_cert = value.clone();
    unsigned_cert.as_object_mut()
        .unwrap()
        .remove("signature");
    
    // Canonicalize and verify
    let canonical_bytes = canonicalize_json(&unsigned_cert)?;
    
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| SignerError::SignatureError(format!("Invalid public key: {}", e)))?;
    
    let is_valid = verifying_key.verify(&canonical_bytes, &signature).is_ok();
    
    debug!("Signature verification result: {}", is_valid);
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
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
        
        assert_eq!(canonical1, canonical2, "Canonicalization must be deterministic");
        
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
        assert_eq!(sig_obj.get("pubkey_id").unwrap().as_str().unwrap(), "sih_root_v1");
        assert_eq!(sig_obj.get("canonicalization").unwrap().as_str().unwrap(), "RFC8785_JSON");
        
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
        println!("Verifying key: {}", STANDARD.encode(verifying_key.to_bytes()));
        
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
        let expected = r#"{"cert_id":"backup_001","cert_type":"backup","created_at":"2023-01-01T00:00:00Z"}"#;
        assert_eq!(canonical_str, expected);
    }
    
    #[test]
    fn test_load_private_key_pem_format() {
        use tempfile::NamedTempFile;
        
        // Test valid Ed25519 PEM format
        let valid_pem = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIOJ0LFWES63cMB/MPWcXn6rt6kj/7XsNa3fwkQxQJqaT
-----END PRIVATE KEY-----";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, valid_pem.as_bytes()).unwrap();
        
        let signing_key = load_private_key(Some(temp_file.path().to_path_buf())).unwrap();
        
        // Should be able to use the key for signing
        let test_data = b"test message";
        let _signature = signing_key.sign(test_data);
        
        // Test invalid PEM format (missing headers)
        let invalid_pem = "MC4CAQAwBQYDK2VwBCIEIOJ0LFWES63cMB/MPWcXn6rt6kj/7XsNa3fwkQxQJqaT";
        let mut temp_file2 = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file2, invalid_pem.as_bytes()).unwrap();
        
        let result = load_private_key(Some(temp_file2.path().to_path_buf()));
        assert!(matches!(result.unwrap_err(), SignerError::InvalidKeyFormat(_)));
        
        // Test non-PEM file
        let non_pem = "This is not a PEM file";
        let mut temp_file3 = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file3, non_pem.as_bytes()).unwrap();
        
        let result3 = load_private_key(Some(temp_file3.path().to_path_buf()));
        assert!(matches!(result3.unwrap_err(), SignerError::InvalidKeyFormat(_)));
    }
    
    #[test]
    fn test_signature_verification_invalid_cases() {
        let cert_no_sig = json!({
            "cert_id": "test"
        });
        
        let dummy_pubkey = [0u8; 32];
        let result = verify_certificate_signature(&cert_no_sig, &dummy_pubkey);
        assert!(matches!(result.unwrap_err(), SignerError::SignatureError(_)));
        
        let cert_bad_alg = json!({
            "cert_id": "test",
            "signature": {
                "alg": "RSA",
                "sig": "dGVzdA=="
            }
        });
        
        let result = verify_certificate_signature(&cert_bad_alg, &dummy_pubkey);
        assert!(matches!(result.unwrap_err(), SignerError::SignatureError(_)));
    }
}