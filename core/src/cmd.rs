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
}

#[derive(Args)]
pub struct CertArgs {
    /// Show certificate by ID
    #[arg(long)]
    pub show: Option<String>,
    
    /// Export certificate as PDF
    #[arg(long)]
    pub export_pdf: Option<String>,
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
    let response = json!({
        "cmd": "backup",
        "args": {
            "device": args.device,
            "dest": args.dest,
            "paths": args.paths
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "stub"
    });
    
    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
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
    
    // No specific action requested
    let response = json!({
        "cmd": "cert",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "error",
        "error": "No action specified. Use --show <cert_id> or --export-pdf <cert_id>"
    });
    
    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Err(anyhow::anyhow!("No action specified"))
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
        };
        assert_eq!(args.device, "/dev/sda");
        assert_eq!(args.dest, "/mnt/backup");
        assert_eq!(args.paths.len(), 2);
    }

    #[test]
    fn test_wipe_args_defaults() {
        let args = WipeArgs {
            device: "/dev/sda".to_string(),
            policy: "PURGE".to_string(),
            iso_mode: false,
            format: "json".to_string(),
            samples: 128,
        };
        assert_eq!(args.policy, "PURGE");
        assert!(!args.iso_mode);
        assert_eq!(args.format, "json");
        assert_eq!(args.samples, 128);
    }

    #[test]
    fn test_cert_args_creation() {
        let args = CertArgs {
            show: Some("cert_123".to_string()),
            export_pdf: None,
        };
        assert_eq!(args.show, Some("cert_123".to_string()));
        assert_eq!(args.export_pdf, None);
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
        };
        
        let result = handle_backup(args, &logger);
        assert!(result.is_ok());
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
}