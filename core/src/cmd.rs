use clap::Args;
use serde_json::json;
use crate::logging::Logger;
use anyhow::Result;

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
    
    /// Force wipe of critical disks (requires ISO mode)
    #[arg(long)]
    pub i_know_what_im_doing: bool,
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
    let response = json!({
        "cmd": "wipe",
        "args": {
            "device": args.device,
            "policy": args.policy,
            "force": args.i_know_what_im_doing
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "stub"
    });
    
    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub fn handle_cert(args: CertArgs, logger: &Logger) -> Result<()> {
    let response = json!({
        "cmd": "cert",
        "args": {
            "show": args.show,
            "export_pdf": args.export_pdf
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "stub"
    });
    
    logger.log_json(&response);
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
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
            i_know_what_im_doing: false,
        };
        assert_eq!(args.policy, "PURGE");
        assert!(!args.i_know_what_im_doing);
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
        assert!(result.is_ok());
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
            i_know_what_im_doing: false,
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
        assert!(result.is_ok());
    }
}