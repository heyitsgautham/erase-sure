use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn get_device_info(device: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Get basic device info using lsblk
    let lsblk_output = std::process::Command::new("lsblk")
        .arg("-J")
        .arg("-o")
        .arg("NAME,SIZE,MODEL,SERIAL,TYPE,MOUNTPOINT")
        .arg(device)
        .output()?;
    
    let device_name = std::path::Path::new(device)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    if lsblk_output.status.success() {
        let lsblk_text = String::from_utf8_lossy(&lsblk_output.stdout);
        if let Ok(lsblk_json) = serde_json::from_str::<serde_json::Value>(&lsblk_text) {
            if let Some(blockdevices) = lsblk_json["blockdevices"].as_array() {
                if let Some(device_info) = blockdevices.first() {
                    return Ok(serde_json::json!({
                        "path": device,
                        "name": device_name,
                        "model": device_info["model"].as_str().unwrap_or("Unknown"),
                        "serial": device_info["serial"].as_str().unwrap_or("Unknown"),
                        "size": device_info["size"].as_str().unwrap_or("Unknown"),
                        "type": device_info["type"].as_str().unwrap_or("disk")
                    }));
                }
            }
        }
    }
    
    // Fallback device info
    Ok(serde_json::json!({
        "path": device,
        "name": device_name,
        "model": "Unknown",
        "serial": "Unknown", 
        "size": "Unknown",
        "type": "disk"
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignature {
    pub alg: String, // "Ed25519"
    pub pubkey_id: String, // "sih_root_v1"
    pub sig: String, // Base64 signature
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
    
    fn export_to_pdf(
        &self,
        cert_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

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
                crate::wipe::WipePolicy::Purge => "PURGE", 
                crate::wipe::WipePolicy::Destroy => "DESTROY"
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
        let linkage = backup_cert_id.map(|id| serde_json::json!({
            "backup_cert_id": id,
            "chain_type": "backup_then_wipe",
            "created_at": created_at
        }));

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
    
    fn export_to_pdf(
        &self,
        _cert_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
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

/// Build a schema-compliant wipe certificate JSON (unsigned)
///
/// Contracts:
/// - Inputs: crate::wipe::WipeResult and optional backup_cert_id
/// - Output: serde_json::Value matching certs/schemas/wipe_schema.json except for signature (added later)
/// - Errors: if device information cannot be gathered minimally (model/serial/bus/capacity)
pub fn build_wipe_certificate_json(
    wipe_result: &crate::wipe::WipeResult,
    backup_cert_id: Option<&str>,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Generate certificate id
    let cert_id = format!(
        "WPE_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );

    // Tool version
    let tool_version = format!("v{}", env!("CARGO_PKG_VERSION"));

    // Created at
    let created_at = chrono::Utc::now().to_rfc3339();

    // Issuer
    let issuer = serde_json::json!({
        "organization": "SecureWipe (SIH)",
        "tool_name": "securewipe",
        "tool_version": tool_version,
        "country": "IN"
    });

    // Device info (schema-compliant)
    let device = schema_device_info(&wipe_result.device)?;

    // Policy mapping
    let nist_level = match wipe_result.policy {
        crate::wipe::WipePolicy::Clear => "CLEAR",
        crate::wipe::WipePolicy::Purge => "PURGE",
        crate::wipe::WipePolicy::Destroy => "DESTROY",
    };

    let method = wipe_result.method.clone();
    let action_mapping = match method.as_str() {
        "controller_sanitize" => "Controller sanitize → PURGE/CLEAR",
        "overwrite" => "Overwrite pass → NIST level",
        _ => "Method → NIST mapping",
    };

    let policy = serde_json::json!({
        "nist_level": nist_level,
        "method": method,
        "action_mapping": action_mapping
    });

    // HPA/DCO section: infer cleared for DESTROY (we call clear_hpa_dco) else false
    let hpa_dco = serde_json::json!({
        "cleared": matches!(wipe_result.policy, crate::wipe::WipePolicy::Destroy)
    });

    // Commands array
    let commands: Vec<Value> = wipe_result
        .commands
        .iter()
        .map(|c| serde_json::json!({
            "cmd": c.command,
            "exit": c.exit_code,
            "ms": c.elapsed_ms
        }))
        .collect();

    // Verify object
    let (verify_result, failures) = if wipe_result.verification_passed {
        ("PASS", 0)
    } else {
        ("FAIL", 1)
    };

    let verify = serde_json::json!({
        "strategy": "random_sectors",
        "samples": wipe_result.verification_samples,
        "coverage": {"mode": "samples", "samples": wipe_result.verification_samples},
        "failures": failures,
        "result": verify_result
    });

    // Overall result
    let result_str = if wipe_result.verification_passed { "PASS" } else { "FAIL" };

    // Environment
    let operator = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let kernel = uname_kernel_string();
    let environment = serde_json::json!({
        "operator": operator,
        "os_kernel": kernel,
        "tool_version": format!("v{}", env!("CARGO_PKG_VERSION")),
    });

    // Evidence (optional fields only) – provide empty object to satisfy required presence
    let evidence = serde_json::json!({});

    // Linkage – schema requires linkage; if absent, use placeholder "UNLINKED"
    let linkage = serde_json::json!({
        "backup_cert_id": backup_cert_id.unwrap_or("UNLINKED")
    });

    // Exceptions – none by default
    let exceptions = serde_json::json!({"items": [], "text": "None"});

    // Metadata – optional; keep minimal for now
    let metadata = serde_json::json!({});

    // Verify URL – optional but useful
    let verify_url = format!(
        "https://verify.securewipe.local/cert/{}",
        &cert_id
    );

    let cert = serde_json::json!({
        "cert_type": "wipe",
        "cert_id": cert_id,
        "certificate_version": "v1.0.0",
        "created_at": created_at,
        "issuer": issuer,
        "device": device,
        "policy": policy,
        "hpa_dco": hpa_dco,
        "commands": commands,
        "verify": verify,
        "result": result_str,
        "environment": environment,
        "evidence": evidence,
        "linkage": linkage,
        "exceptions": exceptions,
        "metadata": metadata,
        "verify_url": verify_url
    });

    Ok(cert)
}

// Helper: produce kernel string like "Linux 6.8.0-35-generic"
fn uname_kernel_string() -> String {
    match Command::new("uname").arg("-sr").output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "Linux".to_string(),
    }
}

// Helper: Build schema-compliant device object from a path (disk or partition)
fn schema_device_info(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // Query lsblk JSON for this path; we will walk up to the disk-level device
    let output = Command::new("lsblk")
        .args(&["-J", "-b", "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,TRAN,PKNAME", path])
        .output()?;
    if !output.status.success() {
        return Err(format!("lsblk failed for {}", path).into());
    }
    let lsblk: Value = serde_json::from_slice(&output.stdout)?;
    let mut model = None;
    let mut serial = None;
    let mut tran = None;
    let mut size_bytes: u64 = 0;
    let mut disk_name = None;

    if let Some(arr) = lsblk.get("blockdevices").and_then(|v| v.as_array()) {
        if let Some(dev) = arr.first() {
            // If this is a partition, prefer parent pkname; otherwise use itself
            let dtype = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let name = dev.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let pkname = dev.get("pkname").and_then(|v| v.as_str());
            let parent_name = if dtype == "part" { pkname.unwrap_or(name) } else { name };
            disk_name = Some(parent_name.to_string());

            // If parent different from current, we need to query lsblk without path filter to find parent entry with details
            if dtype == "part" && pkname.is_some() {
                let all = Command::new("lsblk")
                    .args(&["-J", "-b", "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,TRAN"]) 
                    .output()?;
                if all.status.success() {
                    let all_json: Value = serde_json::from_slice(&all.stdout)?;
                    if let Some(devs) = all_json.get("blockdevices").and_then(|v| v.as_array()) {
                        for d in devs {
                            if d.get("name").and_then(|v| v.as_str()) == Some(parent_name) {
                                model = d.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
                                serial = d.get("serial").and_then(|v| v.as_str()).map(|s| s.to_string());
                                tran = d.get("tran").and_then(|v| v.as_str()).map(|s| s.to_string());
                                size_bytes = d.get("size").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                break;
                            }
                        }
                    }
                }
            } else {
                model = dev.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
                serial = dev.get("serial").and_then(|v| v.as_str()).map(|s| s.to_string());
                tran = dev.get("tran").and_then(|v| v.as_str()).map(|s| s.to_string());
                size_bytes = dev.get("size").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
            }
        }
    }

    let bus: String = match tran.as_deref().map(|s| s.to_lowercase()) {
        Some(ref t) if t == "sata" || t == "ata" => "SATA".to_string(),
        Some(ref t) if t == "nvme" => "NVMe".to_string(),
        Some(ref t) if t == "usb" => "USB".to_string(),
        Some(other) => {
            let upper = other.to_uppercase();
            match upper.as_str() {
                "SAS" | "VIRTIO" => upper,
                _ => "UNKNOWN".to_string(),
            }
        },
        None => "UNKNOWN".to_string(),
    };

    let model = model.unwrap_or_else(|| "Unknown".to_string());
    let serial = serial.unwrap_or_else(|| "Unknown".to_string());
    let capacity_bytes = size_bytes;
    let disk_path = format!("/dev/{}", disk_name.unwrap_or_else(|| path.trim_start_matches("/dev/").to_string()));

    let device = serde_json::json!({
        "model": model,
        "serial": serial,
    "bus": bus,
        "capacity_bytes": capacity_bytes,
        "path": disk_path
    });

    Ok(device)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::{BackupResult, BackupManifest};
    use crate::wipe::{WipeResult, WipePolicy};
    use std::collections::HashMap;
    
    #[test]
    fn test_certificate_operations_trait() {
        let cert_mgr = Ed25519CertificateManager;
        
        let backup_result = BackupResult {
            manifest: BackupManifest {
                files: HashMap::new(),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                total_files: 0,
                total_bytes: 0,
                manifest_sha256: "dummy_hash".to_string(),
            },
            destination: "test".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
            backup_id: "test-backup-123".to_string(),
        };
        
        let result = cert_mgr.create_backup_certificate(&backup_result);
        assert!(result.is_ok());
        
        if let Ok(cert) = result {
            assert_eq!(cert.cert_type, "backup");
            if let Some(signature) = &cert.signature {
                assert_eq!(signature.alg, "Ed25519");
                assert_eq!(signature.pubkey_id, "sih_root_v1");
            }
        }
    }
    
    #[test]
    fn test_wipe_certificate_creation() {
        let cert_mgr = Ed25519CertificateManager;
        
        let wipe_result = WipeResult {
            device: "/dev/sda".to_string(),
            policy: WipePolicy::Purge,
            method: "controller_sanitize".to_string(),
            commands: vec![],
            verification_samples: 5,
            verification_passed: true,
            fallback_reason: None,
        };
        
        let result = cert_mgr.create_wipe_certificate(&wipe_result, Some("backup_cert_123"));
        assert!(result.is_ok());
        
        if let Ok(cert) = result {
            assert_eq!(cert.cert_type, "wipe");
            assert!(cert.linkage.is_some());
        }
    }
    
    #[test]
    fn test_certificate_signature_serialization() {
        let sig = CertificateSignature {
            alg: "Ed25519".to_string(),
            pubkey_id: "sih_root_v1".to_string(),
            sig: "test_signature".to_string(),
        };
        let json = serde_json::to_string(&sig);
        assert!(json.is_ok());
        
        let deserialized: CertificateSignature = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.alg, "Ed25519");
        assert_eq!(deserialized.pubkey_id, "sih_root_v1");
    }
    
    #[test]
    fn test_backup_certificate_serialization() {
        let cert = BackupCertificate {
            cert_id: "backup_123".to_string(),
            cert_type: "backup".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            issuer: serde_json::json!({"organization": "SecureWipe (SIH)"}),
            device: serde_json::json!({"name": "/dev/sda"}),
            files_summary: serde_json::json!({"count": 100}),
            destination: serde_json::json!({"type": "other"}),
            crypto: serde_json::json!({"alg": "AES-256-CTR", "manifest_sha256": "abc123"}),
            verification: serde_json::json!({"strategy": "sampled_files"}),
            policy: serde_json::json!({"name": "NIST SP 800-88 Rev.1"}),
            result: "PASS".to_string(),
            environment: serde_json::json!({"operator": "test"}),
            exceptions: serde_json::json!({"text": "None"}),
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "signature".to_string(),
            }),
            metadata: serde_json::json!({}),
            verify_url: "http://localhost:8000/verify".to_string(),
        };
        
        let json = serde_json::to_string(&cert);
        assert!(json.is_ok());
    }
    
    #[test]
    fn test_wipe_certificate_serialization() {
        let cert = WipeCertificate {
            cert_id: "wipe_123".to_string(),
            cert_type: "wipe".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            device: serde_json::json!({"name": "/dev/sda"}),
            wipe_summary: serde_json::json!({"policy": "PURGE"}),
            linkage: Some(serde_json::json!({"backup_cert_id": "backup_123"})),
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "signature".to_string(),
            }),
        };
        
        let json = serde_json::to_string(&cert);
        assert!(json.is_ok());
    }
    
    #[test]
    fn test_pdf_export() {
        let cert_mgr = Ed25519CertificateManager;
        let result = cert_mgr.export_to_pdf("test_cert_id");
        assert!(result.is_ok());
        
        if let Ok(path) = result {
            assert!(path.contains(".pdf"));
        }
    }

    #[test]
    fn test_backup_certificate_pdf_generation() {
        let cert_mgr = Ed25519CertificateManager;
        let cert = BackupCertificate {
            cert_id: "test_backup_pdf_123".to_string(),
            cert_type: "backup".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
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
            crypto: serde_json::json!({"alg": "AES-256-CTR", "manifest_sha256": "abc123"}),
            verification: serde_json::json!({"strategy": "sampled_files"}),
            policy: serde_json::json!({"name": "NIST SP 800-88 Rev.1"}),
            result: "PASS".to_string(),
            environment: serde_json::json!({"operator": "test"}),
            exceptions: serde_json::json!({"text": "None"}),
            signature: Some(CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "signature".to_string(),
            }),
            metadata: serde_json::json!({}),
            verify_url: "http://localhost:8000/verify".to_string(),
        };

        let result = cert_mgr.generate_backup_certificate_pdf(&cert, Some("https://verify.example.com"));
        assert!(result.is_ok());
        
        if let Ok(path) = result {
            assert!(path.contains("test_backup_pdf_123"));
            assert!(path.contains(".pdf"));
        }
    }

    #[test]
    fn test_wipe_certificate_pdf_generation() {
        let cert_mgr = Ed25519CertificateManager;
        let cert = WipeCertificate {
            cert_id: "test_wipe_pdf_456".to_string(),
            cert_type: "wipe".to_string(),
            certificate_version: "v1.0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
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
                sig: "signature".to_string(),
            }),
        };

        let result = cert_mgr.generate_wipe_certificate_pdf(&cert, None);
        assert!(result.is_ok());
        
        if let Ok(path) = result {
            assert!(path.contains("test_wipe_pdf_456"));
            assert!(path.contains(".pdf"));
        }
    }
}