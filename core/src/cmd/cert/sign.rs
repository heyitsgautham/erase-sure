use anyhow::Result;
use securewipe::logging::Logger;
use serde_json::json;

pub(super) fn handle_cert_sign(
    cert_file_path: std::path::PathBuf,
    sign_key_path: Option<std::path::PathBuf>,
    force: bool,
    logger: &Logger,
) -> Result<()> {
    use securewipe::signer::{load_private_key, sign_certificate};
    use std::fs;

    logger.log_info(&format!(
        "Signing certificate file: {}",
        cert_file_path.display()
    ));

    if !cert_file_path.exists() {
        let response = json!({
            "op": "cert_sign",
            "file": cert_file_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "signed": false,
            "schema_valid": null,
            "error": format!("Certificate file not found: {}", cert_file_path.display())
        });

        logger.log_json(&response);
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Err(anyhow::anyhow!(
            "Certificate file not found: {}",
            cert_file_path.display()
        ));
    }

    let key_source = if sign_key_path.is_some() {
        "flag"
    } else {
        "env"
    };

    // Read certificate file
    let cert_json = fs::read_to_string(&cert_file_path)?;
    let mut cert_value: serde_json::Value = serde_json::from_str(&cert_json)?;

    // For signing, we validate the unsigned certificate (without signature requirement)
    // The full schema requires a signature, but for signing we validate the rest first
    logger.log_info("Validating unsigned certificate structure before signing");

    // We'll do a basic validation to ensure required fields are present except signature
    // This is more lenient than full schema validation which requires signature
    let required_fields = ["cert_type", "cert_id", "certificate_version", "created_at"];
    for field in &required_fields {
        if cert_value.get(field).is_none() {
            let response = json!({
                "op": "cert_sign",
                "file": cert_file_path.display().to_string(),
                "key_source": key_source,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "signed": false,
                "schema_valid": false,
                "error": format!("Missing required field: {}", field)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!("Missing required field: {}", field));
        }
    }

    logger.log_info("Certificate structure validation passed");

    // Load private key
    let signing_key = match load_private_key(sign_key_path) {
        Ok(key) => {
            logger.log_info("Private key loaded successfully");
            key
        }
        Err(e) => {
            let response = json!({
                "op": "cert_sign",
                "file": cert_file_path.display().to_string(),
                "key_source": key_source,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "signed": false,
                "error": format!("Failed to load private key: {}", e)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!("Failed to load private key: {}", e));
        }
    };

    // Sign the certificate
    match sign_certificate(&mut cert_value, &signing_key, force) {
        Ok(()) => {
            logger.log_info("Certificate signed successfully");

            // Write back to file atomically
            let temp_file = cert_file_path.with_extension("tmp");
            let signed_json = serde_json::to_string_pretty(&cert_value)?;
            fs::write(&temp_file, &signed_json)?;
            fs::rename(&temp_file, &cert_file_path)?;

            let response = json!({
                "op": "cert_sign",
                "file": cert_file_path.display().to_string(),
                "key_source": key_source,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "signed": true
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Err(e) => {
            let response = json!({
                "op": "cert_sign",
                "file": cert_file_path.display().to_string(),
                "key_source": key_source,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "signed": false,
                "error": format!("Signing failed: {}", e)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            Err(anyhow::anyhow!("Signing failed: {}", e))
        }
    }
}
