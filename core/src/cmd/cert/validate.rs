use anyhow::Result;
use securewipe::logging::Logger;
use serde_json::json;

pub(super) fn handle_cert_validate(
    cert_file_path: std::path::PathBuf,
    logger: &Logger,
) -> Result<()> {
    use securewipe::schema::CertificateValidator;
    use std::fs;

    logger.log_info(&format!(
        "Validating certificate schema: {}",
        cert_file_path.display()
    ));

    if !cert_file_path.exists() {
        let response = json!({
            "op": "cert_validate",
            "file": cert_file_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
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

    // Read certificate file
    let cert_json = match fs::read_to_string(&cert_file_path) {
        Ok(json) => json,
        Err(e) => {
            let response = json!({
                "op": "cert_validate",
                "file": cert_file_path.display().to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "schema_valid": null,
                "error": format!("Failed to read certificate file: {}", e)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!("Failed to read certificate file: {}", e));
        }
    };

    let cert_value: serde_json::Value = match serde_json::from_str(&cert_json) {
        Ok(value) => value,
        Err(e) => {
            let response = json!({
                "op": "cert_validate",
                "file": cert_file_path.display().to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "schema_valid": false,
                "error": format!("Invalid JSON in certificate file: {}", e)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!("Invalid JSON in certificate file: {}", e));
        }
    };

    // Validate schema
    let validator = CertificateValidator::default();
    let validation_result = match validator.validate_certificate(&cert_value) {
        Ok(result) => result,
        Err(e) => {
            let response = json!({
                "op": "cert_validate",
                "file": cert_file_path.display().to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "schema_valid": null,
                "error": format!("Schema validation error: {}", e)
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!("Schema validation error: {}", e));
        }
    };

    // Create response
    let mut response = json!({
        "op": "cert_validate",
        "file": cert_file_path.display().to_string(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "schema_valid": validation_result.valid,
        "schema_type": validation_result.schema_id
    });

    if !validation_result.valid {
        response.as_object_mut().unwrap().insert(
            "schema_errors".to_string(),
            serde_json::json!(validation_result.errors),
        );
    }

    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);

    if validation_result.valid {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Certificate failed schema validation"))
    }
}
