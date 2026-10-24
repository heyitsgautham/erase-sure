//! `securewipe cert validate` end to end, through the built binary.

use assert_cmd::Command;
use serde_json::Value;

fn validate(file: &str) -> (bool, Value) {
    let output = Command::cargo_bin("securewipe")
        .unwrap()
        .args(["cert", "validate", "--file", file])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{e}: {stdout}"));
    (output.status.success(), json)
}

#[test]
fn schema_valid_backup_certificate_passes() {
    let (ok, out) = validate("../tests/test_backup_cert.json");

    assert!(ok);
    assert_eq!(out["op"], "cert_validate");
    assert_eq!(out["schema_type"], "backup");
    assert_eq!(out["schema_valid"], true);
}

#[test]
fn certificate_missing_required_sections_fails_with_errors() {
    let (ok, out) = validate("test_data/unsigned_cert.json");

    assert!(!ok);
    assert_eq!(out["schema_valid"], false);
    let errors = out["schema_errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e
        .as_str()
        .unwrap()
        .contains("\"crypto\" is a required property")));
}

#[test]
fn malformed_json_is_reported_not_panicked_on() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("broken.json");
    std::fs::write(&path, "{not json").unwrap();

    let (ok, out) = validate(path.to_str().unwrap());

    assert!(!ok);
    assert_eq!(out["schema_valid"], false);
    assert!(out["error"]
        .as_str()
        .unwrap()
        .starts_with("Invalid JSON in certificate file"));
}

#[test]
fn missing_file_is_an_error() {
    let (ok, out) = validate("test_data/does_not_exist.json");

    assert!(!ok);
    assert_eq!(out["schema_valid"], Value::Null);
    assert!(out["error"].as_str().unwrap().contains("not found"));
}
