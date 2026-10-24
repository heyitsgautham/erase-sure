//! Building, signing and saving the backup certificate.

use super::*;

impl EncryptedBackup {
    pub(super) fn create_backup_certificate(
        &self,
        device: &str,
        result: &BackupResult,
        source_paths: &[String],
    ) -> serde_json::Value {
        // Get real device information
        let device_info = self.get_device_info(device);

        // Determine destination type based on path
        let dest_type = if result.destination.contains("/media/")
            || result.destination.contains("/mnt/")
            || result.destination.to_lowercase().contains("usb")
        {
            "usb"
        } else if result.destination.contains("://")
            || result.destination.to_lowercase().contains("nas")
        {
            "nas"
        } else {
            "other"
        };

        // Create a schema-compliant certificate structure
        serde_json::json!({
            "cert_type": "backup",
            "cert_id": result.backup_id,
            "certificate_version": "v1.0.0",
            "created_at": Utc::now().to_rfc3339(),
            "issuer": {
                "organization": "SecureWipe (SIH)",
                "tool_name": "securewipe",
                "tool_version": "v1.0.0",
                "country": "IN"
            },
            "device": {
                "model": device_info.as_ref().and_then(|d| d.model.as_ref()).unwrap_or(&"Unknown".to_string()).clone(),
                "serial": device_info.as_ref().and_then(|d| d.serial.as_ref()).unwrap_or(&"N/A".to_string()).clone(),
                "bus": device_info.as_ref().and_then(|d| d.bus.as_ref()).unwrap_or(&"UNKNOWN".to_string()).clone(),
                "capacity_bytes": device_info.as_ref().map(|d| d.capacity_bytes).unwrap_or(0),
                "path": device
            },
            "files_summary": {
                "count": result.manifest.total_files,
                "personal_bytes": result.manifest.total_bytes,
                "included_paths": source_paths
            },
            "destination": {
                "type": dest_type,
                "path": result.destination
            },
            "crypto": {
                "alg": result.encryption_method,
                "manifest_sha256": result.manifest.manifest_sha256,
                "key_management": "ephemeral_session_key"
            },
            "verification": {
                "strategy": "sampled_files",
                "failures": if result.verification_passed { 0 } else { 1 }
            },
            "policy": {
                "name": "NIST SP 800-88 Rev.1",
                "version": "2023.12"
            },
            "result": if result.verification_passed { "PASS" } else { "FAIL" },
            "environment": {
                "operator": "Automated",
                "os_kernel": std::env::consts::OS,
                "tool_version": "v1.0.0"
            },
            "exceptions": {
                "text": "None"
            },
            "metadata": {
                "qr_payload": {
                    "cert_id": result.backup_id,
                    "issued_at": Utc::now().to_rfc3339(),
                    "device_model": device_info.as_ref().and_then(|d| d.model.as_ref()).unwrap_or(&"Unknown".to_string()).clone(),
                    "result": if result.verification_passed { "PASS" } else { "FAIL" },
                    "nist_level": "SP 800-88 Rev.1",
                    "method": "AES-256-CTR",
                    "verify_url": "https://verify.securewipe.sih/certificate"
                }
            },
            "verify_url": "https://verify.securewipe.sih/certificate"
        })
    }

    pub(super) fn save_certificate(
        &self,
        cert: &serde_json::Value,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let cert_dir = Path::new(&home).join("SecureWipe").join("certificates");
        fs::create_dir_all(&cert_dir)?;

        let cert_id = cert
            .get("cert_id")
            .and_then(|v| v.as_str())
            .ok_or("Certificate ID not found")?;

        let cert_file = cert_dir.join(format!("{}.json", cert_id));
        let cert_json = serde_json::to_string_pretty(cert)?;
        fs::write(&cert_file, cert_json)?;

        Ok(cert_file)
    }

    /// Attempt to automatically sign a certificate using available private key
    pub(super) fn populate_metadata(
        &self,
        cert: &mut serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(cert_obj) = cert.as_object_mut() {
            // Get certificate details for QR payload (collect strings first to avoid borrowing issues)
            let cert_id = cert_obj
                .get("cert_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let issued_at = cert_obj
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let device_model = cert_obj
                .get("device")
                .and_then(|d| d.get("model"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Create QR payload object as per schema
            let qr_payload = serde_json::json!({
                "cert_id": cert_id,
                "issued_at": issued_at,
                "device_model": device_model,
                "result": "PASS"
            });

            // Update metadata with proper QR payload object
            if let Some(metadata) = cert_obj.get_mut("metadata") {
                if let Some(metadata_obj) = metadata.as_object_mut() {
                    metadata_obj.insert("qr_payload".to_string(), qr_payload);
                }
            }

            self.logger.log(
                "info",
                "metadata_populated",
                &format!("Populated metadata with QR payload for cert: {}", cert_id),
                None,
            );
        }
        Ok(())
    }

    pub(super) fn try_sign_certificate(
        &self,
        cert: &mut serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::signer::{load_private_key, sign_certificate};
        use std::path::PathBuf;

        // Try multiple locations for the private key
        let key_paths = vec![
            // 1. Environment variable (if set)
            std::env::var("SECUREWIPE_SIGN_KEY_PATH")
                .ok()
                .map(PathBuf::from),
            // 2. Project-relative path (for development)
            Some(PathBuf::from("keys/dev_private.pem")),
            // 3. Absolute path to development key
            Some(PathBuf::from(
                "/home/user/projects/erase-sure/keys/dev_private.pem",
            )),
            // 4. User's SecureWipe directory
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join("SecureWipe/keys/private.pem")),
        ];

        for key_path in key_paths.into_iter().flatten() {
            if key_path.exists() {
                match load_private_key(Some(key_path.clone())) {
                    Ok(signing_key) => {
                        // Sign the certificate (force=true to overwrite null signature)
                        match sign_certificate(cert, &signing_key, true) {
                            Ok(_) => {
                                // Populate metadata after successful signing
                                self.populate_metadata(cert)?;

                                self.logger.log(
                                    "info",
                                    "signing_success",
                                    &format!(
                                        "Certificate signed using key: {}",
                                        key_path.display()
                                    ),
                                    None,
                                );
                                return Ok(());
                            }
                            Err(e) => {
                                self.logger.log(
                                    "error",
                                    "signing_failed",
                                    &format!(
                                        "Failed to sign certificate with key {}: {}",
                                        key_path.display(),
                                        e
                                    ),
                                    None,
                                );
                                return Err(e.into());
                            }
                        }
                    }
                    Err(e) => {
                        self.logger.log(
                            "debug",
                            "signing_key_failed",
                            &format!("Failed to use key {}: {}", key_path.display(), e),
                            None,
                        );
                        continue;
                    }
                }
            }
        }

        self.logger.log(
            "error",
            "no_signing_key",
            "No valid signing key found in any expected location",
            None,
        );
        Err("No valid signing key found in any expected location".into())
    }
}
