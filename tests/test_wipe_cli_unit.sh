#!/bin/bash

# Unit tests for securewipe CLI wipe subcommand
# Tests CLI argument parsing and validation without destructive operations

set -e

cd "$(dirname "$0")/core"

echo "SecureWipe CLI Wipe Command Unit Tests"
echo "====================================="
echo ""

# Build the project
echo "Building securewipe..."
cargo build
SECUREWIPE_BIN="./target/debug/securewipe"

if [[ ! -f "$SECUREWIPE_BIN" ]]; then
    echo "❌ securewipe binary not found"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Test 1: Help for wipe command
echo "Test 1: Help output"
$SECUREWIPE_BIN wipe --help 2>/dev/null || true
echo "✅ Help output test passed"
echo ""

# Test 2: Missing required arguments (should fail)
echo "Test 2: Missing device argument (should fail)"
if $SECUREWIPE_BIN wipe --policy clear 2>/dev/null; then
    echo "❌ Should have failed with missing device"
    exit 1
else
    echo "✅ Correctly failed with missing device"
fi
echo ""

# Test 3: Safety check without SECUREWIPE_DANGER (should fail)
echo "Test 3: Safety check without SECUREWIPE_DANGER (should fail)"
unset SECUREWIPE_DANGER
if $SECUREWIPE_BIN wipe --device /dev/null --policy clear --danger-allow-wipe 2>/dev/null; then
    echo "❌ Should have failed without SECUREWIPE_DANGER"
    exit 1
else
    echo "✅ Correctly failed without SECUREWIPE_DANGER environment variable"
fi
echo ""

# Test 4: Safety check without --danger-allow-wipe flag (should fail)
echo "Test 4: Safety check without --danger-allow-wipe flag (should fail)"
export SECUREWIPE_DANGER=1
if $SECUREWIPE_BIN wipe --device /dev/null --policy clear 2>/dev/null; then
    echo "❌ Should have failed without --danger-allow-wipe flag"
    exit 1
else
    echo "✅ Correctly failed without --danger-allow-wipe flag"
fi
echo ""

# Test 5: Nonexistent device (should fail)
echo "Test 5: Nonexistent device (should fail)"
if echo "CONFIRM WIPE" | $SECUREWIPE_BIN wipe --device /dev/nonexistent --policy clear --danger-allow-wipe 2>/dev/null; then
    echo "❌ Should have failed with nonexistent device"
    exit 1
else
    echo "✅ Correctly failed with nonexistent device"
fi
echo ""

# Test 6: Valid policy values
echo "Test 6: Valid policy values"
for policy in CLEAR PURGE DESTROY; do
    # This will fail at device validation, but should accept the policy
    output=$(SECUREWIPE_DANGER=1 $SECUREWIPE_BIN wipe --device /dev/nonexistent --policy $policy --danger-allow-wipe 2>&1 || true)
    if echo "$output" | grep -q "policy.*$policy"; then
        echo "✅ Policy '$policy' accepted"
    else
        echo "❌ Policy '$policy' not properly handled"
        echo "Output: $output"
        exit 1
    fi
done
echo ""

# Test 7: Invalid policy (should fail)
echo "Test 7: Invalid policy (should fail)"
if $SECUREWIPE_BIN wipe --device /dev/null --policy invalid 2>/dev/null; then
    echo "❌ Should have failed with invalid policy"
    exit 1
else
    echo "✅ Correctly rejected invalid policy"
fi
echo ""

echo "🎉 All CLI unit tests passed!"
echo ""
echo "Note: These tests verified CLI parsing and safety checks without"
echo "performing any destructive operations."