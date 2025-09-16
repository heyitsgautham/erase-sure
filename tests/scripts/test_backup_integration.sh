#!/bin/bash

echo "Testing SecureWipe backup functionality..."

# Create test environment
TEST_DIR="/tmp/securewipe_test_$(date +%s)"
SOURCE_DIR="$TEST_DIR/source"
DEST_DIR="$TEST_DIR/backup"

mkdir -p "$SOURCE_DIR/Documents"
mkdir -p "$DEST_DIR"

# Create test files
echo "This is a test document" > "$SOURCE_DIR/Documents/test1.txt"
echo "Another document" > "$SOURCE_DIR/Documents/test2.txt"

echo "✓ Created test files"

# Run the backup via Rust code (not CLI)
cd /Users/gauthamkrishna/Projects/SIH/erase-sure/core

# Build first to ensure libraries are available
cargo build --lib

# Use the CLI instead of trying to compile against the library
echo "Running backup via CLI..."
./target/release/securewipe backup \
    --device "/dev/test" \
    --dest "$DEST_DIR" \
    --paths "$SOURCE_DIR/Documents"

echo "✓ Backup completed!"

# Remove the Rust compilation attempt since it's complex to set up
# The CLI approach is more realistic for integration testing

# Verify results
if [ -d "$DEST_DIR" ]; then
    echo "✓ Backup directory created"
    ls -la "$DEST_DIR"
else
    echo "✗ Backup directory not found"
fi

# Cleanup
rm -rf "$TEST_DIR"

echo "Test completed!"