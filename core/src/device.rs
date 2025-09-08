use serde::{Deserialize, Serialize};

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

#[allow(dead_code)] // MVP: Implementation pending
pub trait DeviceDiscovery {
    fn discover_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>>;
}

#[allow(dead_code)] // MVP: Implementation pending
pub struct LinuxDeviceDiscovery;

impl DeviceDiscovery for LinuxDeviceDiscovery {
    fn discover_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>> {
        // Stub implementation - will use lsblk -J in real implementation
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_discovery_trait() {
        let discovery = LinuxDeviceDiscovery;
        let result = discovery.discover_devices();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_risk_level_serialization() {
        let risk = RiskLevel::Critical;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"CRITICAL\"");
        
        let risk = RiskLevel::High;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"HIGH\"");
        
        let risk = RiskLevel::Safe;
        let json = serde_json::to_string(&risk).unwrap();
        assert_eq!(json, "\"SAFE\"");
    }
    
    #[test]
    fn test_risk_level_deserialization() {
        let json = "\"CRITICAL\"";
        let risk: RiskLevel = serde_json::from_str(json).unwrap();
        matches!(risk, RiskLevel::Critical);
        
        let json = "\"HIGH\"";
        let risk: RiskLevel = serde_json::from_str(json).unwrap();
        matches!(risk, RiskLevel::High);
        
        let json = "\"SAFE\"";
        let risk: RiskLevel = serde_json::from_str(json).unwrap();
        matches!(risk, RiskLevel::Safe);
    }
    
    #[test]
    fn test_device_creation() {
        let device = Device {
            name: "/dev/sda".to_string(),
            model: Some("Samsung SSD".to_string()),
            serial: Some("S123456789".to_string()),
            capacity_bytes: 1000000000000,
            bus: Some("SATA".to_string()),
            mountpoints: vec!["/".to_string()],
            risk_level: RiskLevel::Critical,
        };
        
        assert_eq!(device.name, "/dev/sda");
        assert_eq!(device.model, Some("Samsung SSD".to_string()));
        assert_eq!(device.capacity_bytes, 1000000000000);
        assert_eq!(device.mountpoints.len(), 1);
        matches!(device.risk_level, RiskLevel::Critical);
    }
    
    #[test]
    fn test_device_serialization() {
        let device = Device {
            name: "/dev/sda".to_string(),
            model: Some("Test SSD".to_string()),
            serial: Some("TEST123".to_string()),
            capacity_bytes: 500000000000,
            bus: Some("NVMe".to_string()),
            mountpoints: vec![],
            risk_level: RiskLevel::Safe,
        };
        
        let json = serde_json::to_string(&device);
        assert!(json.is_ok());
        
        let deserialized: Device = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.name, device.name);
        assert_eq!(deserialized.capacity_bytes, device.capacity_bytes);
    }
}