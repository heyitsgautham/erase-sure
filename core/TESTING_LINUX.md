# Testing SecureWipe Device Discovery on Linux

The device discovery feature is designed specifically for Linux systems and requires the `util-linux` package (which provides the `lsblk` command).

## Prerequisites

On Linux systems, ensure you have:
- `util-linux` package installed (usually available by default)
- Optional tools for enrichment: `smartmontools`, `hdparm`, `nvme-cli`

```bash
# Ubuntu/Debian
sudo apt install util-linux smartmontools hdparm nvme-cli

# RHEL/CentOS/Fedora
sudo dnf install util-linux smartmontools hdparm nvme-cli

# Arch Linux
sudo pacman -S util-linux smartmontools hdparm nvme-cli
```

## Build and Test

```bash
# Build the project (debug version)
cargo build

# Build optimized release version
cargo build --release

# Run all unit tests (these work on any platform with sample fixtures)
cargo test device::tests --lib

# Run full test suite
cargo test
```

## Recent Fixes

### Size Field Handling (Fixed in Latest Version)
The device discovery now properly handles cases where `lsblk` returns integer values (like `0`) for the size field instead of strings. This commonly happens with loop devices, empty devices, or certain virtualized environments.

**Error Example (Fixed):**
```
Error: Device discovery failed: invalid type: integer `0`, expected a string at line 6 column 18
```

**Solution:** Added custom deserializer that accepts both string and integer values for the size field.

## CLI Testing on Linux

### Basic Device Discovery
```bash
# JSON output (default)
./target/debug/securewipe discover

# Human-readable output
./target/debug/securewipe discover --format human

# Skip device enrichment (faster)
./target/debug/securewipe discover --no-enrich

# Help
./target/debug/securewipe discover --help
```

### Testing in Docker Containers

You can test the device discovery functionality using Docker:

```bash
# Start an Ubuntu container with the project mounted
docker run -it -v ~/Projects/SIH:/workspace ubuntu bash

# Inside the container:
cd /workspace/erase-sure/core

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required packages
apt update && apt install -y util-linux build-essential pkg-config

# Build and test
cargo build
./target/debug/securewipe discover --no-enrich
```

**Note:** In Docker containers, you'll typically see loop devices and virtual filesystems, which is perfect for testing the size field handling.

### ✅ Verified Working on Linux (Ubuntu Docker)

**Test Results from Ubuntu Docker Container:**
- ✅ **18 devices discovered** - Including NBD devices and VIRTIO disks
- ✅ **Risk classification working** - Correctly identified HIGH risk for mounted device (`/dev/vda` with `/etc/hosts`)
- ✅ **Size field handling** - Properly handled integer `0` values for NBD devices
- ✅ **Bus type detection** - Correctly identified VIRTIO bus type
- ✅ **Mountpoint detection** - Found container-specific mounts like `/etc/hosts`
- ✅ **JSON and human formats** - Both output formats working perfectly
- ✅ **Enrichment options** - Both `--no-enrich` and default enrichment modes working

**Sample Output:**
```json
{
  "name": "/dev/vda",
  "model": null,
  "serial": null,
  "capacity_bytes": 1099511627776,
  "bus": "VIRTIO",
  "mountpoints": ["/etc/hosts"],
  "risk_level": "HIGH"
}
```

### Expected Output Structure

#### JSON Format
```json
[
  {
    "name": "/dev/sda",
    "model": "Samsung SSD 980 1TB",
    "serial": "S649NX0R123456A",
    "capacity_bytes": 1000204886016,
    "bus": "NVMe",
    "mountpoints": ["/boot/efi", "/"],
    "risk_level": "CRITICAL"
  },
  {
    "name": "/dev/sdb",
    "model": "WD20EZRZ-00Z5HB0", 
    "serial": "WD-WCC4N7ABCDEF",
    "capacity_bytes": 2000398934016,
    "bus": "SATA",
    "mountpoints": ["/home"],
    "risk_level": "HIGH"
  },
  {
    "name": "/dev/sdc",
    "model": "SanDisk Ultra",
    "serial": "4C530001171122115172",
    "capacity_bytes": 32017047552,
    "bus": "USB",
    "mountpoints": [],
    "risk_level": "SAFE"
  }
]
```

#### Human Format
```
Device: /dev/sda
  Model: Samsung SSD 980 1TB
  Serial: S649NX0R123456A
  Capacity: 1000204886016 bytes
  Bus: NVMe
  Risk Level: Critical
  Mountpoints: /boot/efi, /

Device: /dev/sdc
  Model: SanDisk Ultra
  Serial: 4C530001171122115172
  Capacity: 32017047552 bytes
  Bus: USB
  Risk Level: Safe
```

## Risk Level Classification

- **CRITICAL**: Device contains the root filesystem ("/")
- **HIGH**: Device has mounted writable volumes (excluding system paths like /sys, /proc, /dev, /run, /boot/efi)
- **SAFE**: Device is unmounted or contains only read-only/system mounts

## Testing Different Scenarios

### Test with Various Device Types
1. **System Disk** - Should show as CRITICAL (contains "/")
2. **Data Disk** - Should show as HIGH if mounted, SAFE if unmounted
3. **USB/External** - Usually shows as SAFE unless mounted
4. **NVMe vs SATA vs USB** - Bus type should be correctly identified

### Test Enrichment
- With enrichment: `./target/debug/securewipe discover`
- Without enrichment: `./target/debug/securewipe discover --no-enrich`

The enrichment attempts to gather additional device info from:
- `smartctl -i` for most drives
- `hdparm -I` for SATA devices
- `nvme id-ctrl` for NVMe devices

All enrichment is **read-only** and **non-destructive**.

## Troubleshooting

### Command Not Found
```
Error: lsblk command not found - this tool requires Linux with util-linux package
```
- Install `util-linux` package
- Ensure you're running on a Linux system

### Permission Issues
The tool runs with user permissions and doesn't require sudo for device discovery.

### Missing Enrichment Tools
If `smartctl`, `hdparm`, or `nvme` tools are missing, the discovery will continue but skip enrichment. Use `--no-enrich` to disable enrichment entirely.

## Integration with SecureWipe

The device discovery output feeds into:
1. **Backup operations** - Select source devices safely
2. **Wipe operations** - Identify which devices are safe to wipe (SAFE) vs dangerous (CRITICAL/HIGH)
3. **Risk assessment** - Prevent accidental system disk wiping

The CRITICAL risk level acts as a safety guard against wiping system disks unless explicit override flags are provided.
