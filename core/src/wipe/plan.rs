//! Dry-run planning: what a wipe would do to a device, without touching it.

use super::WipePolicy;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipePlan {
    pub device: String,
    pub risk: String,
    pub policy: WipePolicy,
    pub hpa_dco_clear: bool,
    pub main_method: String,
    pub verification: VerificationPlan,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPlan {
    pub strategy: String,
    pub samples: usize,
}

/// Plan a wipe operation without performing destructive actions
pub fn plan_wipe(
    device: &str,
    policy: Option<WipePolicy>,
    is_critical: bool,
    iso_mode: bool,
    mock_hdparm: Option<&str>,
    mock_nvme: Option<&str>,
) -> WipePlan {
    let policy = policy.unwrap_or(WipePolicy::Purge);
    let mut hpa_dco_clear = false;
    let mut main_method = "overwrite".to_string();
    let risk = if is_critical {
        "CRITICAL".to_string()
    } else {
        "SAFE".to_string()
    };
    let mut blocked = false;
    let mut reason = None;

    // Guard rails: block CRITICAL unless ISO mode
    if is_critical && !iso_mode {
        blocked = true;
        reason =
            Some("CRITICAL disk wipe blocked unless running from bootable ISO mode".to_string());
    }

    // Probe controller capabilities (non-fatal)
    let hdparm_output = if let Some(mock) = mock_hdparm {
        Some(mock.to_string())
    } else {
        Command::new("hdparm")
            .arg("-I")
            .arg(device)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
    };

    let nvme_output = if let Some(mock) = mock_nvme {
        Some(mock.to_string())
    } else {
        Command::new("nvme")
            .arg("id-ctrl")
            .arg(device)
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
    };

    // Determine method based on controller capabilities
    if let Some(ref hdparm) = hdparm_output {
        if hdparm.contains("sanitize") || hdparm.contains("Security") {
            main_method = "controller_sanitize".to_string();
        }
        if hdparm.contains("HPA") || hdparm.contains("DCO") {
            hpa_dco_clear = true;
        }
    }

    if let Some(ref nvme) = nvme_output {
        if nvme.contains("sanitize") {
            main_method = "controller_sanitize".to_string();
        }
    }

    let verification = VerificationPlan {
        strategy: "random_sectors".to_string(),
        samples: 128,
    };

    WipePlan {
        device: device.to_string(),
        risk,
        policy,
        hpa_dco_clear,
        main_method,
        verification,
        blocked,
        reason,
    }
}
