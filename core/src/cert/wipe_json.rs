//! Schema-shaped wipe certificate JSON, as validated by certs/schemas/wipe_schema.json.

use serde_json::Value;
use std::process::Command;

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
    let device = schema_device_info(&wipe_result.device)?;
    Ok(wipe_certificate_for_device(
        wipe_result,
        backup_cert_id,
        device,
    ))
}

/// Assemble the certificate around an already-resolved schema `device` object.
fn wipe_certificate_for_device(
    wipe_result: &crate::wipe::WipeResult,
    backup_cert_id: Option<&str>,
    device: Value,
) -> Value {
    // Generate certificate id
    let cert_id = format!("WPE_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

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

    // Policy mapping
    let nist_level = match wipe_result.policy {
        crate::wipe::WipePolicy::Clear => "CLEAR",
        crate::wipe::WipePolicy::Purge => "PURGE",
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

    // HPA/DCO section: cleared for PURGE (we call clear_hpa_dco)
    let hpa_dco = serde_json::json!({
        "cleared": matches!(wipe_result.policy, crate::wipe::WipePolicy::Purge)
    });

    // Commands array
    let commands: Vec<Value> = wipe_result
        .commands
        .iter()
        .map(|c| {
            serde_json::json!({
                "cmd": c.command,
                "exit": c.exit_code,
                "ms": c.elapsed_ms
            })
        })
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
    let result_str = if wipe_result.verification_passed {
        "PASS"
    } else {
        "FAIL"
    };

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
    let verify_url = format!("https://verify.securewipe.local/cert/{}", cert_id);

    serde_json::json!({
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
    })
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
        .args([
            "-J",
            "-b",
            "-o",
            "NAME,TYPE,SIZE,MODEL,SERIAL,TRAN,PKNAME",
            path,
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!("lsblk failed for {}", path).into());
    }
    let lsblk: Value = serde_json::from_slice(&output.stdout)?;

    // A partition's model, serial and transport live on its parent disk,
    // which needs a second, unfiltered lsblk call.
    let first = lsblk
        .get("blockdevices")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());
    let is_partition_with_parent = first.is_some_and(|dev| {
        dev.get("type").and_then(|v| v.as_str()) == Some("part")
            && dev.get("pkname").and_then(|v| v.as_str()).is_some()
    });
    let all_devices = if is_partition_with_parent {
        let all = Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,TYPE,SIZE,MODEL,SERIAL,TRAN"])
            .output()?;
        if all.status.success() {
            Some(serde_json::from_slice::<Value>(&all.stdout)?)
        } else {
            None
        }
    } else {
        None
    };

    Ok(device_from_lsblk(path, &lsblk, all_devices.as_ref()))
}

/// Map `lsblk -J -b` output for `path` to the schema's device object.
///
/// `all_devices` is the unfiltered lsblk listing, used to find a partition's parent disk.
fn device_from_lsblk(path: &str, lsblk: &Value, all_devices: Option<&Value>) -> Value {
    let str_field = |d: &Value, key: &str| d.get(key).and_then(|v| v.as_str()).map(String::from);
    let size_field = |d: &Value| {
        d.get("size")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    };

    let mut model = None;
    let mut serial = None;
    let mut tran = None;
    let mut size_bytes: u64 = 0;
    let mut disk_name = None;

    if let Some(dev) = lsblk
        .get("blockdevices")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
    {
        // If this is a partition, prefer parent pkname; otherwise use itself
        let dtype = dev.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let name = dev.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let pkname = dev.get("pkname").and_then(|v| v.as_str());
        let parent_name = if dtype == "part" {
            pkname.unwrap_or(name)
        } else {
            name
        };
        disk_name = Some(parent_name.to_string());

        if dtype == "part" && pkname.is_some() {
            let parent = all_devices
                .and_then(|all| all.get("blockdevices"))
                .and_then(|v| v.as_array())
                .and_then(|devs| {
                    devs.iter()
                        .find(|d| d.get("name").and_then(|v| v.as_str()) == Some(parent_name))
                });
            if let Some(d) = parent {
                model = str_field(d, "model");
                serial = str_field(d, "serial");
                tran = str_field(d, "tran");
                size_bytes = size_field(d);
            }
        } else {
            model = str_field(dev, "model");
            serial = str_field(dev, "serial");
            tran = str_field(dev, "tran");
            size_bytes = size_field(dev);
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
        }
        None => "UNKNOWN".to_string(),
    };

    let disk_path = format!(
        "/dev/{}",
        disk_name.unwrap_or_else(|| path.trim_start_matches("/dev/").to_string())
    );

    serde_json::json!({
        "model": model.unwrap_or_else(|| "Unknown".to_string()),
        "serial": serial.unwrap_or_else(|| "Unknown".to_string()),
        "bus": bus,
        "capacity_bytes": size_bytes,
        "path": disk_path
    })
}

#[cfg(test)]
mod tests;
