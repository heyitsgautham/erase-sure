use crate::backup::{EncryptedBackup, BackupOperations};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "backup_cli")]
#[command(about = "A CLI for performing encrypted backups", long_about = None)]
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { device, dest, paths } => {
            handle_backup_command(&device, &dest, paths)?;
        }
    }

    Ok(())
}