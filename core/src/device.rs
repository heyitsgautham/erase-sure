use serde::{Deserialize, Deserializer, Serialize};
use std::process::Command;

// Custom deserializer to handle size field that can be either string or integer
fn deserialize_size<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or integer representing size")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(SizeValueVisitor)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }
    }

    struct SizeValueVisitor;

    impl<'de> Visitor<'de> for SizeValueVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }
    }

    deserializer.deserialize_option(SizeVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "CRITICAL")]
    Critical,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "SAFE")]
    Safe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub capacity_bytes: u64,
    pub bus: Option<String>, // SATA, NVMe, USB
    pub mountpoints: Vec<String>,
    pub risk_level: RiskLevel,
}

// Internal structs for parsing lsblk JSON output
#[derive(Debug, Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Deserialize)]
struct LsblkDevice {
    name: String,
    #[serde(rename = "type")]
    device_type: Option<String>,
    #[serde(deserialize_with = "deserialize_size")]
    size: Option<String>,
    mountpoint: Option<String>,
    model: Option<String>,
    serial: Option<String>,
    tran: Option<String>, // Transport type (sata, nvme, usb, etc.)
    children: Option<Vec<LsblkDevice>>,
}

pub trait DeviceDiscovery {
    fn discover_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>>;
}

pub struct LinuxDeviceDiscovery {
    pub enable_enrichment: bool,
}

impl Default for LinuxDeviceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxDeviceDiscovery {
    pub fn new() -> Self {
        Self {
            enable_enrichment: true,
        }
    }

    pub fn new_without_enrichment() -> Self {
        Self {
            enable_enrichment: false,
        }
    }

    fn run_lsblk(&self) -> Result<LsblkOutput, Box<dyn std::error::Error>> {
        let output = Command::new("lsblk")
            .args([
                "-J", // JSON output
                "-o",
                "NAME,TYPE,SIZE,MOUNTPOINT,MODEL,SERIAL,TRAN,PKNAME",
                "-b", // Show sizes in bytes
            ])
            .output()
            .map_err(|e| -> Box<dyn std::error::Error> {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "lsblk command not found - this tool requires Linux with util-linux package"
                    ))
                } else {
                    Box::new(e)
                }
            })?;

        if !output.status.success() {
            return Err(format!(
                "lsblk failed with exit code: {}",
                output.status.code().unwrap_or(-1)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let lsblk_output: LsblkOutput = serde_json::from_str(&stdout)?;
        Ok(lsblk_output)
    }

    fn parse_size(&self, size_str: Option<&String>) -> u64 {
        size_str.and_then(|s| s.parse::<u64>().ok()).unwrap_or(0)
    }

    fn normalize_transport(&self, tran: Option<&String>) -> Option<String> {
        tran.map(|t| match t.to_lowercase().as_str() {
            "sata" => "SATA".to_string(),
            "nvme" => "NVMe".to_string(),
            "usb" => "USB".to_string(),
            "ata" => "SATA".to_string(), // ATA is typically SATA
            other => other.to_uppercase(),
        })
    }

    fn collect_mountpoints(device: &LsblkDevice) -> Vec<String> {
        let mut mountpoints = Vec::new();

        // Add this device's mountpoint if it exists
        if let Some(ref mp) = device.mountpoint {
            if !mp.is_empty() {
                mountpoints.push(mp.clone());
            }
        }

        // Recursively collect mountpoints from children (partitions)
        if let Some(ref children) = device.children {
            for child in children {
                mountpoints.extend(Self::collect_mountpoints(child));
            }
        }

        mountpoints
    }

    fn classify_risk(&self, mountpoints: &[String]) -> RiskLevel {
        // CRITICAL: Contains root filesystem
        if mountpoints.iter().any(|mp| mp == "/") {
            return RiskLevel::Critical;
        }

        // HIGH: Any mounted writable volume (excluding special filesystems)
        let writable_mounts = mountpoints.iter().any(|mp| {
            !mp.starts_with("/sys") &&
            !mp.starts_with("/proc") &&
            !mp.starts_with("/dev") &&
            !mp.starts_with("/run") &&
            mp != "/boot/efi" && // EFI system partition is typically read-only
            !mp.is_empty()
        });

        if writable_mounts {
            RiskLevel::High
        } else {
            RiskLevel::Safe
        }
    }

    fn enrich_device_info(&self, device: &mut Device) {
        if !self.enable_enrichment {
            return;
        }

        // Try to get additional info from smartctl, hdparm, or nvme-cli
        // These commands are non-destructive and read-only

        // Try smartctl first (works for most drives)
        if let Ok(output) = Command::new("smartctl").args(["-i", &device.name]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                self.parse_smartctl_output(&stdout, device);
            }
        }

        // Try hdparm for SATA devices if we don't have complete info
        if device.bus.as_ref().is_some_and(|b| b == "SATA")
            && (device.model.is_none() || device.serial.is_none())
        {
            if let Ok(output) = Command::new("hdparm").args(["-I", &device.name]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.parse_hdparm_output(&stdout, device);
                }
            }
        }

        // Try nvme-cli for NVMe devices if we don't have complete info
        if device.bus.as_ref().is_some_and(|b| b == "NVMe")
            && (device.model.is_none() || device.serial.is_none())
        {
            if let Ok(output) = Command::new("nvme")
                .args(["id-ctrl", &device.name])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.parse_nvme_output(&stdout, device);
                }
            }
        }
    }

    fn parse_smartctl_output(&self, output: &str, device: &mut Device) {
        for line in output.lines() {
            if line.starts_with("Device Model:") && device.model.is_none() {
                if let Some(model) = line.split(':').nth(1) {
                    device.model = Some(model.trim().to_string());
                }
            } else if line.starts_with("Serial Number:") && device.serial.is_none() {
                if let Some(serial) = line.split(':').nth(1) {
                    device.serial = Some(serial.trim().to_string());
                }
            }
        }
    }

    fn parse_hdparm_output(&self, output: &str, device: &mut Device) {
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Model Number:") && device.model.is_none() {
                if let Some(model) = line.split(':').nth(1) {
                    device.model = Some(model.trim().to_string());
                }
            } else if line.starts_with("Serial Number:") && device.serial.is_none() {
                if let Some(serial) = line.split(':').nth(1) {
                    device.serial = Some(serial.trim().to_string());
                }
            }
        }
    }

    fn parse_nvme_output(&self, output: &str, device: &mut Device) {
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("mn") && line.contains(':') && device.model.is_none() {
                if let Some(model) = line.split(':').nth(1) {
                    device.model = Some(model.trim().to_string());
                }
            } else if line.starts_with("sn") && line.contains(':') && device.serial.is_none() {
                if let Some(serial) = line.split(':').nth(1) {
                    device.serial = Some(serial.trim().to_string());
                }
            }
        }
    }

    fn process_device(&self, lsblk_device: &LsblkDevice) -> Option<Device> {
        // Only process disk devices (not partitions)
        if lsblk_device.device_type.as_ref() != Some(&"disk".to_string()) {
            return None;
        }

        let device_name = format!("/dev/{}", lsblk_device.name);
        let capacity_bytes = self.parse_size(lsblk_device.size.as_ref());
        let mountpoints = Self::collect_mountpoints(lsblk_device);
        let risk_level = self.classify_risk(&mountpoints);
        let bus = self.normalize_transport(lsblk_device.tran.as_ref());

        let mut device = Device {
            name: device_name,
            model: lsblk_device.model.clone(),
            serial: lsblk_device.serial.clone(),
            capacity_bytes,
            bus,
            mountpoints,
            risk_level,
        };

        // Try to enrich with additional device information
        self.enrich_device_info(&mut device);

        Some(device)
    }
}

impl DeviceDiscovery for LinuxDeviceDiscovery {
    fn discover_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>> {
        let lsblk_output = self.run_lsblk()?;

        let mut devices = Vec::new();
        for lsblk_device in &lsblk_output.blockdevices {
            if let Some(device) = self.process_device(lsblk_device) {
                devices.push(device);
            }
        }

        Ok(devices)
    }
}

#[cfg(test)]
mod tests;
