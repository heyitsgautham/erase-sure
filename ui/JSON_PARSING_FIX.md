# JSON Parsing Fix - Testing Guide

## Issue Summary
The application was failing when scanning devices because the CLI output format includes both NDJSON log lines and JSON arrays spanning multiple lines. The previous parsing logic tried to parse each line individually, which failed for multi-line JSON.

## Root Cause
1. **Mixed Output Format**: CLI outputs both log lines (NDJSON) and result data (JSON array)
2. **Multi-line JSON**: The device array spans multiple lines, not parseable line-by-line
3. **Stale Closure**: React hook was using stale state variables in async Promise resolution
4. **Error Propagation**: Unhandled parsing errors were breaking React rendering

## Fixes Applied

### 1. Enhanced JSON Parsing (`parseJsonOutput` function)
- Filters out NDJSON log lines (containing `level` and `timestamp` fields)
- Joins non-log lines to reconstruct multi-line JSON
- Fallback parsing for single-line JSON objects
- Proper error handling with descriptive messages

### 2. Fixed React Hook State Management
- Collects logs directly in Promise scope to avoid stale closures  
- Sets up dedicated event listeners per session
- Proper cleanup of listeners on completion/timeout
- Better error boundary handling

### 3. Enhanced Error Handling
- Added console logging for debugging
- Clear error messages to prevent white screen crashes
- Graceful fallback with empty device list on errors
- Proper state cleanup on failures

### 4. CLI Output Format Mapping
Maps CLI field names to UI interface:
- `name` → `path`
- `capacity_bytes` → `capacity`
- Auto-blocks CRITICAL devices
- Handles null/undefined values gracefully

## Testing Steps

### 1. Start the Application
```bash
cd /home/kinux/projects/erase-sure/ui
npm run tauri dev
```

### 2. Test Device Discovery
1. Navigate to "Discover" screen
2. Click "🔍 Scan Devices" 
3. **Expected**: Should show devices without errors
4. Click "🔍 Scan Devices" again
5. **Expected**: Should work repeatedly without white screen

### 3. Check Console Output
Open developer tools and check console for:
- `CLI stdout:` and `CLI stderr:` debug logs
- No parsing errors
- Proper device mapping

### 4. Verify Device Display
- CRITICAL devices (with `/` mount) should be blocked
- HIGH devices (swap) should be selectable but warned
- Device details should show correct model, capacity, etc.

## CLI Output Format (Reference)
```
{"level":"info","message":"Starting device discovery","timestamp":"2025-09-16T16:52:10.173781055+00:00"}
{"level":"info","message":"Found 2 devices","timestamp":"2025-09-16T16:52:10.209419531+00:00"}
[
  {
    "name": "/dev/zram0",
    "model": null,
    "serial": null,
    "capacity_bytes": 4294967296,
    "bus": null,
    "mountpoints": ["[SWAP]"],
    "risk_level": "HIGH"
  },
  {
    "name": "/dev/nvme0n1", 
    "model": "WD PC SN740 SDDQMQD-512G-1001",
    "serial": "232367403051",
    "capacity_bytes": 512110190592,
    "bus": "NVMe",
    "mountpoints": ["/boot", "/"],
    "risk_level": "CRITICAL"
  }
]
```

## Success Criteria
- ✅ First device scan works and shows devices
- ✅ Second device scan works without white screen
- ✅ CRITICAL devices are properly blocked
- ✅ Error messages are user-friendly
- ✅ Console shows debug information
- ✅ No React rendering crashes

## If Issues Persist
1. Check browser console for specific errors
2. Verify `securewipe` CLI is in PATH
3. Test CLI manually: `securewipe discover --format json`
4. Check Tauri backend logs in terminal
