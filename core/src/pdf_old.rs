use crate::cert::{BackupCertificate, WipeCertificate};
use anyhow::{Context, Result};
use genpdf::{Document, Element};
use genpdf::elements::{Paragraph, Image, Break, LinearLayout, StyledElement};
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
        
        // Create the document with builtin fonts
        let default_font = fonts::from_bytes(include_bytes!("../../default.ttf"));
        let font_family = match default_font {
            Ok(font) => font,
            Err(_) => {
                // Fallback to creating minimal font family
                fonts::FontFamily {
                    regular: fonts::FontData::new(vec![], None)?,
                    bold: fonts::FontData::new(vec![], None)?,
                    italic: fonts::FontData::new(vec![], None)?,
                    bold_italic: fonts::FontData::new(vec![], None)?,
                }
            }
        };
        let mut doc = Document::new(font_family);
        doc.set_title("SecureWipe Backup Certificate");
        doc.set_minimal_conformance();
        doc.set_line_spacing(1.25);

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
        
        // Create the document with builtin fonts
        let font_family = fonts::Builtin::Helvetica;
        let mut doc = Document::new(font_family);
        doc.set_title("SecureWipe Data Sanitization Certificate");
        doc.set_minimal_conformance();
        doc.set_line_spacing(1.25);

        // Add content
        self.add_wipe_content(&mut doc, cert)?;

        // Render to file
        doc.render_to_file(&pdf_path)
            .with_context(|| format!("Failed to render PDF to {}", pdf_path.display()))?;

        info!(pdf_path = %pdf_path.display(), "Wipe certificate PDF generated successfully");
        Ok(pdf_path)
    }

    /// Add backup certificate content to the document
    fn add_backup_content(&self, doc: &mut Document, cert: &BackupCertificate) -> Result<()> {
        // Title
        doc.push(
            Paragraph::new("SecureWipe Backup Certificate")
                .styled(Style::new().bold().with_font_size(24))
        );
        
        doc.push(
            Paragraph::new("NIST SP 800-88 Compliant Data Backup Operation")
                .styled(Style::new().with_font_size(14))
        );
        
        doc.push(Break::new(1.5));

        // Certificate Information
        let cert_info = vec![
            vec!["Certificate ID:".to_string(), cert.cert_id.clone()],
            vec!["Certificate Type:".to_string(), cert.cert_type.to_uppercase()],
            vec!["Created:".to_string(), cert.created_at.clone()],
            vec!["Encryption Method:".to_string(), cert.encryption_method.clone()],
        ];
        
        doc.push(self.create_info_table("Certificate Information", cert_info)?);
        doc.push(Break::new(1.0));

        // Device Information
        let device_info = self.extract_device_info(&cert.device);
        doc.push(self.create_info_table("Device Information", device_info)?);
        doc.push(Break::new(1.0));

        // Backup Summary
        let backup_info = vec![
            vec!["Files Count:".to_string(), cert.backup_summary.get("files").unwrap_or(&serde_json::Value::Null).to_string()],
            vec!["Total Bytes:".to_string(), self.format_bytes(cert.backup_summary.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0))],
            vec!["Manifest SHA256:".to_string(), self.format_hash(&cert.manifest_sha256)],
        ];
        doc.push(self.create_info_table("Backup Summary", backup_info)?);
        doc.push(Break::new(1.0));

        // Digital Signature
        match &cert.signature {
            Some(signature) => {
                let sig_info = vec![
                    vec!["Algorithm:".to_string(), signature.alg.clone()],
                    vec!["Public Key ID:".to_string(), signature.pubkey_id.clone()],
                    vec!["Signature:".to_string(), self.format_hash(&signature.sig)],
                ];
                doc.push(self.create_info_table("Digital Signature", sig_info)?);
            }
            None => {
                let sig_info = vec![
                    vec!["Status:".to_string(), "Unsigned Certificate".to_string()],
                    vec!["Note:".to_string(), "This certificate has not been digitally signed".to_string()],
                ];
                doc.push(self.create_info_table("Digital Signature", sig_info)?);
            }
        }
        doc.push(Break::new(1.0));

        // QR Code for verification
        if let Ok(qr_image) = self.generate_qr_code(&cert.cert_id) {
            doc.push(Paragraph::new("Verification QR Code").styled(Style::new().bold().with_font_size(14)));
            doc.push(qr_image);
        }

        // Footer
        doc.push(Break::new(2.0));
        doc.push(
            Paragraph::new("Generated by SecureWipe - NIST SP 800-88 Compliant")
                .styled(Style::new().with_font_size(10).italic())
        );

        Ok(())
    }

    /// Add wipe certificate content to the document
    fn add_wipe_content(&self, doc: &mut Document, cert: &WipeCertificate) -> Result<()> {
        // Title
        doc.push(
            Paragraph::new("SecureWipe Data Sanitization Certificate")
                .styled(Style::new().bold().with_font_size(24))
        );
        
        doc.push(
            Paragraph::new("NIST SP 800-88 Compliant Wipe Operation")
                .styled(Style::new().with_font_size(14))
        );
        
        doc.push(Break::new(1.5));

        // Certificate Information
        let cert_info = vec![
            vec!["Certificate ID:".to_string(), cert.cert_id.clone()],
            vec!["Certificate Type:".to_string(), cert.cert_type.to_uppercase()],
            vec!["Created:".to_string(), cert.created_at.clone()],
            vec!["Result:".to_string(), cert.wipe_summary.get("verification_passed")
                .and_then(|v| v.as_bool())
                .map(|passed| if passed { "PASS" } else { "FAIL" })
                .unwrap_or("UNKNOWN").to_string()],
        ];
        
        doc.push(self.create_info_table("Certificate Information", cert_info)?);
        doc.push(Break::new(1.0));

        // Device Information
        let device_info = self.extract_device_info(&cert.device);
        doc.push(self.create_info_table("Device Information", device_info)?);
        doc.push(Break::new(1.0));

        // Sanitization Policy
        let policy_info = vec![
            vec!["NIST Level:".to_string(), cert.wipe_summary.get("policy").unwrap_or(&serde_json::Value::Null).to_string()],
            vec!["Method:".to_string(), cert.wipe_summary.get("method").unwrap_or(&serde_json::Value::Null).to_string()],
        ];
        doc.push(self.create_info_table("Sanitization Policy", policy_info)?);
        doc.push(Break::new(1.0));

        // Verification Results
        let verify_info = vec![
            vec!["Samples Verified:".to_string(), cert.wipe_summary.get("verification_samples").unwrap_or(&serde_json::Value::Null).to_string()],
            vec!["Verification Passed:".to_string(), cert.wipe_summary.get("verification_passed").unwrap_or(&serde_json::Value::Null).to_string()],
        ];
        doc.push(self.create_info_table("Verification Results", verify_info)?);
        doc.push(Break::new(1.0));

        // Linkage (if present)
        if let Some(linkage) = &cert.linkage {
            if let Some(backup_cert_id) = linkage.get("backup_cert_id") {
                let linkage_info = vec![
                    vec!["Backup Certificate ID:".to_string(), backup_cert_id.to_string()],
                ];
                doc.push(self.create_info_table("Evidence & Linkage", linkage_info)?);
                doc.push(Break::new(1.0));
            }
        }

        // Digital Signature
        match &cert.signature {
            Some(signature) => {
                let sig_info = vec![
                    vec!["Algorithm:".to_string(), signature.alg.clone()],
                    vec!["Public Key ID:".to_string(), signature.pubkey_id.clone()],
                    vec!["Signature:".to_string(), self.format_hash(&signature.sig)],
                ];
                doc.push(self.create_info_table("Digital Signature & Verification", sig_info)?);
            }
            None => {
                let sig_info = vec![
                    vec!["Status:".to_string(), "Unsigned Certificate".to_string()],
                    vec!["Note:".to_string(), "This certificate has not been digitally signed".to_string()],
                ];
                doc.push(self.create_info_table("Digital Signature & Verification", sig_info)?);
            }
        }
        doc.push(Break::new(1.0));

        // QR Code for verification
        if let Ok(qr_image) = self.generate_qr_code(&cert.cert_id) {
            doc.push(Paragraph::new("Verification QR Code").styled(Style::new().bold().with_font_size(14)));
            doc.push(qr_image);
        }

        // Footer
        doc.push(Break::new(2.0));
        doc.push(
            Paragraph::new("Generated by SecureWipe - NIST SP 800-88 Compliant")
                .styled(Style::new().with_font_size(10).italic())
        );

        Ok(())
    }

    /// Create an information table with a header
    fn create_info_table(&self, title: &str, data: Vec<Vec<String>>) -> Result<LinearLayout> {
        let mut layout = LinearLayout::vertical();
        
        // Add section header
        layout.push(
            Paragraph::new(title)
                .styled(Style::new().bold().with_font_size(16))
        );
        
        // Create table
        let mut table = Table::new(2);
        table.set_header_row(0);
        
        for row in data {
            if row.len() >= 2 {
                table.push_row([
                    StyledElement::new(Paragraph::new(&row[0]), Style::new().bold()),
                    StyledElement::new(Paragraph::new(&row[1]), Style::new()),
                ]);
            }
        }
        
        table.set_layout(TableLayout::new(vec![1, 2]));
        layout.push(table);
        
        Ok(layout)
    }

    /// Extract device information from JSON
    fn extract_device_info(&self, device: &serde_json::Value) -> Vec<Vec<String>> {
        let mut info = Vec::new();
        
        if let Some(model) = device.get("model").and_then(|v| v.as_str()) {
            info.push(vec!["Model:".to_string(), model.to_string()]);
        }
        if let Some(serial) = device.get("serial").and_then(|v| v.as_str()) {
            info.push(vec!["Serial Number:".to_string(), serial.to_string()]);
        }
        if let Some(capacity) = device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            let capacity_str = self.format_bytes(capacity);
            info.push(vec!["Capacity:".to_string(), capacity_str]);
        }
        
        info
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

    /// Generate QR code as an image element
    fn generate_qr_code(&self, cert_id: &str) -> Result<Image> {
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert_id)
        } else {
            format!("cert_id:{}", cert_id)
        };

        info!(qr_data = %qr_data, "Generating QR code");

        let qr_code = QrCode::new(&qr_data)
            .with_context(|| format!("Failed to generate QR code for data: {}", qr_data))?;

        // Convert QR code to image
        let qr_image = qr_code.render::<image::Luma<u8>>().build();
        
        // Convert to RGB for better compatibility
        let rgb_image = DynamicImage::ImageLuma8(qr_image).to_rgb8();
        let dynamic_image = DynamicImage::ImageRgb8(rgb_image);

        // Convert to bytes
        let mut image_bytes = Vec::new();
        {
            let mut cursor = Cursor::new(&mut image_bytes);
            dynamic_image.write_to(&mut cursor, ImageFormat::Png)
                .context("Failed to encode QR code as PNG")?;
        }

        // Create genpdf Image from bytes
        let image = Image::from_dynamic_image(dynamic_image)
            .context("Failed to create Image from QR code")?
            .with_alignment(genpdf::Alignment::Center)
            .with_scale(genpdf::Scale::new(0.3, 0.3));

        Ok(image)
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
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_signature_data_here".to_string(),
            }),
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
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "test_wipe_signature_data_here".to_string(),
            }),
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

    #[test]
    fn test_ensure_certificates_dir() {
        // This test uses a temporary directory to avoid affecting the real home directory
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let result = ensure_certificates_dir();
        assert!(result.is_ok());

        let certs_dir = result.unwrap();
        assert!(certs_dir.exists());
        assert!(certs_dir.is_dir());
        assert!(certs_dir.ends_with("SecureWipe/certificates"));
    }

    #[test]
    fn test_extract_embedded_json_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let fake_pdf_path = temp_dir.path().join("test.pdf");
        fs::write(&fake_pdf_path, b"fake pdf content").unwrap();

        let result = extract_embedded_json(&fake_pdf_path);
        assert!(result.is_ok());
        // Currently returns None as it's a placeholder
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_pdf_generator_with_different_verify_urls() {
        let temp_dir = TempDir::new().unwrap();
        let cert = create_test_backup_cert();

        // Test with verify URL
        let pdf_gen_with_url = PdfGenerator::new(Some("https://verify.example.com".to_string()));
        let result1 = pdf_gen_with_url.generate_backup_pdf(&cert, temp_dir.path());
        assert!(result1.is_ok());

        // Test without verify URL
        let pdf_gen_no_url = PdfGenerator::new(None);
        let result2 = pdf_gen_no_url.generate_backup_pdf(&cert, temp_dir.path());
        assert!(result2.is_ok());

        // Both should produce valid PDFs
        assert!(result1.unwrap().exists());
        assert!(result2.unwrap().exists());
    }
}