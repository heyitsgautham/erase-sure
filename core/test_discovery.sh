#!/bin/bash

# SecureWipe Device Discovery Test Script
# This script tests various scenarios for device discovery

set -e

echo "=== SecureWipe Device Discovery Tests ==="
echo

# Build the project
echo "Building SecureWipe..."
cargo build --release
echo "✓ Build completed"
echo

BINARY="./target/release/securewipe"

# Test 1: Basic device discovery
echo "Test 1: Basic device discovery (JSON output)"
echo "Command: $BINARY discover"
if $BINARY discover > /tmp/discover_test.json 2>/tmp/discover_test.log; then
    echo "✓ Device discovery succeeded"
    echo "Found devices:"
    cat /tmp/discover_test.json | jq -r '.[] | "  - \(.name): \(.model // "Unknown") (\(.risk_level))"' 2>/dev/null || echo "  (jq not available for formatting)"
else
    echo "✗ Device discovery failed"
    echo "Error log:"
    cat /tmp/discover_test.log
fi
echo

# Test 2: Human-readable output
echo "Test 2: Human-readable output"
echo "Command: $BINARY discover --format human"
if $BINARY discover --format human > /tmp/discover_human.txt 2>/tmp/discover_human.log; then
    echo "✓ Human-readable output succeeded"
    echo "Sample output:"
    head -20 /tmp/discover_human.txt | sed 's/^/  /'
else
    echo "✗ Human-readable output failed"
    cat /tmp/discover_human.log
fi
echo

# Test 3: No enrichment mode
echo "Test 3: No enrichment mode (faster)"
echo "Command: $BINARY discover --no-enrich"
if $BINARY discover --no-enrich > /tmp/discover_no_enrich.json 2>/tmp/discover_no_enrich.log; then
    echo "✓ No-enrichment mode succeeded"
    device_count=$(cat /tmp/discover_no_enrich.json | jq '. | length' 2>/dev/null || echo "unknown")
    echo "  Found $device_count devices"
else
    echo "✗ No-enrichment mode failed"
    cat /tmp/discover_no_enrich.log
fi
echo

# Test 4: Invalid arguments
echo "Test 4: Invalid arguments handling"
echo "Command: $BINARY discover --invalid-flag"
if $BINARY discover --invalid-flag 2>/tmp/invalid_test.log; then
    echo "✗ Should have failed with invalid arguments"
else
    echo "✓ Correctly rejected invalid arguments"
    echo "Error message:"
    head -3 /tmp/invalid_test.log | sed 's/^/  /'
fi
echo

# Test 5: Help system
echo "Test 5: Help system"
echo "Command: $BINARY discover --help"
if $BINARY discover --help > /tmp/help_test.txt; then
    echo "✓ Help system works"
    echo "Help content preview:"
    head -10 /tmp/help_test.txt | sed 's/^/  /'
else
    echo "✗ Help system failed"
fi
echo

# Test 6: Risk level validation
echo "Test 6: Risk level validation"
if [ -f /tmp/discover_test.json ]; then
    echo "Checking risk levels in discovered devices:"
    
    # Count devices by risk level
    critical_count=$(cat /tmp/discover_test.json | jq '[.[] | select(.risk_level == "CRITICAL")] | length' 2>/dev/null || echo "0")
    high_count=$(cat /tmp/discover_test.json | jq '[.[] | select(.risk_level == "HIGH")] | length' 2>/dev/null || echo "0")
    safe_count=$(cat /tmp/discover_test.json | jq '[.[] | select(.risk_level == "SAFE")] | length' 2>/dev/null || echo "0")
    
    echo "  CRITICAL devices: $critical_count"
    echo "  HIGH risk devices: $high_count"
    echo "  SAFE devices: $safe_count"
    
    if [ "$critical_count" -gt 0 ]; then
        echo "  ⚠️  CRITICAL devices found (system disks)"
    else
        echo "  ✓ No CRITICAL devices (or running in container/VM)"
    fi
else
    echo "  Cannot validate - no discovery output available"
fi
echo

# Test 7: JSON structure validation
echo "Test 7: JSON structure validation"
if [ -f /tmp/discover_test.json ]; then
    if cat /tmp/discover_test.json | jq empty 2>/dev/null; then
        echo "✓ Output is valid JSON"
        
        # Check required fields
        required_fields=("name" "capacity_bytes" "risk_level")
        for field in "${required_fields[@]}"; do
            if cat /tmp/discover_test.json | jq -e ".[0].$field" >/dev/null 2>&1; then
                echo "  ✓ Field '$field' present"
            else
                echo "  ⚠️  Field '$field' missing or null"
            fi
        done
    else
        echo "✗ Output is not valid JSON"
    fi
else
    echo "  Cannot validate - no JSON output available"
fi
echo

# Cleanup
echo "Cleaning up test files..."
rm -f /tmp/discover_*.json /tmp/discover_*.txt /tmp/discover_*.log /tmp/invalid_test.log /tmp/help_test.txt
echo "✓ Cleanup completed"
echo

echo "=== Test Summary ==="
echo "Device discovery implementation tested with:"
echo "- JSON and human-readable output formats"
echo "- Enrichment and no-enrichment modes"
echo "- Error handling and help system"
echo "- Risk level classification"
echo "- JSON structure validation"
echo
echo "Check the output above for any failures (marked with ✗)"
echo "All tests with ✓ marks passed successfully"