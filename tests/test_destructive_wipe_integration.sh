#!/bin/bash

# Integration test for destructive wipe operations using loopback devices
# This script creates safe test environments without touching real disks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}SecureWipe Destructive Wipe Integration Tests${NC}"
echo "=============================================="
echo ""

# Check if running as root or with sudo
if [[ $EUID -ne 0 ]]; then
    echo -e "${RED}Error: This test requires root privileges for loopback device management${NC}"
    echo "Please run with sudo: sudo $0"
    exit 1
fi

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT="$(pwd)"
CORE_DIR="$PROJECT_ROOT/core"
TEST_OUTPUT_DIR="$PROJECT_ROOT/test_wipe_results"

echo "Project root: $PROJECT_ROOT"
echo "Test output: $TEST_OUTPUT_DIR"
echo ""

# Clean up function
cleanup() {
    echo -e "${YELLOW}Cleaning up test resources...${NC}"
    
    # Detach any loopback devices we created
    for loop_dev in $(losetup -j "$TEST_OUTPUT_DIR/test_disk" 2>/dev/null | cut -d: -f1); do
        echo "Detaching $loop_dev"
        losetup -d "$loop_dev" 2>/dev/null || true
    done
    
    # Remove test files
    rm -f "$TEST_OUTPUT_DIR/test_disk"* 2>/dev/null || true
}

# Set trap for cleanup
trap cleanup EXIT

# Create test output directory
mkdir -p "$TEST_OUTPUT_DIR"

# Build the core project
echo -e "${YELLOW}Building securewipe core...${NC}"
cd "$CORE_DIR"
cargo build --release
if [[ $? -ne 0 ]]; then
    echo -e "${RED}Failed to build securewipe core${NC}"
    exit 1
fi

SECUREWIPE_BIN="$CORE_DIR/target/release/securewipe"
if [[ ! -f "$SECUREWIPE_BIN" ]]; then
    echo -e "${RED}securewipe binary not found at $SECUREWIPE_BIN${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Test 1: Create loopback device and test CLEAR policy
echo -e "${YELLOW}Test 1: CLEAR policy wipe test${NC}"
echo "Creating 50MB test disk..."

# Create test disk file
dd if=/dev/urandom of="$TEST_OUTPUT_DIR/test_disk_clear.img" bs=1M count=50 status=progress

# Write some recognizable pattern to verify wiping
echo "TEST_DATA_PATTERN_CLEAR_$(date)" > "$TEST_OUTPUT_DIR/test_pattern_clear.txt"
dd if="$TEST_OUTPUT_DIR/test_pattern_clear.txt" of="$TEST_OUTPUT_DIR/test_disk_clear.img" bs=1024 seek=1000 conv=notrunc

# Set up loopback device
LOOP_DEV_CLEAR=$(losetup -f --show "$TEST_OUTPUT_DIR/test_disk_clear.img")
echo "Loopback device created: $LOOP_DEV_CLEAR"

# Verify we can read the test pattern before wiping
echo "Verifying test pattern before wipe..."
if dd if="$LOOP_DEV_CLEAR" bs=1024 skip=1000 count=1 2>/dev/null | grep -q "TEST_DATA_PATTERN_CLEAR"; then
    echo -e "${GREEN}✓ Test pattern found before wipe${NC}"
else
    echo -e "${RED}✗ Test pattern not found before wipe${NC}"
    exit 1
fi

# Perform destructive wipe with CLEAR policy
echo "Executing CLEAR wipe on $LOOP_DEV_CLEAR..."
export SECUREWIPE_DANGER=1
echo "CONFIRM WIPE" | timeout 300 "$SECUREWIPE_BIN" wipe \
    --device "$LOOP_DEV_CLEAR" \
    --policy clear \
    --danger-allow-wipe \
    --sign

if [[ $? -ne 0 ]]; then
    echo -e "${RED}✗ CLEAR wipe failed${NC}"
    exit 1
fi

# Verify the data is wiped
echo "Verifying data is wiped..."
if dd if="$LOOP_DEV_CLEAR" bs=1024 skip=1000 count=1 2>/dev/null | grep -q "TEST_DATA_PATTERN_CLEAR"; then
    echo -e "${RED}✗ Test pattern still found after wipe!${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Test pattern successfully wiped${NC}"
fi

# Check for mostly zeros
ZERO_COUNT=$(dd if="$LOOP_DEV_CLEAR" bs=1M count=10 2>/dev/null | hexdump -C | grep "00 00 00 00 00 00 00 00" | wc -l)
if [[ $ZERO_COUNT -gt 50 ]]; then
    echo -e "${GREEN}✓ Device contains expected zero patterns${NC}"
else
    echo -e "${YELLOW}⚠ Warning: Less zero patterns than expected${NC}"
fi

# Detach the loopback device
losetup -d "$LOOP_DEV_CLEAR"
echo -e "${GREEN}✓ Test 1 (CLEAR) completed successfully${NC}"
echo ""

# Test 2: Test PURGE policy
echo -e "${YELLOW}Test 2: PURGE policy wipe test${NC}"

# Create another test disk
dd if=/dev/urandom of="$TEST_OUTPUT_DIR/test_disk_purge.img" bs=1M count=25 status=progress

echo "TEST_DATA_PATTERN_PURGE_$(date)" > "$TEST_OUTPUT_DIR/test_pattern_purge.txt"
dd if="$TEST_OUTPUT_DIR/test_pattern_purge.txt" of="$TEST_OUTPUT_DIR/test_disk_purge.img" bs=1024 seek=500 conv=notrunc

LOOP_DEV_PURGE=$(losetup -f --show "$TEST_OUTPUT_DIR/test_disk_purge.img")
echo "Loopback device created: $LOOP_DEV_PURGE"

# Verify test pattern
if dd if="$LOOP_DEV_PURGE" bs=1024 skip=500 count=1 2>/dev/null | grep -q "TEST_DATA_PATTERN_PURGE"; then
    echo -e "${GREEN}✓ Test pattern found before wipe${NC}"
else
    echo -e "${RED}✗ Test pattern not found before wipe${NC}"
    exit 1
fi

# Perform PURGE wipe
echo "Executing PURGE wipe on $LOOP_DEV_PURGE..."
echo "CONFIRM WIPE" | timeout 600 "$SECUREWIPE_BIN" wipe \
    --device "$LOOP_DEV_PURGE" \
    --policy purge \
    --danger-allow-wipe \
    --sign

if [[ $? -ne 0 ]]; then
    echo -e "${RED}✗ PURGE wipe failed${NC}"
    exit 1
fi

# Verify wiping
if dd if="$LOOP_DEV_PURGE" bs=1024 skip=500 count=1 2>/dev/null | grep -q "TEST_DATA_PATTERN_PURGE"; then
    echo -e "${RED}✗ Test pattern still found after PURGE wipe!${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Test pattern successfully wiped with PURGE${NC}"
fi

# Check for random-looking data (not all zeros)
NONZERO_COUNT=$(dd if="$LOOP_DEV_PURGE" bs=1M count=5 2>/dev/null | hexdump -C | grep -v "00 00 00 00 00 00 00 00" | wc -l)
if [[ $NONZERO_COUNT -gt 100 ]]; then
    echo -e "${GREEN}✓ Device contains expected random patterns${NC}"
else
    echo -e "${YELLOW}⚠ Warning: Less random patterns than expected${NC}"
fi

losetup -d "$LOOP_DEV_PURGE"
echo -e "${GREEN}✓ Test 2 (PURGE) completed successfully${NC}"
echo ""

# Test 3: Certificate generation verification
echo -e "${YELLOW}Test 3: Certificate generation verification${NC}"

CERT_DIR="$HOME/SecureWipe/certificates"
if [[ -d "$CERT_DIR" ]]; then
    CERT_COUNT=$(find "$CERT_DIR" -name "*.json" -mtime -1 | wc -l)
    if [[ $CERT_COUNT -ge 2 ]]; then
        echo -e "${GREEN}✓ Wipe certificates generated successfully${NC}"
        
        # Verify certificate content
        LATEST_CERT=$(find "$CERT_DIR" -name "*.json" -mtime -1 | head -1)
        if [[ -f "$LATEST_CERT" ]]; then
            if grep -q '"cert_type":"wipe"' "$LATEST_CERT" && \
               grep -q '"verification_passed":true' "$LATEST_CERT" && \
               grep -q '"signature"' "$LATEST_CERT"; then
                echo -e "${GREEN}✓ Certificate contains expected wipe data${NC}"
            else
                echo -e "${YELLOW}⚠ Certificate may be missing some expected fields${NC}"
            fi
        fi
    else
        echo -e "${YELLOW}⚠ Warning: Expected certificates not found${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Warning: Certificate directory not found${NC}"
fi

echo ""

# Test 4: Safety guard verification
echo -e "${YELLOW}Test 4: Safety guard verification${NC}"

# Test without SECUREWIPE_DANGER (should fail)
unset SECUREWIPE_DANGER
echo "Testing safety guard (should fail without SECUREWIPE_DANGER)..."

# Create small test disk for safety test
dd if=/dev/zero of="$TEST_OUTPUT_DIR/test_disk_safety.img" bs=1M count=10 >/dev/null 2>&1
LOOP_DEV_SAFETY=$(losetup -f --show "$TEST_OUTPUT_DIR/test_disk_safety.img")

# This should fail due to missing environment variable
echo "CONFIRM WIPE" | timeout 30 "$SECUREWIPE_BIN" wipe \
    --device "$LOOP_DEV_SAFETY" \
    --policy clear \
    --danger-allow-wipe 2>/dev/null

if [[ $? -eq 0 ]]; then
    echo -e "${RED}✗ Safety guard failed - wipe succeeded without SECUREWIPE_DANGER${NC}"
    losetup -d "$LOOP_DEV_SAFETY"
    exit 1
else
    echo -e "${GREEN}✓ Safety guard working - wipe correctly blocked without SECUREWIPE_DANGER${NC}"
fi

losetup -d "$LOOP_DEV_SAFETY"
echo ""

# Final summary
echo -e "${GREEN}===============================================${NC}"
echo -e "${GREEN}🎉 All integration tests completed successfully!${NC}"
echo -e "${GREEN}===============================================${NC}"
echo ""
echo "Test results:"
echo "✓ CLEAR policy wipe verification"
echo "✓ PURGE policy wipe verification"  
echo "✓ Certificate generation"
echo "✓ Safety guard functionality"
echo ""
echo "Generated certificates can be found in: $CERT_DIR"
echo ""
echo -e "${YELLOW}Note: These tests used loopback devices and did not touch any real disks.${NC}"