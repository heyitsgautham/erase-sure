use crate::cert::{BackupCertificate, WipeCertificate};
use anyhow::{Context, Result};
use genpdf::{Document, Element};
use genpdf::elements::{Paragraph, Break, LinearLayout};
use genpdf::fonts;
use genpdf::style::Style;
use qrcode::QrCode;
use image::{DynamicImage, ImageFormat};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Cursor;
use tracing::{info, warn};

pub struct PdfGenerator {
    verify_base_url: Option<String>,
}

impl PdfGenerator {
    pub fn new(verify_base_url: Option<String>) -> Self {
        Self { verify_base_url }
    }

    /// Generate PDF certificate from backup certificate JSON
    pub fn generate_backup_pdf(
        &self,
        cert: &BackupCertificate,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        info!(
            cert_id = %cert.cert_id,
            cert_type = %cert.cert_type,
            "Generating backup certificate PDF"
        );

        let pdf_path = output_dir.join(format!("{}.pdf", cert.cert_id));
        
        // Create simple document
        let mut doc = self.create_document("SecureWipe Backup Certificate")?;
        
        // Add content
        self.add_backup_content(&mut doc, cert)?;

        // Render to file
        doc.render_to_file(&pdf_path)
            .with_context(|| format!("Failed to render PDF to {}", pdf_path.display()))?;

        info!(pdf_path = %pdf_path.display(), "Backup certificate PDF generated successfully");
        Ok(pdf_path)
    }

    /// Generate PDF certificate from wipe certificate JSON
    pub fn generate_wipe_pdf(
        &self,
        cert: &WipeCertificate,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        info!(
            cert_id = %cert.cert_id,
            cert_type = %cert.cert_type,
            "Generating wipe certificate PDF"
        );

        let pdf_path = output_dir.join(format!("{}.pdf", cert.cert_id));
        
        // Create simple document
        let mut doc = self.create_document("SecureWipe Data Sanitization Certificate")?;
        
        // Add content
        self.add_wipe_content(&mut doc, cert)?;

        // Render to file
        doc.render_to_file(&pdf_path)
            .with_context(|| format!("Failed to render PDF to {}", pdf_path.display()))?;

        info!(pdf_path = %pdf_path.display(), "Wipe certificate PDF generated successfully");
        Ok(pdf_path)
    }

    /// Create a simple document with default settings
    fn create_document(&self, title: &str) -> Result<Document> {
        // Use a simple approach - create minimal font
        let regular_font = vec![0u8; 1024]; // Minimal dummy font data
        let font_data = fonts::FontData::new(regular_font, None)?;
        let font_family = fonts::FontFamily {
            regular: font_data.clone(),
            bold: font_data.clone(),
            italic: font_data.clone(),
            bold_italic: font_data,
        };
        
        let mut doc = Document::new(font_family);
        doc.set_title(title);
        doc.set_minimal_conformance();
        doc.set_line_spacing(1.25);
        
        Ok(doc)
    }

    /// Add backup certificate content
    fn add_backup_content(&self, doc: &mut Document, cert: &BackupCertificate) -> Result<()> {
        // Title
        doc.push(
            Paragraph::new("SecureWipe Backup Certificate")
                .styled(Style::new().bold().with_font_size(20))
        );
        
        doc.push(
            Paragraph::new("NIST SP 800-88 Compliant Data Backup Operation")
                .styled(Style::new().with_font_size(12))
        );
        
        doc.push(Break::new(1.5));

        // Certificate Information
        doc.push(Paragraph::new("Certificate Information").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Certificate ID: {}", cert.cert_id)));
        doc.push(Paragraph::new(&format!("Certificate Type: {}", cert.cert_type.to_uppercase())));
        doc.push(Paragraph::new(&format!("Created: {}", cert.created_at)));
        doc.push(Paragraph::new(&format!("Encryption Method: {}", cert.encryption_method)));
        doc.push(Break::new(1.0));

        // Device Information
        doc.push(Paragraph::new("Device Information").styled(Style::new().bold().with_font_size(16)));
        if let Some(model) = cert.device.get("model").and_then(|v| v.as_str()) {
            doc.push(Paragraph::new(&format!("Model: {}", model)));
        }
        if let Some(serial) = cert.device.get("serial").and_then(|v| v.as_str()) {
            doc.push(Paragraph::new(&format!("Serial Number: {}", serial)));
        }
        if let Some(capacity) = cert.device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            doc.push(Paragraph::new(&format!("Capacity: {}", self.format_bytes(capacity))));
        }
        doc.push(Break::new(1.0));

        // Backup Summary
        doc.push(Paragraph::new("Backup Summary").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Files Count: {}", 
            cert.backup_summary.get("files").unwrap_or(&serde_json::Value::Null))));
        doc.push(Paragraph::new(&format!("Total Bytes: {}", 
            self.format_bytes(cert.backup_summary.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0)))));
        doc.push(Paragraph::new(&format!("Manifest SHA256: {}", 
            self.format_hash(&cert.manifest_sha256))));
        doc.push(Break::new(1.0));

        // Digital Signature
        doc.push(Paragraph::new("Digital Signature").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Algorithm: {}", cert.signature.alg)));
        doc.push(Paragraph::new(&format!("Public Key ID: {}", cert.signature.pubkey_id)));
        doc.push(Paragraph::new(&format!("Signature: {}", self.format_hash(&cert.signature.sig))));
        doc.push(Break::new(1.0));

        // QR Code info
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert.cert_id)
        } else {
            format!("cert_id:{}", cert.cert_id)
        };
        doc.push(Paragraph::new("Verification QR Code").styled(Style::new().bold().with_font_size(14)));
        doc.push(Paragraph::new(&format!("QR Code Data: {}", qr_data)));
        doc.push(Break::new(2.0));

        // Footer
        doc.push(
            Paragraph::new("Generated by SecureWipe - NIST SP 800-88 Compliant")
                .styled(Style::new().with_font_size(10).italic())
        );

        Ok(())
    }

    /// Add wipe certificate content
    fn add_wipe_content(&self, doc: &mut Document, cert: &WipeCertificate) -> Result<()> {
        // Title
        doc.push(
            Paragraph::new("SecureWipe Data Sanitization Certificate")
                .styled(Style::new().bold().with_font_size(20))
        );
        
        doc.push(
            Paragraph::new("NIST SP 800-88 Compliant Wipe Operation")
                .styled(Style::new().with_font_size(12))
        );
        
        doc.push(Break::new(1.5));

        // Certificate Information
        doc.push(Paragraph::new("Certificate Information").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Certificate ID: {}", cert.cert_id)));
        doc.push(Paragraph::new(&format!("Certificate Type: {}", cert.cert_type.to_uppercase())));
        doc.push(Paragraph::new(&format!("Created: {}", cert.created_at)));
        
        let result = cert.wipe_summary.get("verification_passed")
            .and_then(|v| v.as_bool())
            .map(|passed| if passed { "PASS" } else { "FAIL" })
            .unwrap_or("UNKNOWN");
        doc.push(Paragraph::new(&format!("Result: {}", result)));
        doc.push(Break::new(1.0));

        // Device Information
        doc.push(Paragraph::new("Device Information").styled(Style::new().bold().with_font_size(16)));
        if let Some(model) = cert.device.get("model").and_then(|v| v.as_str()) {
            doc.push(Paragraph::new(&format!("Model: {}", model)));
        }
        if let Some(serial) = cert.device.get("serial").and_then(|v| v.as_str()) {
            doc.push(Paragraph::new(&format!("Serial Number: {}", serial)));
        }
        if let Some(capacity) = cert.device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            doc.push(Paragraph::new(&format!("Capacity: {}", self.format_bytes(capacity))));
        }
        doc.push(Break::new(1.0));

        // Sanitization Policy  
        doc.push(Paragraph::new("Sanitization Policy").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("NIST Level: {}", 
            cert.wipe_summary.get("policy").unwrap_or(&serde_json::Value::Null))));
        doc.push(Paragraph::new(&format!("Method: {}", 
            cert.wipe_summary.get("method").unwrap_or(&serde_json::Value::Null))));
        doc.push(Break::new(1.0));

        // Verification Results
        doc.push(Paragraph::new("Verification Results").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Samples Verified: {}", 
            cert.wipe_summary.get("verification_samples").unwrap_or(&serde_json::Value::Null))));
        doc.push(Paragraph::new(&format!("Verification Passed: {}", 
            cert.wipe_summary.get("verification_passed").unwrap_or(&serde_json::Value::Null))));
        doc.push(Break::new(1.0));

        // Linkage (if present)
        if let Some(linkage) = &cert.linkage {
            if let Some(backup_cert_id) = linkage.get("backup_cert_id") {
                doc.push(Paragraph::new("Evidence & Linkage").styled(Style::new().bold().with_font_size(16)));
                doc.push(Paragraph::new(&format!("Backup Certificate ID: {}", backup_cert_id)));
                doc.push(Break::new(1.0));
            }
        }

        // Digital Signature
        doc.push(Paragraph::new("Digital Signature & Verification").styled(Style::new().bold().with_font_size(16)));
        doc.push(Paragraph::new(&format!("Algorithm: {}", cert.signature.alg)));
        doc.push(Paragraph::new(&format!("Public Key ID: {}", cert.signature.pubkey_id)));
        doc.push(Paragraph::new(&format!("Signature: {}", self.format_hash(&cert.signature.sig))));
        doc.push(Break::new(1.0));

        // QR Code info  
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert.cert_id)
        } else {
            format!("cert_id:{}", cert.cert_id)
        };
        doc.push(Paragraph::new("Verification QR Code").styled(Style::new().bold().with_font_size(14)));
        doc.push(Paragraph::new(&format!("QR Code Data: {}", qr_data)));
        doc.push(Break::new(2.0));

        // Footer
        doc.push(
            Paragraph::new("Generated by SecureWipe - NIST SP 800-88 Compliant")
                .styled(Style::new().with_font_size(10).italic())
        );

        Ok(())
    }

    /// Format bytes in human readable format
    fn format_bytes(&self, bytes: u64) -> String {
        if bytes >= 1_000_000_000_000 {
            format!("{:.2} TB", bytes as f64 / 1_000_000_000_000.0)
        } else if bytes >= 1_000_000_000 {
            format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.2} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.2} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} bytes", bytes)
        }
    }

    /// Format hash for display (truncate long hashes)
    fn format_hash(&self, hash: &str) -> String {
        if hash.len() > 32 {
            format!("{}...{}", &hash[..16], &hash[hash.len()-8..])
        } else {
            hash.to_string()
        }
    }
}

/// Ensure the certificates directory exists
pub fn ensure_certificates_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .context("Failed to get home directory")?;
    
    let certs_dir = home_dir.join("SecureWipe").join("certificates");
    
    if !certs_dir.exists() {
        fs::create_dir_all(&certs_dir)
            .with_context(|| format!("Failed to create certificates directory: {}", certs_dir.display()))?;
        info!(path = %certs_dir.display(), "Created certificates directory");
    }
    
    Ok(certs_dir)
}

/// Extract embedded JSON from PDF (helper for testing)
pub fn extract_embedded_json(pdf_path: &Path) -> Result<Option<String>> {
    // This is a placeholder implementation
    // In a real implementation, you would parse the PDF and extract the embedded JSON
    warn!(pdf_path = %pdf_path.display(), "extract_embedded_json is not yet implemented");
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::{BackupCertificate, WipeCertificate, CertificateSignature};
    use tempfile::TempDir;
    use std::fs;

    fn create_test_backup_cert() -> BackupCertificate {
        BackupCertificate {
            cert_id: "test_backup_123".to_string(),
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
            cert_id: "test_wipe_456".to_string(),
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
    fn test_backup_pdf_generation() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_generator = PdfGenerator::new(Some("https://verify.securewipe.local".to_string()));
        let cert = create_test_backup_cert();

        let result = pdf_generator.generate_backup_pdf(&cert, temp_dir.path());
        assert!(result.is_ok());

        let pdf_path = result.unwrap();
        assert!(pdf_path.exists());
        assert!(pdf_path.extension().unwrap() == "pdf");
        assert!(pdf_path.file_stem().unwrap() == "test_backup_123");

        // Verify file is not empty
        let metadata = fs::metadata(&pdf_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_wipe_pdf_generation() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_generator = PdfGenerator::new(None);
        let cert = create_test_wipe_cert();

        let result = pdf_generator.generate_wipe_pdf(&cert, temp_dir.path());
        assert!(result.is_ok());

        let pdf_path = result.unwrap();
        assert!(pdf_path.exists());
        assert!(pdf_path.extension().unwrap() == "pdf");
        assert!(pdf_path.file_stem().unwrap() == "test_wipe_456");

        // Verify file is not empty
        let metadata = fs::metadata(&pdf_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_format_bytes() {
        let generator = PdfGenerator::new(None);
        
        assert_eq!(generator.format_bytes(1024), "1.02 KB");
        assert_eq!(generator.format_bytes(1_000_000), "1.00 MB");
        assert_eq!(generator.format_bytes(1_000_000_000), "1.00 GB");
        assert_eq!(generator.format_bytes(1_000_000_000_000), "1.00 TB");
        assert_eq!(generator.format_bytes(500), "500 bytes");
    }

    #[test]
    fn test_format_hash() {
        let generator = PdfGenerator::new(None);
        
        let short_hash = "abc123";
        assert_eq!(generator.format_hash(short_hash), "abc123");
        
        let long_hash = "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678";
        let formatted = generator.format_hash(long_hash);
        assert!(formatted.len() < long_hash.len());
        assert!(formatted.contains("..."));
    }
}
