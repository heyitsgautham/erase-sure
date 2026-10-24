//! `securewipe discover`: list drives with their risk level.

use anyhow::Result;
use clap::Args;
use securewipe::logging::Logger;

#[derive(Args)]
pub struct DiscoverArgs {
    /// Output format (json or human)
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Disable device enrichment (for testing)
    #[arg(long)]
    pub no_enrich: bool,
}

pub fn handle_discover(args: DiscoverArgs, logger: &Logger) -> Result<()> {
    use securewipe::device::{DeviceDiscovery, LinuxDeviceDiscovery};

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
