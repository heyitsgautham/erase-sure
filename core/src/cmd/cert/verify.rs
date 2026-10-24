use anyhow::Result;
use securewipe::logging::Logger;

use super::{create_verify_response, parse_ed25519_public_key_pem};

pub(super) fn handle_cert_verify(
    cert_file_path: std::path::PathBuf,
    pubkey_path: std::path::PathBuf,
    logger: &Logger,
) -> Result<()> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use securewipe::schema::CertificateValidator;
    use securewipe::signer::canonicalize_json;
    use std::fs;

    logger.log_json(&serde_json::json!({
        "step": "cert_verify_start",
        "file": cert_file_path.display().to_string(),
        "pubkey": pubkey_path.display().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Check if certificate file exists
    if !cert_file_path.exists() {
        let response = serde_json::json!({
            "op": "cert_verify",
            "file": cert_file_path.display().to_string(),
            "signature_valid": null,
            "schema_valid": null,
            "pubkey": pubkey_path.display().to_string(),
            "error": "Certificate file not found"
        });
        println!("{}", serde_json::to_string(&response)?);
        return Err(anyhow::anyhow!(
            "Certificate file not found: {}",
            cert_file_path.display()
        ));
    }

    // Check if public key file exists
    if !pubkey_path.exists() {
        let response = serde_json::json!({
            "op": "cert_verify",
            "file": cert_file_path.display().to_string(),
            "signature_valid": null,
            "schema_valid": null,
            "pubkey": pubkey_path.display().to_string(),
            "error": "Public key file not found"
        });
        println!("{}", serde_json::to_string(&response)?);
        return Err(anyhow::anyhow!(
            "Public key file not found: {}",
            pubkey_path.display()
        ));
    }

    // Read and parse certificate
    let cert_json = match fs::read_to_string(&cert_file_path) {
        Ok(json) => json,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                None,
                Some(format!("Failed to read certificate file: {}", e)),
            );
            println!("{}", serde_json::to_string(&response)?);
            return Err(anyhow::anyhow!("Failed to read certificate file: {}", e));
        }
    };

    let cert_value: serde_json::Value = match serde_json::from_str(&cert_json) {
        Ok(value) => value,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                None,
                Some(format!("Invalid JSON in certificate file: {}", e)),
            );
            println!("{}", serde_json::to_string(&response)?);
            return Err(anyhow::anyhow!("Invalid JSON in certificate file: {}", e));
        }
    };

    logger.log_json(&serde_json::json!({
        "step": "cert_loaded",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Validate schema first
    logger.log_info("Validating certificate schema");
    let validator = CertificateValidator::default();
    let validation_result = match validator.validate_certificate(&cert_value) {
        Ok(result) => result,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                None,
                Some(format!("Schema validation error: {}", e)),
            );
            println!("{}", serde_json::to_string(&response)?);
            return Ok(());
        }
    };

    let schema_valid = validation_result.valid;
    let schema_errors = if !validation_result.valid {
        Some(validation_result.errors)
    } else {
        None
    };

    // Check if signature exists
    let signature_obj = match cert_value.get("signature") {
        Some(sig) => sig,
        None => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                Some(schema_valid),
                None,
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    // Validate signature structure and algorithm
    let alg = match signature_obj.get("alg").and_then(|v| v.as_str()) {
        Some(alg) => alg,
        None => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some("Missing signature.alg field".to_string()),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    if alg != "Ed25519" {
        let response = create_verify_response(
            &cert_file_path,
            &pubkey_path,
            Some(false),
            Some(schema_valid),
            Some(format!("Unsupported algorithm: {}", alg)),
        );
        if let Some(errors) = schema_errors {
            let mut response_obj = response.as_object().unwrap().clone();
            response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(response_obj))?
            );
        } else {
            println!("{}", serde_json::to_string(&response)?);
        }
        return Ok(());
    }

    let pubkey_id = match signature_obj.get("pubkey_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some("Missing signature.pubkey_id field".to_string()),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    if pubkey_id != "sih_root_v1" {
        let response = create_verify_response(
            &cert_file_path,
            &pubkey_path,
            Some(false),
            Some(schema_valid),
            Some(format!(
                "Invalid pubkey_id: expected 'sih_root_v1', got '{}'",
                pubkey_id
            )),
        );
        if let Some(errors) = schema_errors {
            let mut response_obj = response.as_object().unwrap().clone();
            response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(response_obj))?
            );
        } else {
            println!("{}", serde_json::to_string(&response)?);
        }
        return Ok(());
    }

    let sig_b64 = match signature_obj.get("sig").and_then(|v| v.as_str()) {
        Some(sig) => sig,
        None => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some("Missing signature.sig field".to_string()),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    logger.log_json(&serde_json::json!({
        "step": "signature_validated",
        "alg": alg,
        "pubkey_id": pubkey_id,
        "schema_valid": schema_valid,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Decode signature
    let signature_bytes = match STANDARD.decode(sig_b64) {
        Ok(bytes) => bytes,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some(format!("Invalid base64 signature: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some(format!("Invalid signature format: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    // Load public key from PEM file
    let pubkey_pem = match fs::read_to_string(&pubkey_path) {
        Ok(pem) => pem,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                Some(schema_valid),
                Some(format!("Failed to read public key file: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Err(anyhow::anyhow!("Failed to read public key file: {}", e));
        }
    };

    // Parse PEM and extract public key bytes
    let pubkey_bytes = match parse_ed25519_public_key_pem(&pubkey_pem) {
        Ok(bytes) => bytes,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                Some(schema_valid),
                Some(format!("Failed to parse public key PEM: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Err(anyhow::anyhow!("Failed to parse public key PEM: {}", e));
        }
    };

    let verifying_key = match VerifyingKey::from_bytes(&pubkey_bytes) {
        Ok(key) => key,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                None,
                Some(schema_valid),
                Some(format!("Invalid public key: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Err(anyhow::anyhow!("Invalid public key: {}", e));
        }
    };

    logger.log_json(&serde_json::json!({
        "step": "pubkey_loaded",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Remove signature for canonicalization
    let mut unsigned_cert = cert_value.clone();
    unsigned_cert.as_object_mut().unwrap().remove("signature");

    // Canonicalize the unsigned certificate
    let canonical_bytes = match canonicalize_json(&unsigned_cert) {
        Ok(bytes) => bytes,
        Err(e) => {
            let response = create_verify_response(
                &cert_file_path,
                &pubkey_path,
                Some(false),
                Some(schema_valid),
                Some(format!("JSON canonicalization failed: {}", e)),
            );
            if let Some(errors) = schema_errors {
                let mut response_obj = response.as_object().unwrap().clone();
                response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::Value::Object(response_obj))?
                );
            } else {
                println!("{}", serde_json::to_string(&response)?);
            }
            return Ok(());
        }
    };

    logger.log_json(&serde_json::json!({
        "step": "canonicalization_complete",
        "canonical_bytes": canonical_bytes.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Verify signature
    let is_valid = verifying_key.verify(&canonical_bytes, &signature).is_ok();

    logger.log_json(&serde_json::json!({
        "step": "verification_complete",
        "signature_valid": is_valid,
        "schema_valid": schema_valid,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Output result
    let response = create_verify_response(
        &cert_file_path,
        &pubkey_path,
        Some(is_valid),
        Some(schema_valid),
        None,
    );
    if let Some(errors) = schema_errors {
        let mut response_obj = response.as_object().unwrap().clone();
        response_obj.insert("schema_errors".to_string(), serde_json::json!(errors));
        println!(
            "{}",
            serde_json::to_string(&serde_json::Value::Object(response_obj))?
        );
    } else {
        println!("{}", serde_json::to_string(&response)?);
    }

    Ok(())
}
