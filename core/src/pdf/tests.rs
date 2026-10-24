use super::*;
use crate::cert::{BackupCertificate, CertificateSignature, WipeCertificate};
use std::fs;
use tempfile::TempDir;

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
