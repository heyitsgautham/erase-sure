use crate::cert::{BackupCertificate, WipeCertificate};
use crate::pdf::{PdfGenerator, ensure_certificates_dir};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

/// High-level PDF certificate generation functions
pub struct CertificatePdfGenerator {
    verify_base_url: Option<String>,
    use_python_generator: bool,
}

impl CertificatePdfGenerator {
    pub fn new(verify_base_url: Option<String>) -> Self {
        Self { 
            verify_base_url,
            use_python_generator: true, // Default to Python for high quality
        }
    }

    pub fn with_rust_generator(verify_base_url: Option<String>) -> Self {
        Self { 
            verify_base_url,
            use_python_generator: false,
        }
    }

    /// Generate PDF for backup certificate and save to standard location
    pub fn generate_backup_certificate_pdf(
        &self,
        cert: &BackupCertificate,
    ) -> Result<PathBuf> {
        info!(cert_id = %cert.cert_id, "Generating backup certificate PDF");
        
        let certs_dir = ensure_certificates_dir()?;
        let pdf_generator = PdfGenerator::new(self.verify_base_url.clone());
        
        // Always use Python generator for high-quality PDFs
        if self.use_python_generator {
            info!("Using Python PDF generator for high-quality output");
            self.generate_backup_pdf_python(cert, &certs_dir)
        } else {
            info!("Using Rust PDF generator");
            pdf_generator.generate_backup_pdf(cert, &certs_dir)
        }
    }

    /// Generate PDF for backup certificate from JSON string (bypasses struct deserialization issues)
    pub fn generate_backup_pdf_from_json(
        &self,
        cert_json: &str,
    ) -> Result<PathBuf> {
        // Parse JSON to get cert_id
        let cert_value: serde_json::Value = serde_json::from_str(cert_json)?;
        let cert_id = cert_value.get("cert_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing cert_id in certificate"))?;
            
        info!(cert_id = %cert_id, "Generating backup certificate PDF from JSON");
        
        let certs_dir = ensure_certificates_dir()?;
        
        // Always use Python generator with no validation to handle unsigned certificates
        info!("Using Python PDF generator for high-quality output");
        self.call_python_generator(cert_json, &certs_dir.join(format!("{}.pdf", cert_id)), "backup")
    }

    /// Generate PDF for wipe certificate and save to standard location
    pub fn generate_wipe_certificate_pdf(
        &self,
        cert: &WipeCertificate,
    ) -> Result<PathBuf> {
        info!(cert_id = %cert.cert_id, "Generating wipe certificate PDF");
        
        let certs_dir = ensure_certificates_dir()?;
        
        if self.use_python_generator {
            self.generate_wipe_pdf_python(cert, &certs_dir)
        } else {
            let pdf_generator = PdfGenerator::new(self.verify_base_url.clone());
            pdf_generator.generate_wipe_pdf(cert, &certs_dir)
        }
    }

    /// Generate PDF for wipe certificate from JSON string (bypasses struct deserialization issues)
    pub fn generate_wipe_pdf_from_json(
        &self,
        cert_json: &str,
    ) -> Result<PathBuf> {
        // Parse JSON to get cert_id
        let cert_value: serde_json::Value = serde_json::from_str(cert_json)?;
        let cert_id = cert_value.get("cert_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing cert_id in certificate"))?;
            
        info!(cert_id = %cert_id, "Generating wipe certificate PDF from JSON");
        
        let certs_dir = ensure_certificates_dir()?;
        
        // Always use Python generator with no validation to handle unsigned certificates
        info!("Using Python PDF generator for high-quality output");
        self.call_python_generator(cert_json, &certs_dir.join(format!("{}.pdf", cert_id)), "wipe")
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

    /// Generate backup PDF using Python script (high quality)
    fn generate_backup_pdf_python(
        &self,
        cert: &BackupCertificate,
        certs_dir: &std::path::Path,
    ) -> Result<PathBuf> {
        let cert_json = serde_json::to_string_pretty(cert)?;
        let output_path = certs_dir.join(format!("{}.pdf", cert.cert_id));
        
        self.call_python_generator(&cert_json, &output_path, "backup")
    }

    /// Generate wipe PDF using Python script (high quality)
    fn generate_wipe_pdf_python(
        &self,
        cert: &WipeCertificate,
        certs_dir: &std::path::Path,
    ) -> Result<PathBuf> {
        let cert_json = serde_json::to_string_pretty(cert)?;
        let output_path = certs_dir.join(format!("{}.pdf", cert.cert_id));
        
        self.call_python_generator(&cert_json, &output_path, "wipe")
    }

    /// Call the Python PDF generator script
    fn call_python_generator(
        &self,
        cert_json: &str,
        output_path: &std::path::Path,
        cert_type: &str,
    ) -> Result<PathBuf> {
        use std::io::Write;
        
        info!("Using Python PDF generator for high-quality output");
        
        // Create temporary certificate file
        let temp_dir = std::env::temp_dir();
        let temp_cert_file = temp_dir.join(format!("cert_{}.json", uuid::Uuid::new_v4()));
        
        {
            let mut file = std::fs::File::create(&temp_cert_file)?;
            file.write_all(cert_json.as_bytes())?;
            file.flush()?;
        }
        
        // Find project root and Python script
        let current_dir = std::env::current_dir()?;
        let project_root = find_project_root(&current_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not find project root"))?;
        
        let python_script = project_root.join("core/src/python_pdf_generator.py");
        
        // Make sure Python script is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if python_script.exists() {
                let mut perms = std::fs::metadata(&python_script)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&python_script, perms)?;
            }
        }
        
        // Call Python script
        let output = Command::new("python3")
            .arg(&python_script)
            .arg("--cert-file")
            .arg(&temp_cert_file)
            .arg("--output")
            .arg(output_path)
            .arg("--type")
            .arg(cert_type)
            .arg("--no-validate")
            .current_dir(&project_root)
            .output()
            .map_err(|e| {
                anyhow::anyhow!("Failed to execute Python PDF generator: {}. Make sure python3 is available and required packages are installed (reportlab, jsonschema, qrcode[pil])", e)
            })?;
        
        // Clean up temp file
        if let Err(e) = std::fs::remove_file(&temp_cert_file) {
            warn!("Failed to clean up temp file: {}", e);
        }
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Python PDF generation failed:\nStderr: {}\nStdout: {}",
                stderr, stdout
            ));
        }
        
        // Check if file was created
        if !output_path.exists() {
            return Err(anyhow::anyhow!("Python script completed but PDF file was not created"));
        }
        
        info!("Python PDF generation completed successfully");
        info!("Python output: {}", String::from_utf8_lossy(&output.stdout));
        
        Ok(output_path.to_path_buf())
    }
}

/// Find project root by looking for characteristic directories
fn find_project_root(start_dir: &std::path::Path) -> Option<PathBuf> {
    let mut current = start_dir;
    
    loop {
        // Look for indicators of project root
        if current.join("tests").exists() && 
           current.join("core").exists() && 
           current.join("certs").exists() {
            return Some(current.to_path_buf());
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
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
            certificate_version: "v1.0.0".to_string(),
            created_at: "2023-12-05T14:30:22.123456Z".to_string(),
            issuer: serde_json::json!({"organization": "SecureWipe (SIH)"}),
            device: serde_json::json!({
                "model": "Test SSD 1TB",
                "serial": "TEST123456",
                "capacity_bytes": 1000000000000u64
            }),
            files_summary: serde_json::json!({
                "count": 100,
                "personal_bytes": 500000000u64
            }),
            destination: serde_json::json!({"type": "other"}),
            crypto: serde_json::json!({"alg": "AES-256-CTR", "manifest_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678"}),
            verification: serde_json::json!({"strategy": "sampled_files"}),
            policy: serde_json::json!({"name": "NIST SP 800-88 Rev.1"}),
            result: "PASS".to_string(),
            environment: serde_json::json!({"operator": "test"}),
            exceptions: serde_json::json!({"text": "None"}),
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_signature_data_here".to_string(),
            }),
            metadata: serde_json::json!({}),
            verify_url: "http://localhost:8000/verify".to_string(),
        }
    }

    fn create_test_wipe_cert() -> WipeCertificate {
        WipeCertificate {
            cert_id: "test_wipe_pdf_integration_456".to_string(),
            cert_type: "wipe".to_string(),
            certificate_version: "v1.0.0".to_string(),
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
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_wipe_signature_data_here".to_string(),
            }),
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
