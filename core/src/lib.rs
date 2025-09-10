pub mod backup;
pub mod cert;
pub mod device;
pub mod wipe;
pub mod logging;
pub mod pdf;
pub mod cert_pdf;
pub mod signer;

// Re-export commonly used types for easier integration testing
pub use backup::{BackupOperations, EncryptedBackup, BackupResult, BackupManifest};
pub use cert::{CertificateOperations, Ed25519CertificateManager, BackupCertificate, WipeCertificate, CertificateSignature};
pub use device::{DeviceDiscovery, LinuxDeviceDiscovery, Device, RiskLevel};
pub use wipe::{WipeOperations, NistAlignedWipe, WipeResult, WipePolicy, WipeCommand};
pub use logging::Logger;
pub use pdf::{PdfGenerator, ensure_certificates_dir, extract_embedded_json};
pub use cert_pdf::{CertificatePdfGenerator, generate_backup_pdf, generate_wipe_pdf};
pub use signer::{load_private_key, canonicalize_json, sign_certificate, verify_certificate_signature, SignerError};