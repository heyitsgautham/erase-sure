use crate::cert::{BackupCertificate, WipeCertificate};
use crate::pdf::{PdfGenerator, ensure_certificates_dir};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// High-level PDF certificate generation functions
pub struct CertificatePdfGenerator {
    verify_base_url: Option<String>,
}

impl CertificatePdfGenerator {
    pub fn new(verify_base_url: Option<String>) -> Self {
        Self { verify_base_url }
    }

    /// Generate PDF for backup certificate and save to standard location
    pub fn generate_backup_certificate_pdf(
        &self,
        cert: &BackupCertificate,
    ) -> Result<PathBuf> {
        info!(cert_id = %cert.cert_id, "Generating backup certificate PDF");
        
        let certs_dir = ensure_certificates_dir()?;
        let pdf_generator = PdfGenerator::new(self.verify_base_url.clone());
        
        pdf_generator.generate_backup_pdf(cert, &certs_dir)
    }

    /// Generate PDF for wipe certificate and save to standard location
    pub fn generate_wipe_certificate_pdf(
        &self,
        cert: &WipeCertificate,
    ) -> Result<PathBuf> {
        info!(cert_id = %cert.cert_id, "Generating wipe certificate PDF");
        
        let certs_dir = ensure_certificates_dir()?;
        let pdf_generator = PdfGenerator::new(self.verify_base_url.clone());
        
        pdf_generator.generate_wipe_pdf(cert, &certs_dir)
    }

    /// Generate PDF for certificate from JSON and certificate type
    pub fn generate_certificate_pdf_from_json(
        &self,
        cert_json: &str,
        cert_type: &str,
    ) -> Result<PathBuf> {
        match cert_type {
            "backup" => {
                let cert: BackupCertificate = serde_json::from_str(cert_json)?;
                self.generate_backup_certificate_pdf(&cert)
            }
            "wipe" => {
                let cert: WipeCertificate = serde_json::from_str(cert_json)?;
                self.generate_wipe_certificate_pdf(&cert)
            }
            _ => {
                anyhow::bail!("Unsupported certificate type: {}", cert_type);
            }
        }
    }
}

/// Convenience function to generate backup certificate PDF
pub fn generate_backup_pdf(
    cert: &BackupCertificate,
    verify_url: Option<&str>,
) -> Result<PathBuf> {
    let generator = CertificatePdfGenerator::new(verify_url.map(|s| s.to_string()));
    generator.generate_backup_certificate_pdf(cert)
}

/// Convenience function to generate wipe certificate PDF
pub fn generate_wipe_pdf(
    cert: &WipeCertificate,
    verify_url: Option<&str>,
) -> Result<PathBuf> {
    let generator = CertificatePdfGenerator::new(verify_url.map(|s| s.to_string()));
    generator.generate_wipe_certificate_pdf(cert)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::CertificateSignature;

    fn create_test_backup_cert() -> BackupCertificate {
        BackupCertificate {
            cert_id: "test_backup_pdf_integration_123".to_string(),
            cert_type: "backup".to_string(),
            created_at: "2023-12-05T14:30:22.123456Z".to_string(),
            device: serde_json::json!({
                "model": "Test SSD 1TB",
                "serial": "TEST123456",
                "capacity_bytes": 1000000000000u64
            }),
            backup_summary: serde_json::json!({
                "files": 100,
                "bytes": 500000000u64
            }),
            manifest_sha256: "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_signature_data_here".to_string(),
            },
        }
    }

    fn create_test_wipe_cert() -> WipeCertificate {
        WipeCertificate {
            cert_id: "test_wipe_pdf_integration_456".to_string(),
            cert_type: "wipe".to_string(),
            created_at: "2023-12-05T15:00:30.654321Z".to_string(),
            device: serde_json::json!({
                "model": "Test SSD 1TB",
                "serial": "TEST123456",
                "capacity_bytes": 1000000000000u64
            }),
            wipe_summary: serde_json::json!({
                "policy": "PURGE",
                "method": "nvme_sanitize",
                "verification_samples": 5,
                "verification_passed": true
            }),
            linkage: Some(serde_json::json!({
                "backup_cert_id": "test_backup_123"
            })),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_wipe_signature_data_here".to_string(),
            },
        }
    }

    #[test]
    fn test_certificate_pdf_generator_backup() {
        let generator = CertificatePdfGenerator::new(Some("https://verify.example.com".to_string()));
        let cert = create_test_backup_cert();

        let result = generator.generate_backup_certificate_pdf(&cert);
        assert!(result.is_ok());

        let pdf_path = result.unwrap();
        assert!(pdf_path.exists());
        assert_eq!(pdf_path.extension().unwrap(), "pdf");
        assert!(pdf_path.file_stem().unwrap().to_string_lossy().contains("test_backup_pdf_integration_123"));
    }

    #[test]
    fn test_certificate_pdf_generator_wipe() {
        let generator = CertificatePdfGenerator::new(None);
        let cert = create_test_wipe_cert();

        let result = generator.generate_wipe_certificate_pdf(&cert);
        assert!(result.is_ok());

        let pdf_path = result.unwrap();
        assert!(pdf_path.exists());
        assert_eq!(pdf_path.extension().unwrap(), "pdf");
        assert!(pdf_path.file_stem().unwrap().to_string_lossy().contains("test_wipe_pdf_integration_456"));
    }

    #[test]
    fn test_generate_certificate_pdf_from_json() {
        let generator = CertificatePdfGenerator::new(Some("https://verify.test.com".to_string()));
        
        // Test backup certificate
        let backup_cert = create_test_backup_cert();
        let backup_json = serde_json::to_string(&backup_cert).unwrap();
        
        let result = generator.generate_certificate_pdf_from_json(&backup_json, "backup");
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
        
        // Test wipe certificate
        let wipe_cert = create_test_wipe_cert();
        let wipe_json = serde_json::to_string(&wipe_cert).unwrap();
        
        let result = generator.generate_certificate_pdf_from_json(&wipe_json, "wipe");
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
        
        // Test invalid certificate type
        let result = generator.generate_certificate_pdf_from_json(&backup_json, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_convenience_functions() {
        let backup_cert = create_test_backup_cert();
        let wipe_cert = create_test_wipe_cert();

        // Test backup convenience function
        let result = generate_backup_pdf(&backup_cert, Some("https://verify.example.com"));
        assert!(result.is_ok());
        assert!(result.unwrap().exists());

        // Test wipe convenience function
        let result = generate_wipe_pdf(&wipe_cert, None);
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}
