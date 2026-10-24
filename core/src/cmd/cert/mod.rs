//! `securewipe cert`: show, export, sign, verify and validate certificates.

mod sign;
mod validate;
mod verify;

use sign::handle_cert_sign;
use validate::handle_cert_validate;
use verify::handle_cert_verify;

use anyhow::Result;
use clap::Args;
use securewipe::logging::Logger;
use serde_json::json;

#[derive(Args)]
pub struct CertArgs {
    /// Show certificate by ID
    #[arg(long)]
    pub show: Option<String>,

    /// Export certificate as PDF
    #[arg(long)]
    pub export_pdf: Option<String>,

    #[command(subcommand)]
    pub command: Option<CertCommands>,
}

#[derive(clap::Subcommand)]
pub enum CertCommands {
    /// Sign a certificate file
    Sign {
        /// Path to certificate JSON file to sign
        #[arg(long)]
        file: std::path::PathBuf,

        /// Path to Ed25519 private key for signing
        #[arg(long)]
        key: Option<std::path::PathBuf>,

        /// Force overwrite existing signature
        #[arg(long)]
        force: bool,
    },
    /// Verify a signed certificate file
    Verify {
        /// Path to certificate JSON file to verify
        #[arg(long)]
        file: std::path::PathBuf,

        /// Path to Ed25519 public key PEM file
        #[arg(long)]
        pubkey: std::path::PathBuf,
    },
    /// Validate certificate schema without signing or verification
    Validate {
        /// Path to certificate JSON file to validate
        #[arg(long)]
        file: std::path::PathBuf,
    },
}

pub fn handle_cert(args: CertArgs, logger: &Logger) -> Result<()> {
    use securewipe::cert_pdf::CertificatePdfGenerator;
    use std::fs;

    logger.log_info("Processing certificate command");

    if let Some(cert_id) = args.show {
        // Show certificate details
        logger.log_info(&format!("Showing certificate: {}", cert_id));

        let response = json!({
            "cmd": "cert",
            "action": "show",
            "cert_id": cert_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "status": "stub - not implemented"
        });

        logger.log_json(&response);
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }

    if let Some(cert_id) = args.export_pdf {
        logger.log_info(&format!("Exporting certificate to PDF: {}", cert_id));

        // Try to find the certificate JSON file
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        let cert_dir = home_dir.join("SecureWipe").join("certificates");
        let cert_json_path = cert_dir.join(format!("{}.json", cert_id));

        if !cert_json_path.exists() {
            let response = json!({
                "cmd": "cert",
                "action": "export_pdf",
                "cert_id": cert_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "status": "error",
                "error": format!("Certificate file not found: {}", cert_json_path.display())
            });

            logger.log_json(&response);
            println!("{}", serde_json::to_string_pretty(&response)?);
            return Err(anyhow::anyhow!(
                "Certificate file not found: {}",
                cert_json_path.display()
            ));
        }

        // Read and parse the certificate JSON
        let cert_json = fs::read_to_string(&cert_json_path)?;
        let cert_value: serde_json::Value = serde_json::from_str(&cert_json)?;

        let cert_type = cert_value
            .get("cert_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid certificate: missing cert_type"))?;

        // Generate PDF based on certificate type
        let pdf_generator =
            CertificatePdfGenerator::new(Some("https://verify.securewipe.local".to_string()));
        let pdf_path = match cert_type {
            "backup" => {
                // Use generic JSON approach to avoid struct definition conflicts
                pdf_generator.generate_backup_pdf_from_json(&cert_json)?
            }
            "wipe" => {
                // Use generic JSON approach to avoid struct definition conflicts
                pdf_generator.generate_wipe_pdf_from_json(&cert_json)?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported certificate type: {}",
                    cert_type
                ));
            }
        };

        let response = json!({
            "cmd": "cert",
            "action": "export_pdf",
            "cert_id": cert_id,
            "cert_type": cert_type,
            "pdf_path": pdf_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "status": "success"
        });

        logger.log_json(&response);
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }

    if let Some(command) = args.command {
        match command {
            CertCommands::Sign { file, key, force } => {
                return handle_cert_sign(file, key, force, logger);
            }
            CertCommands::Verify { file, pubkey } => {
                return handle_cert_verify(file, pubkey, logger);
            }
            CertCommands::Validate { file } => {
                return handle_cert_validate(file, logger);
            }
        }
    }

    // No specific action requested
    let response = json!({
        "cmd": "cert",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "error",
        "error": "No action specified. Use --show <cert_id>, --export-pdf <cert_id>, sign --file <file.json>, or verify --file <file.json> --pubkey <pubkey.pem>"
    });

    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Err(anyhow::anyhow!("No action specified"))
}

/// Parse Ed25519 public key from PEM format
fn parse_ed25519_public_key_pem(pem_content: &str) -> Result<[u8; 32]> {
    // Check for proper Ed25519 PUBLIC KEY PEM headers
    if !pem_content.contains("-----BEGIN PUBLIC KEY-----") {
        return Err(anyhow::anyhow!("Invalid PEM format. Expected '-----BEGIN PUBLIC KEY-----' for Ed25519 public key. Provide an Ed25519 PUBLIC KEY PEM."));
    }

    if !pem_content.contains("-----END PUBLIC KEY-----") {
        return Err(anyhow::anyhow!("Invalid PEM format. Missing '-----END PUBLIC KEY-----' footer. Provide an Ed25519 PUBLIC KEY PEM."));
    }

    // Extract base64 content between headers
    let lines: Vec<&str> = pem_content.lines().collect();
    let start_idx = lines
        .iter()
        .position(|&line| line.contains("BEGIN PUBLIC KEY"))
        .ok_or_else(|| anyhow::anyhow!("No PEM begin marker found"))?;
    let end_idx = lines
        .iter()
        .position(|&line| line.contains("END PUBLIC KEY"))
        .ok_or_else(|| anyhow::anyhow!("No PEM end marker found"))?;

    if start_idx >= end_idx {
        return Err(anyhow::anyhow!("Invalid PEM structure"));
    }

    let base64_lines = &lines[start_idx + 1..end_idx];
    let base64_content = base64_lines.join("");

    use base64::{engine::general_purpose::STANDARD, Engine};
    let der_bytes = STANDARD
        .decode(&base64_content)
        .map_err(|e| anyhow::anyhow!("Failed to decode PEM base64: {}", e))?;

    // For Ed25519 public keys, the DER format has the key at the end
    if der_bytes.len() < 32 {
        return Err(anyhow::anyhow!("Invalid Ed25519 public key DER: too short ({} bytes). Provide an Ed25519 PUBLIC KEY PEM.", der_bytes.len()));
    }

    let key_start = der_bytes.len() - 32;
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&der_bytes[key_start..]);

    Ok(key_bytes)
}

/// Helper function to create consistent verify response JSON
fn create_verify_response(
    cert_file_path: &std::path::Path,
    pubkey_path: &std::path::Path,
    signature_valid: Option<bool>,
    schema_valid: Option<bool>,
    error: Option<String>,
) -> serde_json::Value {
    let mut response = serde_json::json!({
        "op": "cert_verify",
        "file": cert_file_path.display().to_string(),
        "signature_valid": signature_valid,
        "schema_valid": schema_valid,
        "pubkey": pubkey_path.display().to_string()
    });

    if let Some(err) = error {
        response
            .as_object_mut()
            .unwrap()
            .insert("error".to_string(), serde_json::json!(err));
    }

    response
}
