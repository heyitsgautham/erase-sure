use clap::{Parser, Subcommand};
use std::process;

mod cmd;
mod device;
mod backup;
mod wipe;
mod cert;
mod logging;
mod signer;
mod schema;

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
    // Load .env file if present so SECUREWIPE_DANGER is available
    dotenvy::dotenv().ok();
    let logger = Logger::new();
    
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Discover(args) => cmd::handle_discover(args, &logger),
        Commands::Backup(args) => cmd::handle_backup(args, &logger),
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