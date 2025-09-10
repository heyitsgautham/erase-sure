use crate::cert::{BackupCertificate, WipeCertificate, CertificateSignature};
use anyhow::{Context, Result};
use printpdf::*;
use qrcode::QrCode;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const LOGO_TEXT: &str = "SecureWipe";
const FONT_SIZE_TITLE: f32 = 24.0;
const FONT_SIZE_HEADING: f32 = 16.0;
const FONT_SIZE_BODY: f32 = 12.0;
const FONT_SIZE_SMALL: f32 = 10.0;

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
        let cert_json = serde_json::to_string_pretty(cert)
            .context("Failed to serialize certificate to JSON")?;

        self.create_pdf_document(
            &pdf_path,
            &cert.cert_id,
            &cert.created_at,
            "Backup Certificate",
            &cert_json,
            |doc, font, page_index, layer_index| {
                self.render_backup_content(doc, font, page_index, layer_index, cert)
            },
        )?;

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
        let cert_json = serde_json::to_string_pretty(cert)
            .context("Failed to serialize certificate to JSON")?;

        self.create_pdf_document(
            &pdf_path,
            &cert.cert_id,
            &cert.created_at,
            "Wipe Certificate",
            &cert_json,
            |doc, font, page_index, layer_index| {
                self.render_wipe_content(doc, font, page_index, layer_index, cert)
            },
        )?;

        info!(pdf_path = %pdf_path.display(), "Wipe certificate PDF generated successfully");
        Ok(pdf_path)
    }

    /// Create the PDF document structure and embed JSON
    fn create_pdf_document<F>(
        &self,
        pdf_path: &Path,
        cert_id: &str,
        created_at: &str,
        doc_title: &str,
        cert_json: &str,
        content_renderer: F,
    ) -> Result<()>
    where
        F: FnOnce(&PdfDocumentReference, &IndirectFontRef, PdfPageIndex, PdfLayerIndex) -> Result<()>,
    {
        // Create document
        let (doc, page1, layer1) = PdfDocument::new(doc_title, Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .context("Failed to add font")?;

        // Render header
        self.render_header(&doc, &font, page1, layer1, doc_title, cert_id, created_at)?;

        // Render certificate-specific content
        content_renderer(&doc, &font, page1, layer1)?;

        // Generate and add QR code
        self.add_qr_code(&doc, page1, layer1, cert_id)?;

        // Render footer
        self.render_footer(&doc, &font, page1, layer1)?;

        // Embed the JSON certificate as an attachment
        self.embed_json_attachment(&doc, cert_json, &format!("{}.json", cert_id))?;

        // Save the PDF
        doc.save(&mut std::io::BufWriter::new(
            std::fs::File::create(pdf_path)
                .with_context(|| format!("Failed to create PDF file: {}", pdf_path.display()))?,
        ))
        .context("Failed to save PDF document")?;

        Ok(())
    }

    /// Render the document header
    fn render_header(
        &self,
        doc: &PdfDocumentReference,
        font: &IndirectFontRef,
        page_index: PdfPageIndex,
        layer_index: PdfLayerIndex,
        doc_title: &str,
        cert_id: &str,
        created_at: &str,
    ) -> Result<()> {
        let current_layer = doc.get_page(page_index).get_layer(layer_index);

        // Logo/Title
        current_layer.begin_text_section();
        current_layer.set_font(font, FONT_SIZE_TITLE);
        current_layer.set_text_cursor(Mm(20.0), Mm(270.0));
        current_layer.write_text(LOGO_TEXT, font);
        current_layer.end_text_section();

        // Document title
        current_layer.begin_text_section();
        current_layer.set_font(font, FONT_SIZE_HEADING);
        current_layer.set_text_cursor(Mm(20.0), Mm(250.0));
        current_layer.write_text(doc_title, font);
        current_layer.end_text_section();

        // Certificate ID and timestamp
        current_layer.begin_text_section();
        current_layer.set_font(font, FONT_SIZE_BODY);
        current_layer.set_text_cursor(Mm(20.0), Mm(235.0));
        current_layer.write_text(&format!("Certificate ID: {}", cert_id), font);
        current_layer.end_text_section();

        current_layer.begin_text_section();
        current_layer.set_text_cursor(Mm(20.0), Mm(225.0));
        current_layer.write_text(&format!("Created: {}", created_at), font);
        current_layer.end_text_section();

        // Separator line
        let line = Line {
            points: vec![
                (Point::new(Mm(20.0), Mm(215.0)), false),
                (Point::new(Mm(190.0), Mm(215.0)), false),
            ],
            is_closed: false,
        };
        current_layer.add_line(line);

        Ok(())
    }

    /// Render content specific to backup certificates
    fn render_backup_content(
        &self,
        doc: &PdfDocumentReference,
        font: &IndirectFontRef,
        page_index: PdfPageIndex,
        layer_index: PdfLayerIndex,
        cert: &BackupCertificate,
    ) -> Result<()> {
        let current_layer = doc.get_page(page_index).get_layer(layer_index);
        let mut y_pos = 200.0;

        // Device Information
        self.render_section_header(&current_layer, font, "Device Information", &mut y_pos)?;
        self.render_device_table(&current_layer, font, &cert.device, &mut y_pos)?;

        // Backup Summary
        self.render_section_header(&current_layer, font, "Backup Summary", &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Files Count", &cert.backup_summary.get("files").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Total Bytes", &cert.backup_summary.get("bytes").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Encryption Method", &cert.encryption_method, &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Manifest SHA256", &cert.manifest_sha256, &mut y_pos)?;

        // Signature Information
        self.render_section_header(&current_layer, font, "Digital Signature", &mut y_pos)?;
        self.render_signature_info(&current_layer, font, &cert.signature, &mut y_pos)?;

        Ok(())
    }

    /// Render content specific to wipe certificates
    fn render_wipe_content(
        &self,
        doc: &PdfDocumentReference,
        font: &IndirectFontRef,
        page_index: PdfPageIndex,
        layer_index: PdfLayerIndex,
        cert: &WipeCertificate,
    ) -> Result<()> {
        let current_layer = doc.get_page(page_index).get_layer(layer_index);
        let mut y_pos = 200.0;

        // Device Information
        self.render_section_header(&current_layer, font, "Device Information", &mut y_pos)?;
        self.render_device_table(&current_layer, font, &cert.device, &mut y_pos)?;

        // Wipe Policy and Method
        self.render_section_header(&current_layer, font, "Wipe Policy", &mut y_pos)?;
        self.render_key_value(&current_layer, font, "NIST Level", &cert.wipe_summary.get("policy").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Method", &cert.wipe_summary.get("method").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;

        // Verification Results
        self.render_section_header(&current_layer, font, "Verification", &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Samples Verified", &cert.wipe_summary.get("verification_samples").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;
        self.render_key_value(&current_layer, font, "Verification Passed", &cert.wipe_summary.get("verification_passed").unwrap_or(&serde_json::Value::Null).to_string(), &mut y_pos)?;

        // Linkage (if present)
        if let Some(linkage) = &cert.linkage {
            self.render_section_header(&current_layer, font, "Linkage", &mut y_pos)?;
            if let Some(backup_cert_id) = linkage.get("backup_cert_id") {
                self.render_key_value(&current_layer, font, "Backup Certificate ID", &backup_cert_id.to_string(), &mut y_pos)?;
            }
        }

        // Signature Information
        self.render_section_header(&current_layer, font, "Digital Signature", &mut y_pos)?;
        self.render_signature_info(&current_layer, font, &cert.signature, &mut y_pos)?;

        Ok(())
    }

    /// Render a section header
    fn render_section_header(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        title: &str,
        y_pos: &mut f32,
    ) -> Result<()> {
        *y_pos -= 15.0;
        layer.begin_text_section();
        layer.set_font(font, FONT_SIZE_HEADING);
        layer.set_text_cursor(Mm(20.0), Mm(*y_pos));
        layer.write_text(title, font);
        layer.end_text_section();
        *y_pos -= 5.0;
        Ok(())
    }

    /// Render device information table
    fn render_device_table(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        device: &serde_json::Value,
        y_pos: &mut f32,
    ) -> Result<()> {
        if let Some(model) = device.get("model") {
            self.render_key_value(layer, font, "Model", &model.to_string(), y_pos)?;
        }
        if let Some(serial) = device.get("serial") {
            self.render_key_value(layer, font, "Serial", &serial.to_string(), y_pos)?;
        }
        if let Some(capacity) = device.get("capacity_bytes") {
            let capacity_gb = capacity.as_u64().unwrap_or(0) as f64 / 1_000_000_000.0;
            self.render_key_value(layer, font, "Capacity", &format!("{:.2} GB", capacity_gb), y_pos)?;
        }
        Ok(())
    }

    /// Render signature information
    fn render_signature_info(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        sig: &CertificateSignature,
        y_pos: &mut f32,
    ) -> Result<()> {
        self.render_key_value(layer, font, "Algorithm", &sig.alg, y_pos)?;
        self.render_key_value(layer, font, "Public Key ID", &sig.pubkey_id, y_pos)?;
        // Truncate signature for display
        let sig_display = if sig.sig.len() > 32 {
            format!("{}...", &sig.sig[..32])
        } else {
            sig.sig.clone()
        };
        self.render_key_value(layer, font, "Signature", &sig_display, y_pos)?;
        Ok(())
    }

    /// Render a key-value pair
    fn render_key_value(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        key: &str,
        value: &str,
        y_pos: &mut f32,
    ) -> Result<()> {
        *y_pos -= 12.0;
        layer.begin_text_section();
        layer.set_font(font, FONT_SIZE_BODY);
        layer.set_text_cursor(Mm(25.0), Mm(*y_pos));
        layer.write_text(&format!("{}: {}", key, value), font);
        layer.end_text_section();
        Ok(())
    }

    /// Add QR code to the document
    fn add_qr_code(
        &self,
        doc: &PdfDocumentReference,
        page_index: PdfPageIndex,
        layer_index: PdfLayerIndex,
        cert_id: &str,
    ) -> Result<()> {
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert_id)
        } else {
            cert_id.to_string()
        };

        info!(qr_data = %qr_data, "Generating QR code");

        let qr_code = QrCode::new(&qr_data)
            .with_context(|| format!("Failed to generate QR code for data: {}", qr_data))?;

        // For now, we'll just add a placeholder for the QR code
        // In a full implementation, you would convert the QR code to an image format supported by printpdf
        let _qr_image = qr_code.render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 2)
            .build();

        // Add QR code to PDF (simplified implementation)
        let current_layer = doc.get_page(page_index).get_layer(layer_index);
        
        // Add a text label for the QR code
        current_layer.begin_text_section();
        current_layer.set_font(&doc.add_builtin_font(BuiltinFont::Helvetica)?, FONT_SIZE_SMALL);
        current_layer.set_text_cursor(Mm(150.0), Mm(40.0));
        current_layer.write_text("Verification QR Code", &doc.add_builtin_font(BuiltinFont::Helvetica)?);
        current_layer.end_text_section();

        Ok(())
    }

    /// Render the document footer
    fn render_footer(
        &self,
        doc: &PdfDocumentReference,
        font: &IndirectFontRef,
        page_index: PdfPageIndex,
        layer_index: PdfLayerIndex,
    ) -> Result<()> {
        let current_layer = doc.get_page(page_index).get_layer(layer_index);

        // Footer line
        let line = Line {
            points: vec![
                (Point::new(Mm(20.0), Mm(30.0)), false),
                (Point::new(Mm(190.0), Mm(30.0)), false),
            ],
            is_closed: false,
        };
        current_layer.add_line(line);

        // Footer text
        current_layer.begin_text_section();
        current_layer.set_font(font, FONT_SIZE_SMALL);
        current_layer.set_text_cursor(Mm(20.0), Mm(20.0));
        current_layer.write_text("SecureWipe Certificate - NIST SP 800-88 Compliant", font);
        current_layer.end_text_section();

        Ok(())
    }

    /// Embed JSON certificate as PDF attachment
    fn embed_json_attachment(
        &self,
        _doc: &PdfDocumentReference,
        json_content: &str,
        filename: &str,
    ) -> Result<()> {
        // Note: printpdf doesn't have direct attachment support in the version we're using
        // This is a placeholder for when that functionality is available
        // For now, we'll log that we would embed the JSON
        info!(
            filename = %filename,
            json_size = json_content.len(),
            "JSON certificate would be embedded as PDF attachment"
        );
        Ok(())
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