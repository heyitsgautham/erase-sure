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

cat > test_backup_run.rs << 'EOF'
use securewipe::backup::{EncryptedBackup, BackupOperations};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backup_engine = EncryptedBackup::new();
    
    let source_paths = vec![
        std::env::args().nth(1).expect("Need source path")
    ];
    let dest = std::env::args().nth(2).expect("Need dest path");
    
    println!("Starting backup...");
    let result = backup_engine.perform_backup("/dev/test", &source_paths, &dest)?;
    
    println!("✓ Backup completed successfully!");
    println!("  Backup ID: {}", result.backup_id);
    println!("  Files: {}", result.manifest.total_files);
    println!("  Bytes: {}", result.manifest.total_bytes);
    println!("  Verification: {}", if result.verification_passed { "PASSED" } else { "FAILED" });
    
    Ok(())
}
EOF

# Compile and run the test
rustc --edition 2021 -L target/debug/deps test_backup_run.rs -o test_backup_run --extern securewipe=target/debug/libsecurewipe.rlib

echo "Running backup test..."
./test_backup_run "$SOURCE_DIR/Documents" "$DEST_DIR"

# Verify results
if [ -d "$DEST_DIR" ]; then
    echo "✓ Backup directory created"
    ls -la "$DEST_DIR"
else
    echo "✗ Backup directory not found"
fi

# Cleanup
rm -rf "$TEST_DIR" test_backup_run.rs test_backup_run

echo "Test completed!"