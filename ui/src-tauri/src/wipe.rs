//! Checks and arguments for the one destructive call the desktop app can make.

use serde::{Deserialize, Serialize};

/// What the user confirmed in the wipe dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WipeConfirmation {
    pub device: String,
    pub serial: String,
    pub policy: String,
    pub user_input: String,
}

/// The phrase the user must type: `WIPE <serial>`.
pub fn expected_confirmation(serial: &str) -> String {
    format!("WIPE {}", serial)
}

/// Accept only the exact phrase for this device's serial.
pub fn check_confirmation(confirmation: &WipeConfirmation) -> Result<(), String> {
    let expected = expected_confirmation(&confirmation.serial);
    if confirmation.user_input == expected {
        Ok(())
    } else {
        Err(format!(
            "Confirmation failed. Expected '{}', got '{}'",
            expected, confirmation.user_input
        ))
    }
}

/// CLI arguments for the confirmed wipe. The certificate is always signed.
pub fn build_wipe_args(
    confirmation: &WipeConfirmation,
    backup_cert_id: Option<String>,
) -> Vec<String> {
    let mut args = vec![
        "wipe".to_string(),
        "--device".to_string(),
        confirmation.device.clone(),
        "--policy".to_string(),
        confirmation.policy.to_uppercase(),
        "--danger-allow-wipe".to_string(),
        "--sign".to_string(),
    ];
    if let Some(id) = backup_cert_id {
        args.push("--backup-cert-id".to_string());
        args.push(id);
    }
    args
}

/// True when `mount` output shows the device on a system path.
pub fn is_critical_mount(mount_output: &str, device: &str) -> bool {
    mount_output.lines().any(|line| {
        line.contains(device)
            && (line.contains(" / ")
                || line.contains(" /boot ")
                || line.contains(" /usr ")
                || line.contains(" /etc "))
    })
}
