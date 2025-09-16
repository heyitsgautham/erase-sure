#!/bin/bash

# Test script for SecureWipe backup functionality
set -e

echo "Setting up test environment..."

# Create test directories
TEST_DIR="/tmp/securewipe_test"
SOURCE_DIR="$TEST_DIR/source"
DEST_DIR="$TEST_DIR/backup"

rm -rf "$TEST_DIR"
mkdir -p "$SOURCE_DIR/Documents"
mkdir -p "$SOURCE_DIR/Pictures"
mkdir -p "$DEST_DIR"

# Create test files
echo "Creating test files..."
echo "This is a test document" > "$SOURCE_DIR/Documents/test1.txt"
echo "Another test document with more content" > "$SOURCE_DIR/Documents/test2.txt"
echo "Picture metadata" > "$SOURCE_DIR/Pictures/photo1.jpg"

# Build the core binary
echo "Building SecureWipe core..."
cd /Users/gauthamkrishna/Projects/SIH/erase-sure/core
cargo build --release

# Test backup command
echo "Testing backup command..."
./target/release/securewipe-core backup \
    --device "/dev/test_device" \
    --dest "$DEST_DIR" \
    --paths "$SOURCE_DIR/Documents,$SOURCE_DIR/Pictures"

echo "Backup test completed!"

# Verify results
echo "Verifying backup results..."
BACKUP_ID=$(ls "$DEST_DIR" | head -1)
echo "Backup ID: $BACKUP_ID"

if [ -f "$DEST_DIR/$BACKUP_ID/manifest.json" ]; then
    echo "✓ Manifest file created"
    cat "$DEST_DIR/$BACKUP_ID/manifest.json" | jq .
else
    echo "✗ Manifest file missing"
fi

if [ -d "$HOME/SecureWipe/certificates" ]; then
    echo "✓ Certificate directory exists"
    ls -la "$HOME/SecureWipe/certificates"
else
    echo "✗ Certificate directory missing"
fi

echo "Test completed successfully!"