use crate::backup::{EncryptedBackup, BackupOperations};
use crate::wipe::{WipePolicy, NistAlignedWipe, WipeOperations, plan_wipe};
use crate::cert::{Ed25519CertificateManager, CertificateOperations};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "securewipe")]
#[command(about = "SecureWipe CLI - NIST-aligned data backup and wiping", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Perform encrypted backup of specified paths
    Backup {
        /// Device to backup from
        #[arg(long)]
        device: String,

        /// Destination path for backup
        #[arg(long)]
        dest: String,

        /// Paths to backup (defaults to ~/Documents, ~/Desktop, ~/Pictures)
        #[arg(long, value_delimiter = ',')]
        paths: Option<Vec<String>>,
    },
    /// Perform NIST-aligned secure wipe of a device
    Wipe {
        /// Device to wipe (e.g., /dev/sdb)
        #[arg(long)]
        device: String,

        /// Wipe policy to use
        #[arg(long, value_enum, default_value_t = WipePolicyArg::Purge)]
        policy: WipePolicyArg,

        /// Sign the wipe certificate after completion
        #[arg(long)]
        sign: bool,

        /// Required safety flag to enable destructive wiping
        #[arg(long)]
        danger_allow_wipe: bool,

        /// Link to existing backup certificate ID
        #[arg(long)]
        backup_cert_id: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum WipePolicyArg {
    /// Single overwrite with zeros, minimal verification
    Clear,
    /// Random overwrite + comprehensive verification (default)
    Purge,
    /// Multi-pass overwrite + HPA/DCO clearing + extensive verification
    Destroy,
}

impl From<WipePolicyArg> for WipePolicy {
    fn from(policy: WipePolicyArg) -> Self {
        match policy {
            WipePolicyArg::Clear => WipePolicy::Clear,
            WipePolicyArg::Purge => WipePolicy::Purge,
            WipePolicyArg::Destroy => WipePolicy::Destroy,
        }
    }
}

pub fn handle_wipe_command(
    device: &str,
    policy: WipePolicyArg,
    sign: bool,
    danger_allow_wipe: bool,
    backup_cert_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Critical safety check: require SECUREWIPE_DANGER=1 environment variable
    if std::env::var("SECUREWIPE_DANGER").unwrap_or_default() != "1" {
        eprintln!("ERROR: SECUREWIPE_DANGER=1 environment variable required for destructive operations");
        eprintln!("This is a safety measure to prevent accidental data loss.");
        eprintln!("Run: SECUREWIPE_DANGER=1 securewipe wipe --device {} --danger-allow-wipe", device);
        std::process::exit(1);
    }

    // Require explicit danger flag
    if !danger_allow_wipe {
        eprintln!("ERROR: --danger-allow-wipe flag is required for destructive operations");
        eprintln!("This ensures you understand this will permanently destroy data on {}", device);
        std::process::exit(1);
    }

    // Check if device exists
    if !std::path::Path::new(device).exists() {
        eprintln!("ERROR: Device {} does not exist", device);
        std::process::exit(1);
    }

    let wipe_policy = WipePolicy::from(policy);
    
    // Plan the wipe first (safety check)
    println!("Planning wipe operation for device: {}", device);
    println!("Policy: {:?}", wipe_policy);
    
    // Detect if device is critical by checking if it contains root filesystem
    let is_critical = detect_critical_device(device)?;
    let iso_mode = std::env::var("SECUREWIPE_ISO_MODE").unwrap_or_default() == "1";
    
    if is_critical {
        println!("⚠️  WARNING: Device {} appears to contain system files (CRITICAL)", device);
        if !iso_mode {
            eprintln!("ERROR: Cannot wipe system disk unless running from bootable ISO mode");
            eprintln!("Set SECUREWIPE_ISO_MODE=1 if you are running from a bootable environment");
            std::process::exit(1);
        }
    }
    
    let plan = plan_wipe(device, Some(wipe_policy.clone()), is_critical, iso_mode, None, None);
    
    if plan.blocked {
        eprintln!("ERROR: Wipe operation blocked: {}", plan.reason.unwrap_or_default());
        std::process::exit(1);
    }

    // Show plan to user
    println!("Wipe Plan:");
    println!("  Device: {}", plan.device);
    println!("  Risk Level: {}", plan.risk);
    println!("  Policy: {:?}", plan.policy);
    println!("  Method: {}", plan.main_method);
    println!("  HPA/DCO Clear: {}", plan.hpa_dco_clear);
    println!("  Verification: {} samples using {}", plan.verification.samples, plan.verification.strategy);
    
    // Final confirmation prompt
    print!("This will PERMANENTLY DESTROY ALL DATA on {}. Type 'CONFIRM WIPE' to proceed: ", device);
    std::io::Write::flush(&mut std::io::stdout())?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if input != "CONFIRM WIPE" {
        println!("Wipe operation cancelled.");
        return Ok(());
    }

    println!("Starting destructive wipe operation...");
    
    // Perform the actual wipe
    let wipe_engine = NistAlignedWipe;
    let wipe_result = wipe_engine.perform_wipe(device, wipe_policy, is_critical)?;

    println!("Wipe operation completed!");
    println!("Method used: {}", wipe_result.method);
    println!("Commands executed: {}", wipe_result.commands.len());
    println!("Verification samples: {}", wipe_result.verification_samples);
    println!("Verification result: {}", if wipe_result.verification_passed { "PASSED" } else { "FAILED" });
    
    if let Some(reason) = &wipe_result.fallback_reason {
        println!("Fallback reason: {}", reason);
    }

    // Generate certificate
    let cert_manager = Ed25519CertificateManager;
    let wipe_cert = cert_manager.create_wipe_certificate(&wipe_result, backup_cert_id)?;
    
    // Save certificate to file
    let cert_dir = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join("SecureWipe")
        .join("certificates");
    std::fs::create_dir_all(&cert_dir)?;
    
    let cert_file = cert_dir.join(format!("{}.json", wipe_cert.cert_id));
    let cert_json = serde_json::to_string_pretty(&wipe_cert)?;
    std::fs::write(&cert_file, cert_json)?;
    
    println!("Wipe certificate saved to: {}", cert_file.display());

    // Generate PDF if requested
    if sign {
        let pdf_path = cert_manager.generate_wipe_certificate_pdf(&wipe_cert, Some("http://localhost:8000/verify"))?;
        println!("Signed PDF certificate generated: {}", pdf_path);
    }

    if !wipe_result.verification_passed {
        eprintln!("WARNING: Wipe verification failed! Some sectors may not be properly wiped.");
        std::process::exit(1);
    }

    Ok(())
}

pub fn handle_backup_command(
    device: &str,
    dest: &str,
    paths: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let backup_engine = EncryptedBackup::new();
    let source_paths = paths.unwrap_or_default();

    println!("Starting backup operation...");
    println!("Device: {}", device);
    println!("Destination: {}", dest);

    if source_paths.is_empty() {
        println!("Using default paths: ~/Documents, ~/Desktop, ~/Pictures");
    } else {
        println!("Source paths: {:?}", source_paths);
    }

    let result = backup_engine.perform_backup(device, &source_paths, dest)?;

    println!("Backup completed successfully!");
    println!("Backup ID: {}", result.backup_id);
    println!("Encryption: {}", result.encryption_method);
    println!("Files processed: {}", result.manifest.total_files);
    println!("Total bytes: {}", result.manifest.total_bytes);
    println!(
        "Verification samples: {}/{}",
        if result.verification_passed {
            result.verification_samples
        } else { 0 },
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
        eprintln!("WARNING: Backup verification failed! Some files may be corrupted.");
        std::process::exit(1);
    }

    Ok(())
}

    }
}

fn detect_critical_device(device: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Check if device contains mounted filesystems
    let mount_output = std::process::Command::new("mount")
        .output()?;
    
    let mount_text = String::from_utf8_lossy(&mount_output.stdout);
    
    // Look for this device in mount output
    for line in mount_text.lines() {
        if line.contains(device) {
            // Check if it's mounted on critical paths
            if line.contains(" / ") ||           // root filesystem
               line.contains(" /boot ") ||       // boot partition
               line.contains(" /usr ") ||        // usr partition
               line.contains(" /etc ") ||        // etc partition
               line.contains(" /bin ") ||        // bin partition
               line.contains(" /sbin ") {        // sbin partition
                return Ok(true);
            }
        }
    }
    
    // Also check lsblk to see if any partition on this device is mounted on critical paths
    let lsblk_output = std::process::Command::new("lsblk")
        .arg("-J")
        .arg("-o")
        .arg("NAME,MOUNTPOINT")
        .arg(device)
        .output()?;
    
    if lsblk_output.status.success() {
        let lsblk_text = String::from_utf8_lossy(&lsblk_output.stdout);
        if lsblk_text.contains("\"mountpoint\":\"/\"") ||
           lsblk_text.contains("\"mountpoint\":\"/boot\"") ||
           lsblk_text.contains("\"mountpoint\":\"/usr\"") ||
           lsblk_text.contains("\"mountpoint\":\"/etc\"") {
            return Ok(true);
        }
    }
    
    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { device, dest, paths } => {
            handle_backup_command(&device, &dest, paths)?;
        }
        Commands::Wipe { device, policy, sign, danger_allow_wipe, backup_cert_id } => {
            handle_wipe_command(&device, policy, sign, danger_allow_wipe, backup_cert_id.as_deref())?;
        }
    }

    Ok(())
}