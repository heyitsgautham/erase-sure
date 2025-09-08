use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignature {
    pub alg: String, // "Ed25519"
    pub pubkey_id: String, // "sih_root_v1"
    pub sig: String, // base64 encoded signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCertificate {
    pub cert_id: String,
    pub cert_type: String, // "backup"
    pub created_at: String,
    pub device: serde_json::Value,
    pub backup_summary: serde_json::Value,
    pub manifest_sha256: String,
    pub signature: CertificateSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeCertificate {
    pub cert_id: String,
    pub cert_type: String, // "wipe"
    pub created_at: String,
    pub device: serde_json::Value,
    pub wipe_summary: serde_json::Value,
    pub linkage: Option<serde_json::Value>,
    pub signature: CertificateSignature,
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
            created_at: chrono::Utc::now().to_rfc3339(),
            device: serde_json::json!({}),
            backup_summary: serde_json::json!({}),
            manifest_sha256: "stub_hash".to_string(),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "stub_signature".to_string(),
            },
        })
    }
    
    fn create_wipe_certificate(
        &self,
        _wipe_result: &crate::wipe::WipeResult,
        backup_cert_id: Option<&str>,
    ) -> Result<WipeCertificate, Box<dyn std::error::Error>> {
        // Stub implementation - will create actual signed certificates
        Ok(WipeCertificate {
            cert_id: "stub_wipe_cert_id".to_string(),
            cert_type: "wipe".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            device: serde_json::json!({}),
            wipe_summary: serde_json::json!({}),
            linkage: backup_cert_id.map(|id| serde_json::json!({"backup_cert_id": id})),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "stub_signature".to_string(),
            },
        })
    }
    
    fn export_to_pdf(
        &self,
        _cert_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Stub implementation - will generate styled PDF
        Ok("stub_pdf_path.pdf".to_string())
    }
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
            },
            destination: "test".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };
        
        let result = cert_mgr.create_backup_certificate(&backup_result);
        assert!(result.is_ok());
        
        if let Ok(cert) = result {
            assert_eq!(cert.cert_type, "backup");
            assert_eq!(cert.signature.alg, "Ed25519");
            assert_eq!(cert.signature.pubkey_id, "sih_root_v1");
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
            created_at: "2023-01-01T00:00:00Z".to_string(),
            device: serde_json::json!({"name": "/dev/sda"}),
            backup_summary: serde_json::json!({"files": 100}),
            manifest_sha256: "abc123".to_string(),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "signature".to_string(),
            },
        };
        
        let json = serde_json::to_string(&cert);
        assert!(json.is_ok());
    }
    
    #[test]
    fn test_wipe_certificate_serialization() {
        let cert = WipeCertificate {
            cert_id: "wipe_123".to_string(),
            cert_type: "wipe".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            device: serde_json::json!({"name": "/dev/sda"}),
            wipe_summary: serde_json::json!({"policy": "PURGE"}),
            linkage: Some(serde_json::json!({"backup_cert_id": "backup_123"})),
            signature: CertificateSignature {
                alg: "Ed25519".to_string(),
                pubkey_id: "sih_root_v1".to_string(),
                sig: "signature".to_string(),
            },
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
}