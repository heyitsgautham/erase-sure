//! `securewipe backup`: encrypted backup with a signed certificate.

use anyhow::Result;
use clap::Args;
use securewipe::logging::Logger;

#[derive(Args)]
pub struct BackupArgs {
    /// Source device to backup from
    #[arg(long)]
    pub device: String,

    /// Destination path for backup
    #[arg(long)]
    pub dest: String,

    /// Specific paths to backup (defaults to common user directories)
    #[arg(long)]
    pub paths: Vec<String>,

    /// Sign the generated certificate
    #[arg(long)]
    pub sign: bool,

    /// Path to Ed25519 private key for signing
    #[arg(long)]
    pub sign_key_path: Option<std::path::PathBuf>,

    /// Allow overwriting existing signature
    #[arg(long)]
    pub force: bool,
}

pub fn handle_backup(args: BackupArgs, logger: &Logger) -> Result<()> {
    use securewipe::backup::{BackupOperations, EncryptedBackup};

    logger.log_info("Starting backup operation");

    let backup_engine = EncryptedBackup::new();
    let paths = &args.paths;

    match backup_engine.perform_backup(&args.device, paths, &args.dest) {
        Ok(result) => {
            logger.log_info("Backup completed successfully");

            println!("Backup completed successfully!");
            println!("Backup ID: {}", result.backup_id);
            println!("Encryption: {}", result.encryption_method);
            println!("Files processed: {}", result.manifest.total_files);
            println!("Total bytes: {}", result.manifest.total_bytes);
            println!(
                "Verification samples: {}/{}",
                if result.verification_passed {
                    result.verification_samples
                } else {
                    0
                },
                result.verification_samples
            );
            println!(
                "Verification status: {}",
                if result.verification_passed {
                    "PASSED"
                } else {
                    "FAILED"
                }
            );

            if !result.verification_passed {
                logger.log_error("Backup verification failed");
                eprintln!("WARNING: Backup verification failed! Some files may be corrupted.");
                return Err(anyhow::anyhow!("Backup verification failed"));
            }

            // Load the certificate already created by the backup process
            use std::fs;

            logger.log_info("Loading backup certificate for signing");
            let home_dir = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
            let cert_dir = home_dir.join("SecureWipe").join("certificates");
            let cert_file = cert_dir.join(format!("{}.json", result.backup_id));

            let cert_json = fs::read_to_string(&cert_file)
                .map_err(|e| anyhow::anyhow!("Failed to read certificate file: {}", e))?;
            let mut cert_value: serde_json::Value = serde_json::from_str(&cert_json)
                .map_err(|e| anyhow::anyhow!("Failed to parse certificate JSON: {}", e))?;

            // Validate schema before proceeding
            use securewipe::schema::CertificateValidator;
            logger.log_info("Validating backup certificate schema");
            let validator = CertificateValidator::default();
            let validation_result = validator
                .validate_certificate(&cert_value)
                .map_err(|e| anyhow::anyhow!("Schema validation error: {}", e))?;

            if !validation_result.valid {
                logger.log_error("Backup certificate failed schema validation");
                eprintln!("WARNING: Generated certificate failed schema validation:");
                for error in &validation_result.errors {
                    eprintln!("  - {}", error);
                }
                // Continue anyway for backup operation, but log the issue
            } else {
                logger.log_info("Backup certificate passed schema validation");
            }

            // Certificate file already exists from backup process
            // Get the cert_id for logging
            let cert_id = cert_value
                .get("cert_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Sign certificate if requested or if not already signed
            let should_sign =
                args.sign || cert_value.get("signature").is_none_or(|sig| sig.is_null());
            if should_sign {
                use securewipe::signer::{load_private_key, sign_certificate};

                logger.log_info("Signing backup certificate");
                logger.log_json(&serde_json::json!({
                    "step": "certificate_signing",
                    "cert_id": cert_id,
                    "key_source": if args.sign_key_path.is_some() { "flag" } else { "env" },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));

                let signing_key = load_private_key(args.sign_key_path.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to load signing key: {}", e))?;

                sign_certificate(&mut cert_value, &signing_key, args.force)
                    .map_err(|e| anyhow::anyhow!("Failed to sign certificate: {}", e))?;

                logger.log_json(&serde_json::json!({
                    "step": "certificate_signed",
                    "cert_id": cert_id,
                    "signed": true,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));
            } else {
                logger.log_info("Certificate already signed, skipping signing step");
            }

            // Write certificate file atomically
            let cert_json = serde_json::to_string_pretty(&cert_value)?;
            let temp_file = cert_file.with_extension("tmp");
            fs::write(&temp_file, &cert_json)?;
            fs::rename(&temp_file, &cert_file)?;

            logger.log_json(&serde_json::json!({
                "step": "certificate_updated",
                "cert_id": cert_id,
                "cert_path": cert_file.display().to_string(),
                "signed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            println!(
                "Backup certificate signed and updated: {}",
                cert_file.display()
            );

            Ok(())
        }
        Err(e) => {
            logger.log_error(&format!("Backup failed: {}", e));
            Err(anyhow::anyhow!("Backup failed: {}", e))
        }
    }
}
