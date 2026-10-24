//! PDF layout for wipe certificates.

use super::*;

impl PdfGenerator {
    /// Generate PDF certificate from wipe certificate JSON
    pub fn generate_wipe_pdf(&self, cert: &WipeCertificate, output_dir: &Path) -> Result<PathBuf> {
        info!(
            cert_id = %cert.cert_id,
            cert_type = %cert.cert_type,
            "Generating wipe certificate PDF"
        );

        let pdf_path = output_dir.join(format!("{}.pdf", cert.cert_id));

        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            "SecureWipe Data Sanitization Certificate",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );
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
        self.add_section(
            &current_layer,
            &font,
            "Certificate Information",
            &mut y_position,
        );
        self.add_field(
            &current_layer,
            &font,
            "Certificate ID",
            &cert.cert_id,
            &mut y_position,
        );
        self.add_field(
            &current_layer,
            &font,
            "Type",
            &cert.cert_type.to_uppercase(),
            &mut y_position,
        );
        self.add_field(
            &current_layer,
            &font,
            "Created",
            &cert.created_at,
            &mut y_position,
        );

        let result = cert
            .wipe_summary
            .get("verification_passed")
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
            self.add_field(
                &current_layer,
                &font,
                "Serial Number",
                serial,
                &mut y_position,
            );
        }
        if let Some(capacity) = cert.device.get("capacity_bytes").and_then(|v| v.as_u64()) {
            let capacity_str = self.format_bytes(capacity);
            self.add_field(
                &current_layer,
                &font,
                "Capacity",
                &capacity_str,
                &mut y_position,
            );
        }

        y_position -= 10.0;

        // Sanitization Policy
        self.add_section(
            &current_layer,
            &font,
            "Sanitization Policy",
            &mut y_position,
        );
        if let Some(policy) = cert.wipe_summary.get("policy") {
            self.add_field(
                &current_layer,
                &font,
                "NIST Level",
                &policy.to_string(),
                &mut y_position,
            );
        }
        if let Some(method) = cert.wipe_summary.get("method") {
            self.add_field(
                &current_layer,
                &font,
                "Method",
                &method.to_string(),
                &mut y_position,
            );
        }

        y_position -= 10.0;

        // Verification Results
        self.add_section(
            &current_layer,
            &font,
            "Verification Results",
            &mut y_position,
        );
        if let Some(samples) = cert.wipe_summary.get("verification_samples") {
            self.add_field(
                &current_layer,
                &font,
                "Samples Verified",
                &samples.to_string(),
                &mut y_position,
            );
        }
        if let Some(passed) = cert.wipe_summary.get("verification_passed") {
            self.add_field(
                &current_layer,
                &font,
                "Verification Passed",
                &passed.to_string(),
                &mut y_position,
            );
        }

        // Linkage (if present)
        if let Some(linkage) = &cert.linkage {
            if let Some(backup_cert_id) = linkage.get("backup_cert_id") {
                y_position -= 10.0;
                self.add_section(&current_layer, &font, "Evidence & Linkage", &mut y_position);
                self.add_field(
                    &current_layer,
                    &font,
                    "Backup Certificate ID",
                    &backup_cert_id.to_string(),
                    &mut y_position,
                );
            }
        }

        y_position -= 10.0;

        // Digital Signature
        self.add_section(
            &current_layer,
            &font,
            "Digital Signature & Verification",
            &mut y_position,
        );
        match &cert.signature {
            Some(signature) => {
                self.add_field(
                    &current_layer,
                    &font,
                    "Algorithm",
                    &signature.alg,
                    &mut y_position,
                );
                self.add_field(
                    &current_layer,
                    &font,
                    "Public Key ID",
                    &signature.pubkey_id,
                    &mut y_position,
                );
                let sig_display = self.format_hash(&signature.sig);
                self.add_field(
                    &current_layer,
                    &font,
                    "Signature",
                    &sig_display,
                    &mut y_position,
                );
            }
            None => {
                self.add_field(
                    &current_layer,
                    &font,
                    "Status",
                    "Unsigned Certificate",
                    &mut y_position,
                );
                self.add_field(
                    &current_layer,
                    &font,
                    "Note",
                    "This certificate has not been digitally signed",
                    &mut y_position,
                );
            }
        }

        y_position -= 10.0;

        // QR Code info
        let qr_data = if let Some(base_url) = &self.verify_base_url {
            format!("{}/verify/{}", base_url, cert.cert_id)
        } else {
            format!("cert_id:{}", cert.cert_id)
        };
        self.add_section(
            &current_layer,
            &font,
            "Verification QR Code",
            &mut y_position,
        );
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
}
