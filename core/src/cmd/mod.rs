//! CLI subcommand arguments and handlers, one module per subcommand.

mod backup;
mod cert;
mod discover;
mod wipe;

#[cfg(test)]
mod tests;

pub use backup::{handle_backup, BackupArgs};
pub use cert::{handle_cert, CertArgs};
pub use discover::{handle_discover, DiscoverArgs};
pub use wipe::{handle_wipe, WipeArgs};
