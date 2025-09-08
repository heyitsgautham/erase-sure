pub mod backup;
pub mod cert;
pub mod device;
pub mod wipe;
pub mod logging;

// Re-export commonly used types for easier integration testing
pub use backup::{BackupOperations, EncryptedBackup, BackupResult, BackupManifest};
pub use cert::{CertificateOperations, Ed25519CertificateManager, BackupCertificate, WipeCertificate, CertificateSignature};
pub use device::{DeviceDiscovery, LinuxDeviceDiscovery, Device, RiskLevel};
pub use wipe::{WipeOperations, NistAlignedWipe, WipeResult, WipePolicy, WipeCommand};
pub use logging::Logger;