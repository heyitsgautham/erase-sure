use super::*;
use crate::backup::{BackupManifest, BackupResult};
use crate::wipe::{WipePolicy, WipeResult};
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

    let result =
        cert_mgr.generate_backup_certificate_pdf(&cert, Some("https://verify.example.com"));
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
