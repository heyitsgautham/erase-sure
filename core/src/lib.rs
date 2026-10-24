pub mod backup;
pub mod cert;
pub mod cert_pdf;
pub mod device;
pub mod logging;
pub mod pdf;
pub mod schema;
pub mod signer;
pub mod wipe;

// Re-export commonly used types for easier integration testing
pub use backup::{BackupManifest, BackupOperations, BackupResult, EncryptedBackup};
pub use cert::{
    BackupCertificate, CertificateOperations, CertificateSignature, Ed25519CertificateManager,
    WipeCertificate,
};
pub use cert_pdf::{generate_backup_pdf, generate_wipe_pdf, CertificatePdfGenerator};
pub use device::{Device, DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel};
pub use logging::Logger;
pub use pdf::{ensure_certificates_dir, extract_embedded_json, PdfGenerator};
pub use schema::{
    validate_certificate, validate_certificate_file, validate_certificate_json,
    CertificateValidator, ValidationResult,
};
pub use signer::{
    canonicalize_json, load_private_key, sign_certificate, verify_certificate_signature,
    SignerError,
};
pub use wipe::{NistAlignedWipe, WipeCommand, WipeOperations, WipePolicy, WipeResult};
