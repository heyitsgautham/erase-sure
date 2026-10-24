use crate::device::{Device, DeviceDiscovery, LinuxDeviceDiscovery};
use aes::cipher::{KeyIvInit, StreamCipher};
use aes::Aes256;
use chrono::Utc;
use ctr::Ctr64BE;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

type Aes256Ctr = Ctr64BE<Aes256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub files: HashMap<String, String>, // relative_path -> sha256
    pub created_at: String,
    pub total_files: usize,
    pub total_bytes: u64,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub manifest: BackupManifest,
    pub destination: String,
    pub encryption_method: String,
    pub verification_samples: usize,
    pub verification_passed: bool,
    pub backup_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCertificate {
    pub cert_type: String,
    pub cert_id: String,
    pub certificate_version: String,
    pub created_at: String,
    pub device: DeviceInfo,
    pub backup: BackupInfo,
    pub verification: VerificationInfo,
    pub signature: Option<SignatureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_path: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub bus_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub source_paths: Vec<String>,
    pub destination: String,
    pub encryption_method: String,
    pub compression_method: String,
    pub manifest: BackupManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub samples_verified: usize,
    pub samples_passed: usize,
    pub verification_method: String,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub alg: String,
    pub pubkey_id: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLog {
    pub timestamp: String,
    pub level: String,
    pub step_id: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub trait BackupOperations {
    fn perform_backup(
        &self,
        device: &str,
        paths: &[String],
        destination: &str,
    ) -> Result<BackupResult, Box<dyn std::error::Error>>;
}

pub struct EncryptedBackup {
    pub logger: Box<dyn BackupLogger>,
}

pub trait BackupLogger {
    fn log(&self, level: &str, step_id: &str, message: &str, data: Option<serde_json::Value>);
}

pub struct JsonLogger;

impl BackupLogger for JsonLogger {
    fn log(&self, level: &str, step_id: &str, message: &str, data: Option<serde_json::Value>) {
        let data = data.map(|d| d.to_string()).unwrap_or_default();
        match level {
            "error" => tracing::error!(step_id, data = %data, "{message}"),
            "warn" | "warning" => tracing::warn!(step_id, data = %data, "{message}"),
            "debug" => tracing::debug!(step_id, data = %data, "{message}"),
            _ => tracing::info!(step_id, data = %data, "{message}"),
        }
    }
}

impl Default for EncryptedBackup {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptedBackup {
    pub fn new() -> Self {
        Self {
            logger: Box::new(JsonLogger),
        }
    }

    fn get_default_paths() -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        vec![
            format!("{}/Documents", home),
            format!("{}/Desktop", home),
            format!("{}/Pictures", home),
        ]
    }

    fn get_device_info(&self, device_path: &str) -> Option<Device> {
        let discovery = LinuxDeviceDiscovery::new();
        match discovery.discover_devices() {
            Ok(devices) => {
                devices.into_iter().find(|device| {
                    // device.name is already in format "/dev/nvme0n1"
                    device.name == device_path ||
                    // Handle case where device_path might be just "nvme0n1" 
                    device.name == format!("/dev/{}", device_path) ||
                    // Handle case where device_path has /dev/ but device.name doesn't
                    format!("/dev/{}", device.name.trim_start_matches("/dev/")) == device_path
                })
            }
            Err(e) => {
                self.logger.log(
                    "warn",
                    "device_discovery_failed",
                    &format!("Failed to discover device info for {}: {}", device_path, e),
                    None,
                );
                None
            }
        }
    }
}

impl BackupOperations for EncryptedBackup {
    fn perform_backup(
        &self,
        device: &str,
        paths: &[String],
        destination: &str,
    ) -> Result<BackupResult, Box<dyn std::error::Error>> {
        let backup_id = Uuid::new_v4().to_string();

        self.logger.log(
            "info",
            "backup_start",
            &format!("Starting backup for device {}", device),
            None,
        );

        // Use provided paths or defaults
        let source_paths = if paths.is_empty() {
            Self::get_default_paths()
        } else {
            paths.to_vec()
        };

        // Expand destination path (handle ~ and environment variables)
        let expanded_destination = shellexpand::full(destination)
            .map_err(|e| format!("Failed to expand destination path '{}': {}", destination, e))?;
        let destination_path = Path::new(expanded_destination.as_ref());

        // Create backup directory
        let backup_dir = destination_path.join(&backup_id);
        fs::create_dir_all(&backup_dir)?;

        self.logger.log(
            "info",
            "backup_dir_created",
            &format!("Created backup directory: {:?}", backup_dir),
            None,
        );

        // Generate encryption key and IV
        let mut key = [0u8; 32];
        let mut iv = [0u8; 16];
        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);

        let mut cipher = Aes256Ctr::new(&key.into(), &iv.into());

        // Collect files
        self.logger.log(
            "info",
            "file_collection",
            "Collecting files from source paths",
            None,
        );
        let files = self.collect_files(&source_paths)?;

        // Process files
        let mut manifest_files = HashMap::new();
        let mut total_bytes = 0u64;
        let source_base = Path::new(&source_paths[0])
            .parent()
            .unwrap_or(Path::new("/"));

        for file_path in &files {
            self.logger.log(
                "info",
                "file_processing",
                &format!("Processing file: {:?}", file_path),
                None,
            );

            // Compute original hash
            let original_hash = self.compute_file_hash(file_path)?;

            // Get relative path
            let rel_path = file_path
                .strip_prefix(source_base)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            // Encrypt and write
            let dest_file = backup_dir.join(&rel_path);
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)?;
            }

            let file_bytes = self.encrypt_and_compress_file(file_path, &dest_file, &mut cipher)?;

            manifest_files.insert(rel_path, original_hash);
            total_bytes += file_bytes;
        }

        self.logger.log(
            "info",
            "encryption_complete",
            &format!(
                "Encrypted {} files, {} bytes total",
                files.len(),
                total_bytes
            ),
            None,
        );

        // Create manifest
        let mut manifest = BackupManifest {
            files: manifest_files,
            created_at: Utc::now().to_rfc3339(),
            total_files: files.len(),
            total_bytes,
            manifest_sha256: String::new(),
        };

        manifest.manifest_sha256 = self.compute_manifest_hash(&manifest);

        // Save manifest
        let manifest_path = backup_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, manifest_json)?;

        self.logger.log(
            "info",
            "manifest_created",
            "Manifest created and saved",
            None,
        );

        // Verify random files
        self.logger.log(
            "info",
            "verification_start",
            "Starting post-copy verification",
            None,
        );
        let (samples, verified) =
            self.verify_random_files(&manifest, &backup_dir, source_base, 5)?;
        let verification_passed = samples == verified;

        self.logger.log(
            if verification_passed { "info" } else { "error" },
            "verification_complete",
            &format!("Verified {}/{} samples", verified, samples),
            Some(serde_json::json!({
                "samples_total": samples,
                "samples_verified": verified,
                "passed": verification_passed
            })),
        );

        let result = BackupResult {
            manifest,
            destination: destination.to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: samples,
            verification_passed,
            backup_id: backup_id.clone(),
        };

        // Add artificial delay for small backups (< 1MB) to allow UI to properly show progress
        if total_bytes < 50_000_000 {
            self.logger.log(
                "info",
                "small_backup_delay",
                &format!(
                    "Small backup detected ({} bytes), adding UI synchronization delay",
                    total_bytes
                ),
                None,
            );
            std::thread::sleep(std::time::Duration::from_secs(3));
        }

        // Create and save certificate
        let mut certificate = self.create_backup_certificate(device, &result, &source_paths);

        // Automatically sign the certificate if signing key is available
        match self.try_sign_certificate(&mut certificate) {
            Ok(_) => {
                self.logger.log(
                    "info",
                    "certificate_signed",
                    "Certificate automatically signed",
                    None,
                );
            }
            Err(e) => {
                self.logger.log(
                    "warn",
                    "certificate_signing_failed",
                    &format!("Certificate created but not signed: {}", e),
                    None,
                );
            }
        }

        let cert_path = self.save_certificate(&certificate)?;

        self.logger.log(
            "info",
            "certificate_created",
            &format!("Certificate saved to: {:?}", cert_path),
            None,
        );
        self.logger.log(
            "info",
            "backup_complete",
            "Backup operation completed successfully",
            None,
        );

        Ok(result)
    }
}

mod certificate;
mod files;

#[cfg(test)]
mod tests;
