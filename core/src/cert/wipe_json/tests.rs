use super::*;
use crate::schema::CertificateValidator;
use crate::signer::sign_certificate;
use crate::wipe::{WipeCommand, WipePolicy, WipeResult};
use ed25519_dalek::SigningKey;
use serde_json::json;

fn wipe_result(policy: WipePolicy, passed: bool) -> WipeResult {
    WipeResult {
        device: "/dev/sdz".to_string(),
        policy,
        method: "overwrite".to_string(),
        commands: vec![WipeCommand {
            command: "dd if=/dev/zero of=/dev/sdz bs=1M conv=fdatasync status=progress".into(),
            exit_code: 1,
            elapsed_ms: 1234,
            output: "No space left on device".into(),
        }],
        verification_samples: 32,
        verification_passed: passed,
        fallback_reason: Some("Controller sanitize not available or failed".into()),
    }
}

fn sample_device() -> Value {
    json!({
        "model": "Samsung SSD 980 PRO 1TB",
        "serial": "S6TXNX0R123456",
        "bus": "NVMe",
        "capacity_bytes": 1000204886016u64,
        "path": "/dev/nvme0n1"
    })
}

#[test]
fn signed_certificate_passes_the_wipe_schema() {
    let mut cert = wipe_certificate_for_device(
        &wipe_result(WipePolicy::Purge, true),
        Some("BCK_123"),
        sample_device(),
    );
    let key = SigningKey::generate(&mut rand::rngs::OsRng);
    sign_certificate(&mut cert, &key, false).unwrap();

    let validator = CertificateValidator::new().unwrap();
    let result = validator.validate_certificate(&cert).unwrap();
    assert!(result.valid, "schema errors: {:?}", result.errors);
}

#[test]
fn certificate_records_policy_commands_and_linkage() {
    let cert = wipe_certificate_for_device(
        &wipe_result(WipePolicy::Purge, true),
        Some("BCK_123"),
        sample_device(),
    );

    assert_eq!(cert["cert_type"], "wipe");
    assert!(cert["cert_id"].as_str().unwrap().starts_with("WPE_"));
    assert_eq!(cert["policy"]["nist_level"], "PURGE");
    assert_eq!(cert["hpa_dco"]["cleared"], true);
    assert_eq!(cert["commands"][0]["exit"], 1);
    assert_eq!(cert["commands"][0]["ms"], 1234);
    assert_eq!(cert["verify"]["samples"], 32);
    assert_eq!(cert["result"], "PASS");
    assert_eq!(cert["linkage"]["backup_cert_id"], "BCK_123");
    assert_eq!(cert["device"]["bus"], "NVMe");
}

#[test]
fn failed_verification_and_missing_backup_are_explicit() {
    let cert = wipe_certificate_for_device(
        &wipe_result(WipePolicy::Clear, false),
        None,
        sample_device(),
    );

    assert_eq!(cert["result"], "FAIL");
    assert_eq!(cert["verify"]["result"], "FAIL");
    assert_eq!(cert["verify"]["failures"], 1);
    assert_eq!(cert["hpa_dco"]["cleared"], false);
    assert_eq!(cert["linkage"]["backup_cert_id"], "UNLINKED");
}

#[test]
fn whole_disk_fields_come_from_lsblk() {
    let lsblk = json!({"blockdevices": [{
        "name": "sdb", "type": "disk", "size": "64023257088",
        "model": "SanDisk Ultra", "serial": "4C531001", "tran": "usb"
    }]});

    let device = device_from_lsblk("/dev/sdb", &lsblk, None);

    assert_eq!(device["model"], "SanDisk Ultra");
    assert_eq!(device["serial"], "4C531001");
    assert_eq!(device["bus"], "USB");
    assert_eq!(device["capacity_bytes"], 64023257088u64);
    assert_eq!(device["path"], "/dev/sdb");
}

#[test]
fn partition_reports_its_parent_disk() {
    let lsblk = json!({"blockdevices": [{
        "name": "nvme0n1p2", "type": "part", "size": "1000", "pkname": "nvme0n1"
    }]});
    let all = json!({"blockdevices": [
        {"name": "sda", "type": "disk", "size": "5", "model": "Other", "serial": "X", "tran": "sata"},
        {"name": "nvme0n1", "type": "disk", "size": "1000204886016",
         "model": "Samsung SSD 980 PRO 1TB", "serial": "S6TX", "tran": "nvme"}
    ]});

    let device = device_from_lsblk("/dev/nvme0n1p2", &lsblk, Some(&all));

    assert_eq!(device["path"], "/dev/nvme0n1");
    assert_eq!(device["model"], "Samsung SSD 980 PRO 1TB");
    assert_eq!(device["bus"], "NVMe");
    assert_eq!(device["capacity_bytes"], 1000204886016u64);
}

#[test]
fn unknown_transport_and_empty_listing_fall_back_to_placeholders() {
    let odd = json!({"blockdevices": [{"name": "vda", "type": "disk", "size": "x", "tran": "fc"}]});
    let device = device_from_lsblk("/dev/vda", &odd, None);
    assert_eq!(device["bus"], "UNKNOWN");
    assert_eq!(device["capacity_bytes"], 0);
    assert_eq!(device["model"], "Unknown");

    let virtio = json!({"blockdevices": [{"name": "vdb", "type": "disk", "tran": "virtio"}]});
    assert_eq!(
        device_from_lsblk("/dev/vdb", &virtio, None)["bus"],
        "VIRTIO"
    );

    let empty = json!({"blockdevices": []});
    let device = device_from_lsblk("/dev/sdq", &empty, None);
    assert_eq!(device["path"], "/dev/sdq");
    assert_eq!(device["serial"], "Unknown");
}
