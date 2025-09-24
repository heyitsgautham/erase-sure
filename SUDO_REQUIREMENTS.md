# Sudo Requirements for SecureWipe

## Overview

SecureWipe uses **selective privilege escalation** - only operations that require root access use sudo, while safe operations run with normal user permissions.

## Operations and Permission Requirements

### ✅ No Sudo Required (Normal User)
- **Device Discovery**: Reading device information (`lsblk`, `smartctl`)
- **Backup Operations**: Creating encrypted backups to user-accessible locations
- **Wipe Planning**: Analyzing devices and generating wipe plans
- **Certificate Operations**: Signing, verifying, and managing certificates

### 🔐 Sudo Required (Root Access)
- **Destructive Wipe Execution**: Writing to raw block devices requires root permissions

## How It Works

1. **GUI Application**: 
   - Runs as normal user by default
   - Automatically uses `sudo` only for destructive wipe operations
   - Prompts for password when needed

2. **CLI Application**:
   - Most commands work without sudo: `./securewipe discover`, `./securewipe backup`
   - Only destructive wipes need sudo: `sudo ./securewipe wipe --device /dev/sdX --danger-allow-wipe`

## Security Benefits

- **Principle of Least Privilege**: Only escalates when absolutely necessary
- **Reduced Attack Surface**: Most operations run with normal user permissions
- **Clear Separation**: Destructive operations are clearly marked and require confirmation

## User Experience

- No need to run the entire GUI as root
- Password prompt only appears when performing actual wipe operations
- Discovery and backup work immediately without any permission prompts

## Troubleshooting

If you get "Permission denied" errors:

1. **For Discovery/Backup**: Should work without sudo - check file permissions
2. **For Wipe Operations**: This is expected behavior - the system will prompt for sudo password
3. **GUI Not Prompting**: Ensure `sudo` is installed and configured for GUI applications

## Technical Implementation

The application detects destructive operations by checking for the `--danger-allow-wipe` flag and automatically prepends `sudo` to the command execution.