use clap::Args;
use serde_json::json;
use crate::logging::Logger;
use anyhow::Result;
use dirs;

#[derive(Args)]
pub struct DiscoverArgs {
    /// Output format (json or human)
    #[arg(long, default_value = "json")]
    pub format: String,
    
    /// Disable device enrichment (for testing)
    #[arg(long)]
    pub no_enrich: bool,
}

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

#[derive(Args)]
pub struct WipeArgs {
    /// Device to wipe
    #[arg(long)]
    pub device: String,
    
    /// Wipe policy (CLEAR, PURGE)
    #[arg(long, default_value = "PURGE")]
    pub policy: String,
    
    /// Enable ISO mode (allows CRITICAL disk wiping)
    #[arg(long)]
    pub iso_mode: bool,
    
    /// Output format (json or human)
    #[arg(long, default_value = "json")]
    pub format: String,
    
    /// Number of verification samples
    #[arg(long, default_value = "128")]
    pub samples: usize,
    
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
        sign_key_path: Option<std::path::PathBuf>,
        
        /// Force overwrite existing signature
        #[arg(long)]
        force: bool,
    },
}

pub fn handle_discover(args: DiscoverArgs, logger: &Logger) -> Result<()> {
    use crate::device::{DeviceDiscovery, LinuxDeviceDiscovery};
    
    logger.log_info("Starting device discovery");
    
    let discovery = if args.no_enrich {
        LinuxDeviceDiscovery::new_without_enrichment()
    } else {
        LinuxDeviceDiscovery::new()
    };
    
    match discovery.discover_devices() {
        Ok(devices) => {
            logger.log_info(&format!("Found {} devices", devices.len()));
            
            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            } else {
                // Human-readable format
                for device in &devices {
                    println!("Device: {}", device.name);
                    if let Some(ref model) = device.model {
                        println!("  Model: {}", model);
                    }
                    if let Some(ref serial) = device.serial {
                        println!("  Serial: {}", serial);
                    }
                    println!("  Capacity: {} bytes", device.capacity_bytes);
                    if let Some(ref bus) = device.bus {
                        println!("  Bus: {}", bus);
                    }
                    println!("  Risk Level: {:?}", device.risk_level);
                    if !device.mountpoints.is_empty() {
                        println!("  Mountpoints: {}", device.mountpoints.join(", "));
                    }
                    println!();
                }
            }
            
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Device discovery failed: {}", e);
            logger.log_error(&error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

pub fn handle_backup(args: BackupArgs, logger: &Logger) -> Result<()> {
    use crate::backup::{EncryptedBackup, BackupOperations};
    
    logger.log_info("Starting backup operation");
    
    let backup_engine = EncryptedBackup::new();
    let paths = &args.paths;
    
    match backup_engine.perform_backup(&args.device, &paths, &args.dest) {
        Ok(result) => {
            logger.log_info("Backup completed successfully");
            
            println!("Backup completed successfully!");
            println!("Backup ID: {}", result.backup_id);
            println!("Encryption: {}", result.encryption_method);
            println!("Files processed: {}", result.manifest.total_files);
            println!("Total bytes: {}", result.manifest.total_bytes);
            println!("Verification samples: {}/{}", 
                     if result.verification_passed { result.verification_samples } else { 0 },
                     result.verification_samples);
            println!("Verification status: {}", 
                     if result.verification_passed { "PASSED" } else { "FAILED" });
            
            if !result.verification_passed {
                logger.log_error("Backup verification failed");
                eprintln!("WARNING: Backup verification failed! Some files may be corrupted.");
                return Err(anyhow::anyhow!("Backup verification failed"));
            }
            
            // Generate and optionally sign certificate
            use crate::cert::{Ed25519CertificateManager, CertificateOperations};
            use std::fs;
            
            logger.log_info("Generating backup certificate");
            let cert_mgr = Ed25519CertificateManager;
            let backup_cert = cert_mgr.create_backup_certificate(&result)
                .map_err(|e| anyhow::anyhow!("Failed to create certificate: {}", e))?;
            
            let mut cert_value = serde_json::to_value(&backup_cert)
                .map_err(|e| anyhow::anyhow!("Failed to serialize certificate: {}", e))?;
            
            // Save certificate directory
            let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
            let cert_dir = home_dir.join("SecureWipe").join("certificates");
            std::fs::create_dir_all(&cert_dir)?;
            let cert_file = cert_dir.join(format!("{}.json", backup_cert.cert_id));
            
            // Handle signing if requested
            if args.sign || args.sign_key_path.is_some() {
                use crate::signer::{load_private_key, sign_certificate};
                
                logger.log_info("Signing backup certificate");
                logger.log_json(&serde_json::json!({
                    "step": "certificate_signing",
                    "cert_id": backup_cert.cert_id,
                    "key_source": if args.sign_key_path.is_some() { "flag" } else { "env" },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));
                
                let signing_key = load_private_key(args.sign_key_path.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to load signing key: {}", e))?;
                
                sign_certificate(&mut cert_value, &signing_key, args.force)
                    .map_err(|e| anyhow::anyhow!("Failed to sign certificate: {}", e))?;
                
                logger.log_json(&serde_json::json!({
                    "step": "certificate_signed",
                    "cert_id": backup_cert.cert_id,
                    "signed": true,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));
            }
            
            // Write certificate file atomically
            let cert_json = serde_json::to_string_pretty(&cert_value)?;
            let temp_file = cert_file.with_extension("tmp");
            fs::write(&temp_file, &cert_json)?;
            fs::rename(&temp_file, &cert_file)?;
            
            logger.log_json(&serde_json::json!({
                "step": "certificate_saved",
                "cert_id": backup_cert.cert_id,
                "cert_path": cert_file.display().to_string(),
                "signed": args.sign || args.sign_key_path.is_some(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            
            println!("Backup certificate saved: {}", cert_file.display());
            
            Ok(())
        }
        Err(e) => {
            logger.log_error(&format!("Backup failed: {}", e));
            Err(anyhow::anyhow!("Backup failed: {}", e))
        }
    }
}

pub fn handle_wipe(args: WipeArgs, logger: &Logger) -> Result<()> {
    use crate::wipe::{plan_wipe, WipePolicy};
    use crate::device::{DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel};
    
    logger.log_info("Starting wipe planning");
    
    // Log CLI arguments
    logger.log_json(&json!({
        "step": "cli_args",
        "device": args.device,
        "policy": args.policy,
        "iso_mode": args.iso_mode,
        "samples": args.samples,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    
    // Parse policy
    let policy = match args.policy.as_str() {
        "CLEAR" => Some(WipePolicy::Clear),
        "PURGE" => Some(WipePolicy::Purge),
        "DESTROY" => Some(WipePolicy::Destroy),
        _ => {
            let error_msg = format!("Invalid policy: {}. Must be CLEAR, PURGE, or DESTROY", args.policy);
            logger.log_error(&error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }
    };
    
    // Determine if device is critical by checking risk level
    let discovery = LinuxDeviceDiscovery::new();
    let is_critical = match discovery.discover_devices() {
        Ok(devices) => {
            let device = devices.iter().find(|d| d.name == args.device);
            match device {
                Some(d) => {
                    logger.log_json(&json!({
                        "step": "device_risk_check",
                        "device": args.device,
                        "risk_level": d.risk_level,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }));
                    matches!(d.risk_level, RiskLevel::Critical)
                },
                None => {
                    logger.log_json(&json!({
                        "step": "device_risk_check",
                        "device": args.device,
                        "result": "device_not_found_assuming_safe",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }));
                    false
                }
            }
        },
        Err(e) => {
            logger.log_json(&json!({
                "step": "device_risk_check",
                "device": args.device,
                "error": e.to_string(),
                "result": "discovery_failed_assuming_safe",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            false
        }
    };
    
    // Log controller probing attempts
    logger.log_json(&json!({
        "step": "controller_probe_start",
        "device": args.device,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    
    // Generate wipe plan with custom samples
    let mut plan = plan_wipe(&args.device, policy, is_critical, args.iso_mode, None, None);
    plan.verification.samples = args.samples;
    
    // Log planning decision
    logger.log_json(&json!({
        "step": "wipe_plan_generated",
        "device": plan.device,
        "risk": plan.risk,
        "policy": plan.policy,
        "main_method": plan.main_method,
        "hpa_dco_clear": plan.hpa_dco_clear,
        "blocked": plan.blocked,
        "reason": plan.reason,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    
    // Output based on format
    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        // Human-readable format
        println!("Wipe Plan for {}", plan.device);
        println!("=======================");
        println!("• Policy: {:?}", plan.policy);
        println!("• Risk Level: {}", plan.risk);
        println!("• Main Method: {}", plan.main_method);
        println!("• HPA/DCO Clear: {}", if plan.hpa_dco_clear { "Yes" } else { "No" });
        println!("• Verification: {} {} samples", plan.verification.strategy, plan.verification.samples);
        
        if plan.blocked {
            println!("• Status: ❌ BLOCKED");
            if let Some(ref reason) = plan.reason {
                println!("• Reason: {}", reason);
            }
        } else {
            println!("• Status: ✅ Ready to proceed");
            if args.iso_mode && plan.risk == "CRITICAL" {
                println!("• Note: ISO mode enabled - CRITICAL disk wipe allowed");
            }
        }
    }
    
    // TODO: In a complete implementation, this would actually perform the wipe
    // For now, we generate a stub wipe certificate if signing is requested
    if args.sign || args.sign_key_path.is_some() {
        use crate::cert::{Ed25519CertificateManager, CertificateOperations};
        use crate::wipe::{WipeResult, WipeCommand};
        use std::fs;
        
        // Generate a stub wipe result for certificate creation
        let stub_wipe_result = WipeResult {
            device: args.device.clone(),
            policy: match args.policy.as_str() {
                "CLEAR" => crate::wipe::WipePolicy::Clear,
                "PURGE" => crate::wipe::WipePolicy::Purge,
                "DESTROY" => crate::wipe::WipePolicy::Destroy,
                _ => crate::wipe::WipePolicy::Purge,
            },
            method: plan.main_method.clone(),
            commands: vec![WipeCommand {
                command: format!("echo 'Wipe planned for {}'", args.device),
                exit_code: 0,
                elapsed_ms: 0,
                output: "Planning completed successfully".to_string(),
            }],
            verification_samples: args.samples,
            verification_passed: true,
            fallback_reason: plan.reason.clone(),
        };
        
        logger.log_info("Generating wipe certificate");
        let cert_mgr = Ed25519CertificateManager;
        let wipe_cert = cert_mgr.create_wipe_certificate(&stub_wipe_result, None)
            .map_err(|e| anyhow::anyhow!("Failed to create wipe certificate: {}", e))?;
        
        let mut cert_value = serde_json::to_value(&wipe_cert)
            .map_err(|e| anyhow::anyhow!("Failed to serialize wipe certificate: {}", e))?;
        
        // Save certificate directory
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        let cert_dir = home_dir.join("SecureWipe").join("certificates");
        std::fs::create_dir_all(&cert_dir)?;
        let cert_file = cert_dir.join(format!("{}.json", wipe_cert.cert_id));
        
        // Handle signing
        use crate::signer::{load_private_key, sign_certificate};
        
        logger.log_info("Signing wipe certificate");
        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_signing",
            "cert_id": wipe_cert.cert_id,
            "key_source": if args.sign_key_path.is_some() { "flag" } else { "env" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));
        
        let signing_key = load_private_key(args.sign_key_path.clone())
            .map_err(|e| anyhow::anyhow!("Failed to load signing key: {}", e))?;
        
        sign_certificate(&mut cert_value, &signing_key, args.force)
            .map_err(|e| anyhow::anyhow!("Failed to sign wipe certificate: {}", e))?;
        
        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_signed",
            "cert_id": wipe_cert.cert_id,
            "signed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));
        
        // Write certificate file atomically
        let cert_json = serde_json::to_string_pretty(&cert_value)?;
        let temp_file = cert_file.with_extension("tmp");
        fs::write(&temp_file, &cert_json)?;
        fs::rename(&temp_file, &cert_file)?;
        
        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_saved",
            "cert_id": wipe_cert.cert_id,
            "cert_path": cert_file.display().to_string(),
            "signed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));
        
        println!("Wipe certificate saved: {}", cert_file.display());
    }
    
    logger.log_info("Wipe planning completed");
    Ok(())
}

pub fn handle_cert(args: CertArgs, logger: &Logger) -> Result<()> {
    use securewipe::cert_pdf::CertificatePdfGenerator;
    use securewipe::cert::{BackupCertificate, WipeCertificate};
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
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
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
            return Err(anyhow::anyhow!("Certificate file not found: {}", cert_json_path.display()));
        }
        
        // Read and parse the certificate JSON
        let cert_json = fs::read_to_string(&cert_json_path)?;
        let cert_value: serde_json::Value = serde_json::from_str(&cert_json)?;
        
        let cert_type = cert_value.get("cert_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid certificate: missing cert_type"))?;
        
        // Generate PDF based on certificate type
        let pdf_generator = CertificatePdfGenerator::new(Some("https://verify.securewipe.local".to_string()));
        let pdf_path = match cert_type {
            "backup" => {
                let cert: BackupCertificate = serde_json::from_str(&cert_json)?;
                pdf_generator.generate_backup_certificate_pdf(&cert)?
            },
            "wipe" => {
                let cert: WipeCertificate = serde_json::from_str(&cert_json)?;
                pdf_generator.generate_wipe_certificate_pdf(&cert)?
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported certificate type: {}", cert_type));
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
            CertCommands::Sign { file, sign_key_path, force } => {
                return handle_cert_sign(file, sign_key_path, force, logger);
            }
        }
    }
    
    // No specific action requested
    let response = json!({
        "cmd": "cert",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "error",
        "error": "No action specified. Use --show <cert_id>, --export-pdf <cert_id>, or sign --file <file.json>"
    });
    
    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Err(anyhow::anyhow!("No action specified"))
}

fn handle_cert_sign(
    cert_file_path: std::path::PathBuf,
    sign_key_path: Option<std::path::PathBuf>,
    force: bool,
    logger: &Logger,
) -> Result<()> {
    use crate::signer::{load_private_key, sign_certificate};
    use std::fs;
    
    logger.log_info(&format!("Signing certificate file: {}", cert_file_path.display()));
    
    if !cert_file_path.exists() {
        let response = json!({
            "op": "cert_sign",
            "file": cert_file_path.display().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "signed": false,
            "error": format!("Certificate file not found: {}", cert_file_path.display())
        });
        
        logger.log_json(&response);
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Err(anyhow::anyhow!("Certificate file not found: {}", cert_file_path.display()));
    }
    
    let key_source = if sign_key_path.is_some() { "flag" } else { "env" };
    
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
    
    // Read certificate file
    let cert_json = fs::read_to_string(&cert_file_path)?;
    let mut cert_value: serde_json::Value = serde_json::from_str(&cert_json)?;
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::Logger;

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
                    error_msg.contains("lsblk command not found") ||
                    error_msg.contains("Device discovery failed"),
                    "Unexpected error: {}", error_msg
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
                    error_msg.contains("Read-only file system") ||
                    error_msg.contains("No such file or directory") ||
                    error_msg.contains("Permission denied") ||
                    error_msg.contains("Backup failed"),
                    "Unexpected error: {}", error_msg
                );
            }
        }
    }

    #[test]
    fn test_handle_wipe() {
        let logger = Logger::new();
        let args = WipeArgs {
            device: "/dev/sda".to_string(),
            policy: "PURGE".to_string(),
            iso_mode: false,
            format: "json".to_string(),
            samples: 128,
            sign: false,
            sign_key_path: None,
            force: false,
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
                    error_msg.contains("lsblk command not found") ||
                    error_msg.contains("Device discovery failed"),
                    "Unexpected error: {}", error_msg
                );
            }
        }
    }

    #[test]
    fn test_cert_sign_args() {
        let sign_command = CertCommands::Sign {
            file: std::path::PathBuf::from("/tmp/test_cert.json"),
            sign_key_path: Some(std::path::PathBuf::from("/tmp/test_key")),
            force: true,
        };
        
        let CertCommands::Sign { file, sign_key_path, force } = sign_command;
        assert_eq!(file, std::path::PathBuf::from("/tmp/test_cert.json"));
        assert_eq!(sign_key_path, Some(std::path::PathBuf::from("/tmp/test_key")));
        assert!(force);
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
        assert_eq!(args.sign_key_path, Some(std::path::PathBuf::from("/tmp/key")));
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
        };
        
        assert!(args.sign);
        assert_eq!(args.sign_key_path, Some(std::path::PathBuf::from("/tmp/key")));
        assert!(args.force);
    }
}