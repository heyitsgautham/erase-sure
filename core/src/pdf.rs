use crate::cert::{BackupCertificate, WipeCertificate};
use anyhow::{Context, Result};
use printpdf::*;
use std::fs;
use std::path::{Path, PathBuf};
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
        
        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new("SecureWipe Backup Certificate", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add title
        current_layer.begin_text_section();
        current_layer.set_font(&font, 20.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(270.0));
        current_layer.write_text("SecureWipe Backup Certificate", &font);
        current_layer.end_text_section();

        // Add subtitle
        current_layer.begin_text_section();
        current_layer.set_font(&font, 12.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(250.0));
        current_layer.write_text("NIST SP 800-88 Compliant Data Backup Operation", &font);
        current_layer.end_text_section();

        let mut y_position = 230.0;

        // Certificate Information
        self.add_section(&current_layer, &font, "Certificate Information", &mut y_position);
        self.add_field(&current_layer, &font, "Certificate ID", &cert.cert_id, &mut y_position);
        self.add_field(&current_layer, &font, "Type", &cert.cert_type.to_uppercase(), &mut y_position);
        self.add_field(&current_layer, &font, "Version", &cert.certificate_version, &mut y_position);
        self.add_field(&current_layer, &font, "Created", &cert.created_at, &mut y_position);
        self.add_field(&current_layer, &font, "Result", &cert.result, &mut y_position);

        y_position -= 10.0;

        // Device Information
        self.add_section(&current_layer, &font, "Device Information", &mut y_position);
        if let Some(model) = cert.device.get("model").and_then(|v| v.as_str()) {
            self.add_field(&current_layer, &font, "Model", model, &mut y_position);
        }
        if let Some(serial) = cert.device.get("serial").and_then(|v| v.as_str()) {
            self.add_field(&current_layer, &font, "Serial Number", serial, &mut y_position);
        }
        if let Some(capacity) = cert.device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            let capacity_str = self.format_bytes(capacity);
            self.add_field(&current_layer, &font, "Capacity", &capacity_str, &mut y_position);
        }

        y_position -= 10.0;

        // Backup Summary
        self.add_section(&current_layer, &font, "Backup Summary", &mut y_position);
        if let Some(count) = cert.files_summary.get("count") {
            self.add_field(&current_layer, &font, "Files Count", &count.to_string(), &mut y_position);
        }
        if let Some(bytes) = cert.files_summary.get("personal_bytes").and_then(|v| v.as_u64()) {
            let bytes_str = self.format_bytes(bytes);
            self.add_field(&current_layer, &font, "Personal Bytes", &bytes_str, &mut y_position);
        }
        if let Some(alg) = cert.crypto.get("alg").and_then(|v| v.as_str()) {
            self.add_field(&current_layer, &font, "Encryption", alg, &mut y_position);
        }
        if let Some(hash) = cert.crypto.get("manifest_sha256").and_then(|v| v.as_str()) {
            let hash_display = self.format_hash(hash);
            self.add_field(&current_layer, &font, "Manifest SHA256", &hash_display, &mut y_position);
        }

        y_position -= 10.0;

        // Digital Signature
        self.add_section(&current_layer, &font, "Digital Signature", &mut y_position);
        match &cert.signature {
            Some(signature) => {
                self.add_field(&current_layer, &font, "Algorithm", &signature.alg, &mut y_position);
                self.add_field(&current_layer, &font, "Public Key ID", &signature.pubkey_id, &mut y_position);
                let sig_display = self.format_hash(&signature.sig);
                self.add_field(&current_layer, &font, "Signature", &sig_display, &mut y_position);
            }
            None => {
                self.add_field(&current_layer, &font, "Status", "Unsigned Certificate", &mut y_position);
                self.add_field(&current_layer, &font, "Note", "This certificate has not been digitally signed", &mut y_position);
            }
        }

        y_position -= 10.0;

        // QR Code info
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert.cert_id)
        } else {
            format!("cert_id:{}", cert.cert_id)
        };
        self.add_section(&current_layer, &font, "Verification QR Code", &mut y_position);
        self.add_field(&current_layer, &font, "QR Data", &qr_data, &mut y_position);

        // Footer
        current_layer.begin_text_section();
        current_layer.set_font(&font, 10.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(20.0));
        current_layer.write_text("Generated by SecureWipe - NIST SP 800-88 Compliant", &font);
        current_layer.end_text_section();

        // Save PDF
        doc.save(&mut std::io::BufWriter::new(
            std::fs::File::create(&pdf_path)
                .with_context(|| format!("Failed to create PDF file: {}", pdf_path.display()))?,
        ))
        .context("Failed to save PDF document")?;

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
        
        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new("SecureWipe Data Sanitization Certificate", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add title
        current_layer.begin_text_section();
        current_layer.set_font(&font, 20.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(270.0));
        current_layer.write_text("SecureWipe Data Sanitization Certificate", &font);
        current_layer.end_text_section();

        // Add subtitle
        current_layer.begin_text_section();
        current_layer.set_font(&font, 12.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(250.0));
        current_layer.write_text("NIST SP 800-88 Compliant Wipe Operation", &font);
        current_layer.end_text_section();

        let mut y_position = 230.0;

        // Certificate Information
        self.add_section(&current_layer, &font, "Certificate Information", &mut y_position);
        self.add_field(&current_layer, &font, "Certificate ID", &cert.cert_id, &mut y_position);
        self.add_field(&current_layer, &font, "Type", &cert.cert_type.to_uppercase(), &mut y_position);
        self.add_field(&current_layer, &font, "Created", &cert.created_at, &mut y_position);
        
        let result = cert.wipe_summary.get("verification_passed")
            .and_then(|v| v.as_bool())
            .map(|passed| if passed { "PASS" } else { "FAIL" })
            .unwrap_or("UNKNOWN");
        self.add_field(&current_layer, &font, "Result", result, &mut y_position);

        y_position -= 10.0;

        // Device Information
        self.add_section(&current_layer, &font, "Device Information", &mut y_position);
        if let Some(model) = cert.device.get("model").and_then(|v| v.as_str()) {
            self.add_field(&current_layer, &font, "Model", model, &mut y_position);
        }
        if let Some(serial) = cert.device.get("serial").and_then(|v| v.as_str()) {
            self.add_field(&current_layer, &font, "Serial Number", serial, &mut y_position);
        }
        if let Some(capacity) = cert.device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            let capacity_str = self.format_bytes(capacity);
            self.add_field(&current_layer, &font, "Capacity", &capacity_str, &mut y_position);
        }

        y_position -= 10.0;

        // Sanitization Policy
        self.add_section(&current_layer, &font, "Sanitization Policy", &mut y_position);
        if let Some(policy) = cert.wipe_summary.get("policy") {
            self.add_field(&current_layer, &font, "NIST Level", &policy.to_string(), &mut y_position);
        }
        if let Some(method) = cert.wipe_summary.get("method") {
            self.add_field(&current_layer, &font, "Method", &method.to_string(), &mut y_position);
        }

        y_position -= 10.0;

        // Verification Results
        self.add_section(&current_layer, &font, "Verification Results", &mut y_position);
        if let Some(samples) = cert.wipe_summary.get("verification_samples") {
            self.add_field(&current_layer, &font, "Samples Verified", &samples.to_string(), &mut y_position);
        }
        if let Some(passed) = cert.wipe_summary.get("verification_passed") {
            self.add_field(&current_layer, &font, "Verification Passed", &passed.to_string(), &mut y_position);
        }

        // Linkage (if present)
        if let Some(linkage) = &cert.linkage {
            if let Some(backup_cert_id) = linkage.get("backup_cert_id") {
                y_position -= 10.0;
                self.add_section(&current_layer, &font, "Evidence & Linkage", &mut y_position);
                self.add_field(&current_layer, &font, "Backup Certificate ID", &backup_cert_id.to_string(), &mut y_position);
            }
        }

        y_position -= 10.0;

        // Digital Signature
        self.add_section(&current_layer, &font, "Digital Signature & Verification", &mut y_position);
        match &cert.signature {
            Some(signature) => {
                self.add_field(&current_layer, &font, "Algorithm", &signature.alg, &mut y_position);
                self.add_field(&current_layer, &font, "Public Key ID", &signature.pubkey_id, &mut y_position);
                let sig_display = self.format_hash(&signature.sig);
                self.add_field(&current_layer, &font, "Signature", &sig_display, &mut y_position);
            }
            None => {
                self.add_field(&current_layer, &font, "Status", "Unsigned Certificate", &mut y_position);
                self.add_field(&current_layer, &font, "Note", "This certificate has not been digitally signed", &mut y_position);
            }
        }

        y_position -= 10.0;

        // QR Code info
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert.cert_id)
        } else {
            format!("cert_id:{}", cert.cert_id)
        };
        self.add_section(&current_layer, &font, "Verification QR Code", &mut y_position);
        self.add_field(&current_layer, &font, "QR Data", &qr_data, &mut y_position);

        // Footer
        current_layer.begin_text_section();
        current_layer.set_font(&font, 10.0);
        current_layer.set_text_cursor(Mm(20.0), Mm(20.0));
        current_layer.write_text("Generated by SecureWipe - NIST SP 800-88 Compliant", &font);
        current_layer.end_text_section();

        // Save PDF
        doc.save(&mut std::io::BufWriter::new(
            std::fs::File::create(&pdf_path)
                .with_context(|| format!("Failed to create PDF file: {}", pdf_path.display()))?,
        ))
        .context("Failed to save PDF document")?;

        info!(pdf_path = %pdf_path.display(), "Wipe certificate PDF generated successfully");
        Ok(pdf_path)
    }

    /// Add a section header
    fn add_section(&self, layer: &PdfLayerReference, font: &IndirectFontRef, title: &str, y_pos: &mut f32) {
        *y_pos -= 15.0;
        layer.begin_text_section();
        layer.set_font(font, 16.0);
        layer.set_text_cursor(Mm(20.0), Mm(*y_pos));
        layer.write_text(title, font);
        layer.end_text_section();
        *y_pos -= 5.0;
    }

    /// Add a field with label and value
    fn add_field(&self, layer: &PdfLayerReference, font: &IndirectFontRef, label: &str, value: &str, y_pos: &mut f32) {
        *y_pos -= 12.0;
        layer.begin_text_section();
        layer.set_font(font, 10.0);
        layer.set_text_cursor(Mm(25.0), Mm(*y_pos));
        layer.write_text(&format!("{}: {}", label, value), font);
        layer.end_text_section();
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
            cert_id: "test_wipe_456".to_string(),
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
