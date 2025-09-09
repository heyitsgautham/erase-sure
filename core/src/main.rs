use clap::{Parser, Subcommand};
use std::process;

mod cmd;
mod device;
mod backup;
mod wipe;
mod cert;
mod logging;

use cmd::{DiscoverArgs, BackupArgs, WipeArgs, CertArgs};
use logging::Logger;
// ...existing code...

#[derive(Parser)]
#[command(name = "securewipe")]
#[command(about = "Secure backup and NIST-aligned disk wiping tool")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover storage devices with risk classification
    Discover(DiscoverArgs),
    /// Perform encrypted backup to external storage
    Backup(BackupArgs),
    /// Execute NIST-aligned disk wipe operations
    Wipe(WipeArgs),
    /// Show or export stored certificates
    Cert(CertArgs),
}

fn main() {
    let logger = Logger::new();
    
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Discover(args) => cmd::handle_discover(args, &logger),
        Commands::Backup(args) => {
            use backup::{EncryptedBackup, BackupOperations};
            let backup_engine = EncryptedBackup::new();
            
            let paths = &args.paths;
            
            match backup_engine.perform_backup(&args.device, &paths, &args.dest) {
                Ok(result) => {
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
                        eprintln!("WARNING: Backup verification failed! Some files may be corrupted.");
                        std::process::exit(1);
                    }
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Backup failed: {}", e))
            }
        }
        Commands::Wipe(args) => cmd::handle_wipe(args, &logger),
        Commands::Cert(args) => cmd::handle_cert(args, &logger),
    };
    
    match result {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}