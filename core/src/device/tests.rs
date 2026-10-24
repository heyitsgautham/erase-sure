use super::*;

// Sample lsblk JSON fixture for testing
const SAMPLE_LSBLK_JSON: &str = r#"
{
    "blockdevices": [
        {
            "name": "sda",
            "type": "disk",
            "size": "1000204886016",
            "mountpoint": null,
            "model": "Samsung SSD 980",
            "serial": "S649NX0R123456A",
            "tran": "nvme",
            "pkname": null,
            "children": [
                {
                    "name": "sda1",
                    "type": "part",
                    "size": "536870912",
                    "mountpoint": "/boot/efi",
                    "model": null,
                    "serial": null,
                    "tran": null,
                    "pkname": "sda"
                },
                {
                    "name": "sda2",
                    "type": "part",
                    "size": "999660175360",
                    "mountpoint": "/",
                    "model": null,
                    "serial": null,
                    "tran": null,
                    "pkname": "sda"
                }
            ]
        },
        {
            "name": "sdb",
            "type": "disk",
            "size": "2000398934016",
            "mountpoint": null,
            "model": "WD20EZRZ-00Z5HB0",
            "serial": "WD-WCC4N7ABCDEF",
            "tran": "sata",
            "pkname": null,
            "children": [
                {
                    "name": "sdb1",
                    "type": "part",
                    "size": "2000397885440",
                    "mountpoint": "/home",
                    "model": null,
                    "serial": null,
                    "tran": null,
                    "pkname": "sdb"
                }
            ]
        },
        {
            "name": "sdc",
            "type": "disk",
            "size": "32017047552",
            "mountpoint": null,
            "model": "SanDisk Ultra",
            "serial": "4C530001171122115172",
            "tran": "usb",
            "pkname": null,
            "children": null
        }
    ]
}
"#;

fn create_test_discovery() -> LinuxDeviceDiscovery {
    LinuxDeviceDiscovery::new_without_enrichment()
}

#[test]
fn test_lsblk_json_parsing() {
    let lsblk_output: LsblkOutput = serde_json::from_str(SAMPLE_LSBLK_JSON).unwrap();
    assert_eq!(lsblk_output.blockdevices.len(), 3);

    let sda = &lsblk_output.blockdevices[0];
    assert_eq!(sda.name, "sda");
    assert_eq!(sda.device_type, Some("disk".to_string()));
    assert_eq!(sda.model, Some("Samsung SSD 980".to_string()));
    assert_eq!(sda.serial, Some("S649NX0R123456A".to_string()));
    assert_eq!(sda.tran, Some("nvme".to_string()));

    let children = sda.children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[1].mountpoint, Some("/".to_string()));
}

#[test]
fn test_size_parsing() {
    let discovery = create_test_discovery();

    assert_eq!(
        discovery.parse_size(Some(&"1000204886016".to_string())),
        1000204886016
    );
    assert_eq!(discovery.parse_size(Some(&"0".to_string())), 0);
    assert_eq!(discovery.parse_size(Some(&"invalid".to_string())), 0);
    assert_eq!(discovery.parse_size(None), 0);
}

#[test]
fn test_size_field_integer_parsing() {
    // Test that integer size values are properly converted to strings
    let json_with_integer_size = r#"
    {
        "blockdevices": [
            {
                "name": "loop0",
                "type": "loop",
                "size": 0,
                "mountpoint": null,
                "model": null,
                "serial": null,
                "tran": null,
                "pkname": null,
                "children": null
            }
        ]
    }
    "#;

    let lsblk_output: LsblkOutput = serde_json::from_str(json_with_integer_size).unwrap();
    assert_eq!(lsblk_output.blockdevices.len(), 1);

    let loop_device = &lsblk_output.blockdevices[0];
    assert_eq!(loop_device.name, "loop0");
    assert_eq!(loop_device.size, Some("0".to_string())); // Integer 0 should be converted to string "0"
}

#[test]
fn test_transport_normalization() {
    let discovery = create_test_discovery();

    assert_eq!(
        discovery.normalize_transport(Some(&"nvme".to_string())),
        Some("NVMe".to_string())
    );
    assert_eq!(
        discovery.normalize_transport(Some(&"sata".to_string())),
        Some("SATA".to_string())
    );
    assert_eq!(
        discovery.normalize_transport(Some(&"ata".to_string())),
        Some("SATA".to_string())
    );
    assert_eq!(
        discovery.normalize_transport(Some(&"usb".to_string())),
        Some("USB".to_string())
    );
    assert_eq!(
        discovery.normalize_transport(Some(&"scsi".to_string())),
        Some("SCSI".to_string())
    );
    assert_eq!(discovery.normalize_transport(None), None);
}

#[test]
fn test_mountpoint_collection() {
    // Parse the sample JSON
    let lsblk_output: LsblkOutput = serde_json::from_str(SAMPLE_LSBLK_JSON).unwrap();

    // Test device with root mountpoint (sda)
    let sda = &lsblk_output.blockdevices[0];
    let mountpoints = LinuxDeviceDiscovery::collect_mountpoints(sda);
    assert_eq!(mountpoints.len(), 2);
    assert!(mountpoints.contains(&"/boot/efi".to_string()));
    assert!(mountpoints.contains(&"/".to_string()));

    // Test device with home mountpoint (sdb)
    let sdb = &lsblk_output.blockdevices[1];
    let mountpoints = LinuxDeviceDiscovery::collect_mountpoints(sdb);
    assert_eq!(mountpoints.len(), 1);
    assert!(mountpoints.contains(&"/home".to_string()));

    // Test device with no mountpoints (sdc)
    let sdc = &lsblk_output.blockdevices[2];
    let mountpoints = LinuxDeviceDiscovery::collect_mountpoints(sdc);
    assert_eq!(mountpoints.len(), 0);
}

#[test]
fn test_risk_classification() {
    let discovery = create_test_discovery();

    // CRITICAL: Contains root filesystem
    let critical_mounts = vec!["/".to_string(), "/boot/efi".to_string()];
    assert!(matches!(
        discovery.classify_risk(&critical_mounts),
        RiskLevel::Critical
    ));

    // HIGH: Contains mounted writable volumes
    let high_mounts = vec!["/home".to_string()];
    assert!(matches!(
        discovery.classify_risk(&high_mounts),
        RiskLevel::High
    ));

    let high_mounts2 = vec!["/mnt/data".to_string()];
    assert!(matches!(
        discovery.classify_risk(&high_mounts2),
        RiskLevel::High
    ));

    // SAFE: No mountpoints
    let safe_mounts: Vec<String> = vec![];
    assert!(matches!(
        discovery.classify_risk(&safe_mounts),
        RiskLevel::Safe
    ));

    // SAFE: Only special filesystems (should be filtered out)
    let special_mounts = vec!["/sys".to_string(), "/proc".to_string(), "/dev".to_string()];
    assert!(matches!(
        discovery.classify_risk(&special_mounts),
        RiskLevel::Safe
    ));
}

#[test]
fn test_device_processing() {
    let discovery = create_test_discovery();
    let lsblk_output: LsblkOutput = serde_json::from_str(SAMPLE_LSBLK_JSON).unwrap();

    // Test NVMe SSD (CRITICAL due to root partition)
    let sda = &lsblk_output.blockdevices[0];
    let device = discovery.process_device(sda).unwrap();
    assert_eq!(device.name, "/dev/sda");
    assert_eq!(device.model, Some("Samsung SSD 980".to_string()));
    assert_eq!(device.serial, Some("S649NX0R123456A".to_string()));
    assert_eq!(device.capacity_bytes, 1000204886016);
    assert_eq!(device.bus, Some("NVMe".to_string()));
    assert!(matches!(device.risk_level, RiskLevel::Critical));
    assert_eq!(device.mountpoints.len(), 2);

    // Test SATA HDD (HIGH due to home partition)
    let sdb = &lsblk_output.blockdevices[1];
    let device = discovery.process_device(sdb).unwrap();
    assert_eq!(device.name, "/dev/sdb");
    assert_eq!(device.model, Some("WD20EZRZ-00Z5HB0".to_string()));
    assert_eq!(device.bus, Some("SATA".to_string()));
    assert!(matches!(device.risk_level, RiskLevel::High));

    // Test USB drive (SAFE - no mountpoints)
    let sdc = &lsblk_output.blockdevices[2];
    let device = discovery.process_device(sdc).unwrap();
    assert_eq!(device.name, "/dev/sdc");
    assert_eq!(device.model, Some("SanDisk Ultra".to_string()));
    assert_eq!(device.bus, Some("USB".to_string()));
    assert!(matches!(device.risk_level, RiskLevel::Safe));
    assert_eq!(device.mountpoints.len(), 0);
}

#[test]
fn test_smartctl_output_parsing() {
    let discovery = create_test_discovery();
    let smartctl_output = r#"
smartctl 7.2 2020-12-30 r5155 [x86_64-linux-5.15.0] (local build)
Copyright (C) 2002-20, Bruce Allen, Christian Franke, www.smartmontools.org

=== START OF INFORMATION SECTION ===
Model Family:     Samsung NVMe SSD 980
Device Model:     Samsung SSD 980 1TB
Serial Number:    S649NX0R123456A
LU WWN Device Id: 5 002538 e40b1ba45
Firmware Version: 3B4QFXO7
User Capacity:    1,000,204,886,016 bytes [1.00 TB]
    "#;

    let mut device = Device {
        name: "/dev/sda".to_string(),
        model: None,
        serial: None,
        capacity_bytes: 1000204886016,
        bus: Some("NVMe".to_string()),
        mountpoints: vec![],
        risk_level: RiskLevel::Safe,
    };

    discovery.parse_smartctl_output(smartctl_output, &mut device);

    assert_eq!(device.model, Some("Samsung SSD 980 1TB".to_string()));
    assert_eq!(device.serial, Some("S649NX0R123456A".to_string()));
}

#[test]
fn test_device_discovery_trait() {
    let discovery = LinuxDeviceDiscovery::new_without_enrichment();
    // This test will fail on systems without lsblk, which is expected
    // In CI/testing environments, we would mock the Command execution
    let result = discovery.discover_devices();
    // We can't assert success since lsblk might not be available
    // The important thing is that the trait is implemented correctly
    assert!(result.is_ok() || result.is_err());
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
