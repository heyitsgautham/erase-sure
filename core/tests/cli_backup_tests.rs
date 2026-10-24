//! `securewipe backup` end to end: temp source, temp destination, temp HOME.

mod common;

use assert_cmd::Command;
use serde_json::Value;

#[test]
fn backup_encrypts_files_and_writes_a_signed_certificate() {
    let work = tempfile::TempDir::new().unwrap();
    let source = work.path().join("source");
    let dest = work.path().join("dest");
    let home = work.path().join("home");
    for dir in [&source, &dest, &home] {
        std::fs::create_dir_all(dir).unwrap();
    }
    let plaintext = b"quarterly payroll, do not share";
    std::fs::write(source.join("payroll.txt"), plaintext).unwrap();
    std::fs::write(source.join("notes.txt"), b"second file").unwrap();
    let keys = common::TestKeys::generate();

    let output = Command::cargo_bin("securewipe")
        .unwrap()
        .current_dir(work.path())
        .env("HOME", &home)
        .env("SECUREWIPE_SIGN_KEY_PATH", &keys.private)
        .args([
            "backup",
            "--device",
            "/dev/securewipe-test",
            "--dest",
            dest.to_str().unwrap(),
            "--paths",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Files processed: 2"));
    assert!(stdout.contains("Verification status: PASSED"));

    // The destination holds a manifest and ciphertext, not the plaintext.
    let backup_dir = std::fs::read_dir(&dest)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let backup_id = backup_dir
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(backup_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["total_files"], 2);
    let stored = std::fs::read(backup_dir.join("source").join("payroll.txt")).unwrap();
    assert_ne!(stored, plaintext);

    // The certificate under HOME is signed and verifies with the matching public key.
    let cert = home
        .join("SecureWipe")
        .join("certificates")
        .join(format!("{backup_id}.json"));
    let verify = Command::cargo_bin("securewipe")
        .unwrap()
        .current_dir(work.path())
        .args([
            "cert",
            "verify",
            "--file",
            cert.to_str().unwrap(),
            "--pubkey",
            keys.public_str(),
        ])
        .output()
        .unwrap();
    let result: Value = serde_json::from_slice(&verify.stdout).unwrap();
    assert_eq!(result["signature_valid"], true, "{result}");
}
