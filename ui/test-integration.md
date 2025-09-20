# Tauri SecureWipe Integration Test

## Overview
This document provides testing steps for the Tauri process bridge and React hook integration.

## Prerequisites
1. Rust CLI binary `securewipe` should be available in PATH or in the same directory
2. Tauri dev environment should be set up
3. UI should be built successfully

## Testing Steps

### 1. Start the Tauri Development Environment
```bash
cd /home/kinux/projects/erase-sure/ui
npm run tauri dev
```

### 2. Test Device Discovery
1. Navigate to the "Discover" screen
2. Click "🔍 Scan Devices"
3. Verify that:
   - The button shows loading state
   - Logs appear in real-time (if CLI is present)
   - Device list is populated or error is shown
   - No UI freezing occurs

### 3. Test Wipe Planning
1. Select a SAFE device from the discovery list
2. Click "📋 View Wipe Plan"
3. Verify that:
   - Wipe plan analysis runs
   - Logs stream in real-time
   - Plan summary is displayed
   - No destructive operations are allowed

### 4. Test Backup Operation
1. From a selected device, click "📦 Continue to Backup"
2. Configure backup destination
3. Click "🛡️ Run Backup (Encrypted)"
4. Verify that:
   - Progress indicator appears
   - Logs stream with timestamps
   - Process can be cancelled if needed
   - Certificate paths are displayed on completion

### 5. Test Error Handling
1. Try operations without CLI binary present
2. Verify error messages are clear
3. Test cancellation functionality
4. Verify forbidden arguments are blocked

## Expected Behavior

### Security Validations
- ✅ Only whitelisted subcommands allowed: discover, wipe, backup, cert
- ✅ Destructive flags are blocked: --apply, --execute, --force, etc.
- ✅ Wipe command only allows planning flags
- ✅ Process timeout after 20 minutes
- ✅ Output truncation for oversized lines

### UI Integration
- ✅ Real-time log streaming with timestamps
- ✅ Process status indicators
- ✅ Error handling with user-friendly messages
- ✅ Proper loading states
- ✅ Log viewer with copy functionality

### Backend Security
- ✅ Process isolation with session IDs
- ✅ Argument sanitization
- ✅ Graceful process termination
- ✅ No private key exposure

## Fallback Testing
If CLI binary is not available, the system should:
1. Show clear error messages
2. Not crash the UI
3. Allow navigation between screens
4. Display mock data where appropriate

## Success Criteria
- [x] Tauri backend compiles without errors
- [x] Frontend builds successfully
- [x] Real CLI integration works when binary is present
- [x] Error handling works when binary is missing
- [x] Security validations prevent dangerous operations
- [x] UI remains responsive during operations
- [x] Logs stream with proper formatting and timestamps
