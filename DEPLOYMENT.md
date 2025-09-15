# SecureWipe Deployment Guide

## Platform Requirements

SecureWipe is designed for **Linux environments** and requires specific system tools for device discovery and secure wiping operations.

### Required Linux Tools
- `lsblk` - Block device listing (part of util-linux)
- `hdparm` - ATA/SATA device management 
- `nvme-cli` - NVMe device management
- `smartctl` - SMART monitoring tools (smartmontools)

## Development vs Production

### Development Environment (Any OS)
- **UI Development**: Tauri + React can be developed on macOS/Windows/Linux
- **Backend Testing**: Rust compilation works on any platform
- **Limited Functionality**: Device discovery will show platform errors on non-Linux

### Production Environment (Linux Required)
- **Real Device Discovery**: Requires Linux with proper tools
- **Secure Wiping**: NIST-compliant operations need hardware access
- **Certificate Generation**: Full functionality available

## Platform-Specific Setup

### Linux (Ubuntu/Debian)
```bash
# Install system dependencies
sudo apt update
sudo apt install -y lsblk hdparm nvme-cli smartmontools

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build SecureWipe CLI
cd core
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify installation
securewipe --help
```

### Arch Linux
```bash
# Install system dependencies
sudo pacman -S util-linux hdparm nvme-cli smartmontools

# Install Rust (if needed)
sudo pacman -S rustup
rustup default stable

# Build SecureWipe CLI
cd core
cargo build --release

# Add to PATH (add to ~/.zshrc or ~/.bashrc for persistence)
export PATH="$PWD/target/release:$PATH"

# Verify installation
securewipe --help
```

### RHEL/CentOS/Fedora
```bash
# Install system dependencies
sudo dnf install -y util-linux hdparm nvme-cli smartmontools

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build SecureWipe CLI
cd core
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify installation
securewipe --help
```

### macOS/Windows (Development Only)
```bash
# Install Rust
# macOS: brew install rustup-init && rustup-init
# Windows: Download from rustup.rs

# Build CLI (will compile but device discovery won't work)
cd core
cargo build --release

# UI development works normally
cd ../ui
npm install
npm run tauri dev
```

## Testing Real Device Discovery

### Prerequisites
1. Linux environment with required tools
2. SecureWipe CLI built and in PATH
3. Proper permissions for device access

### Test Commands
```bash
# Test device discovery
securewipe discover --format json

# Test backup planning (safe, no actual backup)
securewipe backup --device /dev/sdX --dest ~/test --dry-run

# Test wipe planning (safe, no actual wiping)
securewipe wipe --device /dev/sdX --method CLEAR --dry-run
```

### UI Testing
1. Start Tauri app: `cd ui && npm run tauri dev`
2. Open SecureWipe Test component
3. Click "Discover Devices" to see real device data
4. Use other buttons to test backup/wipe planning

## Security Considerations

### Development Safety
- UI blocks destructive flags (`--apply`, `--execute`, `--force`)
- Only planning operations are exposed
- Real wiping requires direct CLI usage with explicit flags

### Production Safety
- Always test on non-critical devices first
- Use `--dry-run` flags for planning
- Verify device identification before destructive operations
- Keep backup certificates for audit trails

## Troubleshooting

### "Command not found" Error
- Ensure SecureWipe CLI is built: `cargo build --release`
- Check PATH includes: `echo $PATH | grep target/release`
- Verify binary exists: `ls core/target/release/securewipe`

### "Platform not supported" Error
- SecureWipe requires Linux for device operations
- Use Linux VM or WSL for testing on other platforms
- Development UI will show helpful error messages

### Permission Errors
- Device access may require sudo: `sudo securewipe discover`
- Consider adding user to disk group: `sudo usermod -a -G disk $USER`
- Log out and back in for group changes to take effect

### Missing Tools Error
- Install required packages for your distribution
- Verify tools work: `lsblk --version`, `hdparm --version`
- Some tools may need manual installation on minimal systems

## Deployment Checklist

- [ ] Linux environment confirmed
- [ ] Required tools installed (lsblk, hdparm, nvme-cli, smartctl)
- [ ] Rust toolchain installed
- [ ] SecureWipe CLI built (`cargo build --release`)
- [ ] CLI in PATH and `securewipe --help` works
- [ ] Device discovery tested (`securewipe discover`)
- [ ] UI can communicate with CLI backend
- [ ] Backup/wipe planning tested (dry-run mode)
- [ ] Certificates generate and validate properly