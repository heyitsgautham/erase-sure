use crate::device::{DeviceDiscovery, LinuxDeviceDiscovery};
use serde::{Deserialize, Serialize};

fn get_device_info(device: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Use proper device discovery to get detailed information including bus type
    let discovery = LinuxDeviceDiscovery::new();
    match discovery.discover_devices() {
        Ok(devices) => {
            if let Some(device_info) = devices.into_iter().find(|d| {
                // device.name is already in format "/dev/nvme0n1"
                d.name == device ||
                // Handle case where device might be just "nvme0n1"
                d.name == format!("/dev/{}", device) ||
                // Handle case where device has /dev/ but d.name doesn't
                format!("/dev/{}", d.name.trim_start_matches("/dev/")) == device
            }) {
                let bus = device_info.bus.unwrap_or_else(|| "UNKNOWN".to_string());
                return Ok(serde_json::json!({
                    "model": device_info.model.unwrap_or_else(|| "Unknown".to_string()),
                    "serial": device_info.serial.unwrap_or_else(|| "Unknown".to_string()),
                    "bus": bus.clone(),
                    "capacity_bytes": device_info.capacity_bytes,
                    "path": device_info.name,
                    "firmware": "N/A", // Not available from device discovery yet
                    "logical_block_size": 512, // Default, could be enhanced
                    "total_lbas": device_info.capacity_bytes / 512, // Approximate
                    "protocol_path": bus
                }));
            }
        }
        Err(e) => {
            eprintln!(
                "Device discovery failed: {}, falling back to basic lsblk",
                e
            );
        }
    }

    // Fallback to basic lsblk info if device discovery fails
    let lsblk_output = std::process::Command::new("lsblk")
        .arg("-J")
        .arg("-o")
        .arg("NAME,SIZE,MODEL,SERIAL,TYPE,MOUNTPOINT")
        .arg(device)
        .output();

    // A missing lsblk binary (non-Linux hosts) falls through to the placeholder below.
    if let Some(lsblk_output) = lsblk_output.ok().filter(|o| o.status.success()) {
        let lsblk_text = String::from_utf8_lossy(&lsblk_output.stdout);
        if let Ok(lsblk_json) = serde_json::from_str::<serde_json::Value>(&lsblk_text) {
            if let Some(blockdevices) = lsblk_json["blockdevices"].as_array() {
                if let Some(device_info) = blockdevices.first() {
                    return Ok(serde_json::json!({
                        "model": device_info["model"].as_str().unwrap_or("Unknown"),
                        "serial": device_info["serial"].as_str().unwrap_or("Unknown"),
                        "bus": "UNKNOWN", // lsblk doesn't provide bus type
                        "capacity_bytes": 0, // Would need parsing of size string
                        "path": device,
                        "firmware": "N/A",
                        "logical_block_size": 512,
                        "total_lbas": 0,
                        "protocol_path": "UNKNOWN"
                    }));
                }
            }
        }
    }

    // Final fallback
    Ok(serde_json::json!({
        "model": "Unknown",
        "serial": "Unknown",
        "bus": "UNKNOWN",
        "capacity_bytes": 0,
        "path": device,
        "firmware": "N/A",
        "logical_block_size": 512,
        "total_lbas": 0,
        "protocol_path": "UNKNOWN"
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignature {
    pub alg: String,       // "Ed25519"
    pub pubkey_id: String, // "sih_root_v1"
    pub sig: String,       // Base64 signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCertificate {
    pub cert_id: String,
    pub cert_type: String, // "backup"
    pub certificate_version: String,
    pub created_at: String,
    pub issuer: serde_json::Value,
    pub device: serde_json::Value,
    pub files_summary: serde_json::Value,
    pub destination: serde_json::Value,
    pub crypto: serde_json::Value,
    pub verification: serde_json::Value,
    pub policy: serde_json::Value,
    pub result: String,
    pub environment: serde_json::Value,
    pub exceptions: serde_json::Value,
    pub signature: Option<CertificateSignature>,
    pub metadata: serde_json::Value,
    pub verify_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeCertificate {
    pub cert_id: String,
    pub cert_type: String, // "wipe"
    pub certificate_version: String,
    pub created_at: String,
    pub device: serde_json::Value,
    pub wipe_summary: serde_json::Value,
    pub linkage: Option<serde_json::Value>,
    pub signature: Option<CertificateSignature>,
}

#[allow(dead_code)] // MVP: Implementation pending
pub trait CertificateOperations {
    fn create_backup_certificate(
        &self,
        backup_result: &crate::backup::BackupResult,
    ) -> Result<BackupCertificate, Box<dyn std::error::Error>>;

    fn create_wipe_certificate(
        &self,
        wipe_result: &crate::wipe::WipeResult,
        backup_cert_id: Option<&str>,
    ) -> Result<WipeCertificate, Box<dyn std::error::Error>>;

    fn export_to_pdf(&self, cert_id: &str) -> Result<String, Box<dyn std::error::Error>>;

    fn generate_backup_certificate_pdf(
        &self,
        cert: &BackupCertificate,
        verify_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>>;

    fn generate_wipe_certificate_pdf(
        &self,
        cert: &WipeCertificate,
        verify_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

#[allow(dead_code)] // MVP: Implementation pending
pub struct Ed25519CertificateManager;

impl CertificateOperations for Ed25519CertificateManager {
    fn create_backup_certificate(
        &self,
        _backup_result: &crate::backup::BackupResult,
    ) -> Result<BackupCertificate, Box<dyn std::error::Error>> {
        // Stub implementation - will create actual signed certificates
        Ok(BackupCertificate {
            cert_id: "stub_backup_cert_id".to_string(),
            cert_type: "backup".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            issuer: serde_json::json!({
                "organization": "SecureWipe (SIH)",
                "tool_name": "securewipe",
                "tool_version": "v2.1.0",
                "country": "IN"
            }),
            device: serde_json::json!({}),
            files_summary: serde_json::json!({"count": 0, "personal_bytes": 0}),
            destination: serde_json::json!({"type": "other", "path": "/backup"}),
            crypto: serde_json::json!({"alg": "AES-256-CTR", "manifest_sha256": "stub_hash"}),
            verification: serde_json::json!({"strategy": "sampled_files", "samples": 0}),
            policy: serde_json::json!({"name": "NIST SP 800-88 Rev.1", "version": "2023.12"}),
            result: "PASS".to_string(),
            environment: serde_json::json!({"operator": "test", "os_kernel": "test"}),
            exceptions: serde_json::json!({"text": "None"}),
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "stub_signature".to_string(),
            }),
            metadata: serde_json::json!({}),
            verify_url: "http://localhost:8000/verify".to_string(),
        })
    }

    fn create_wipe_certificate(
        &self,
        wipe_result: &crate::wipe::WipeResult,
        backup_cert_id: Option<&str>,
    ) -> Result<WipeCertificate, Box<dyn std::error::Error>> {
        let cert_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        // Get device information
        let device_info = get_device_info(&wipe_result.device)?;

        // Create comprehensive wipe summary
        let wipe_summary = serde_json::json!({
            "policy": match wipe_result.policy {
                crate::wipe::WipePolicy::Clear => "CLEAR",
                crate::wipe::WipePolicy::Purge => "PURGE"
            },
            "method": wipe_result.method,
            "commands_executed": wipe_result.commands.len(),
            "verification_samples": wipe_result.verification_samples,
            "verification_passed": wipe_result.verification_passed,
            "fallback_reason": wipe_result.fallback_reason,
            "execution_log": wipe_result.commands.iter().map(|cmd| serde_json::json!({
                "command": cmd.command,
                "exit_code": cmd.exit_code,
                "elapsed_ms": cmd.elapsed_ms,
                "success": cmd.exit_code == 0
            })).collect::<Vec<_>>(),
            "total_execution_time_ms": wipe_result.commands.iter().map(|c| c.elapsed_ms).sum::<u64>()
        });

        // Create linkage if backup cert provided
        let linkage = backup_cert_id.map(|id| {
            serde_json::json!({
                "backup_cert_id": id,
                "chain_type": "backup_then_wipe",
                "created_at": created_at
            })
        });

        Ok(WipeCertificate {
            cert_id: cert_id.clone(),
            cert_type: "wipe".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at,
            device: device_info,
            wipe_summary,
            linkage,
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: format!("unsigned_wipe_{}", cert_id), // Will be replaced with real signature
            }),
        })
    }

    fn export_to_pdf(&self, _cert_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Stub implementation - will generate styled PDF
        Ok("stub_pdf_path.pdf".to_string())
    }

    fn generate_backup_certificate_pdf(
        &self,
        cert: &BackupCertificate,
        verify_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Stub implementation for MVP
        let _verify_url = verify_url;
        let cert_filename = format!("{}.pdf", cert.cert_id);
        Ok(format!("~/SecureWipe/certificates/{}", cert_filename))
    }

    fn generate_wipe_certificate_pdf(
        &self,
        cert: &WipeCertificate,
        verify_url: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Stub implementation for MVP
        let _verify_url = verify_url;
        let cert_filename = format!("{}.pdf", cert.cert_id);
        Ok(format!("~/SecureWipe/certificates/{}", cert_filename))
    }
}

mod wipe_json;
pub use wipe_json::build_wipe_certificate_json;

#[cfg(test)]
mod tests;
