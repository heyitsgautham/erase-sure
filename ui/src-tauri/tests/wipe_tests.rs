//! The checks in front of the destructive wipe. Nothing here runs a wipe.

use app::wipe::{
    build_wipe_args, check_confirmation, expected_confirmation, is_critical_mount, WipeConfirmation,
};

fn confirmation(policy: &str, user_input: &str) -> WipeConfirmation {
    WipeConfirmation {
        device: "/dev/sdb".to_string(),
        serial: "4C53000123".to_string(),
        policy: policy.to_string(),
        user_input: user_input.to_string(),
    }
}

#[test]
fn only_the_exact_phrase_for_the_serial_confirms() {
    assert_eq!(expected_confirmation("X1"), "WIPE X1");
    assert!(check_confirmation(&confirmation("purge", "WIPE 4C53000123")).is_ok());

    for wrong in ["wipe 4C53000123", "WIPE 4C5300012", "WIPE  4C53000123", ""] {
        let err = check_confirmation(&confirmation("purge", wrong)).unwrap_err();
        assert!(err.contains("Expected 'WIPE 4C53000123'"), "{err}");
    }
}

#[test]
fn wipe_args_carry_the_policy_the_danger_flag_and_signing() {
    let args = build_wipe_args(&confirmation("purge", "WIPE 4C53000123"), None);
    assert_eq!(
        args,
        [
            "wipe",
            "--device",
            "/dev/sdb",
            "--policy",
            "PURGE",
            "--danger-allow-wipe",
            "--sign"
        ]
    );
}

#[test]
fn wipe_args_link_a_backup_certificate_when_given() {
    let args = build_wipe_args(&confirmation("clear", "x"), Some("b-1".to_string()));
    assert_eq!(
        &args[4..],
        [
            "CLEAR",
            "--danger-allow-wipe",
            "--sign",
            "--backup-cert-id",
            "b-1"
        ]
    );
}

#[test]
fn system_mounts_mark_a_device_critical() {
    let mounts = "\
/dev/nvme0n1p2 on / type ext4 (rw,relatime)
/dev/nvme0n1p1 on /boot type vfat (rw)
/dev/sdb1 on /media/usb type vfat (rw)
tmpfs on /run type tmpfs (rw)";

    assert!(is_critical_mount(mounts, "/dev/nvme0n1"));
    assert!(!is_critical_mount(mounts, "/dev/sdb"));
    assert!(!is_critical_mount(mounts, "/dev/sdc"));
}
