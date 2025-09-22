#!/bin/bash

# End-to-end test script for SecureWipe certificate handling enhancements
# Tests signing, verification, and schema validation

set -e  # Exit on any error

echo "============================================"
echo "SecureWipe Certificate Handling E2E Tests"
echo "============================================"

# Set up test environment
TEST_DIR="/tmp/securewipe_test_$(date +%s)"
SECUREWIPE_BIN="/home/kinux/projects/erase-sure/core/target/release/securewipe"
KEYS_DIR="/home/kinux/projects/erase-sure/keys"

mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "Test directory: $TEST_DIR"
echo "Using securewipe binary: $SECUREWIPE_BIN"

# Test 1: Create a valid certificate for signing
echo ""
echo "Test 1: Create valid unsigned certificate"
cat > valid_unsigned_cert.json << 'EOF'
{
  "cert_type": "backup",
  "cert_id": "test_e2e_backup_001",
  "certificate_version": "v1.0.0",
  "created_at": "2023-12-05T14:30:22.123456Z",
  "issuer": {
    "organization": "SecureWipe E2E Test",
    "tool_name": "securewipe",
    "tool_version": "v1.0.0",
    "country": "US"
  },
  "device": {
    "model": "Test SSD 1TB E2E",
    "serial": "E2E123456",
    "bus": "NVMe",
    "capacity_bytes": 1000000000000,
    "logical_block_size": 512,
    "total_lbas": 1953125000,
    "firmware": "E2E1.0",
    "path": "/dev/nvme0n1"
  },
  "files_summary": {
    "count": 50,
    "personal_bytes": 250000000,
    "included_paths": ["/home/testuser/Documents"],
    "excluded_paths": ["/home/testuser/.cache"]
  },
  "destination": {
    "type": "usb",
    "label": "E2E Test Drive",
    "fs": "exfat",
    "mountpoint": "/mnt/e2e_backup",
    "path": "/dev/sdb1"
  },
  "crypto": {
    "alg": "AES-256-CTR",
    "manifest_sha256": "e2e1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
    "key_management": "ephemeral_session_key"
  },
  "verification": {
    "strategy": "sampled_files",
    "samples": 3,
    "coverage": {
      "mode": "samples",
      "samples": 3
    },
    "failures": 0
  },
  "policy": {
    "name": "NIST SP 800-88 Rev.1",
    "version": "2023.12"
  },
  "result": "PASS",
  "environment": {
    "operator": "e2e_test_user",
    "os_kernel": "Linux 6.1.0",
    "tool_version": "v1.0.0"
  },
  "exceptions": {
    "text": "None"
  },
  "metadata": {
    "certificate_json_sha256": "e2e567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  }
}
EOF

echo "✓ Created valid unsigned certificate"

# Test 2: Validate unsigned certificate structure (should pass - we validate content, not signature presence)
echo ""
echo "Test 2: Validate unsigned certificate structure (should pass for structure)"
if $SECUREWIPE_BIN cert validate --file valid_unsigned_cert.json; then
    echo "✓ Unsigned certificate structure validation passed (signature not required for structure validation)"
else
    echo "✗ ERROR: Certificate structure should be valid even without signature"
    exit 1
fi

# Test 3: Sign the certificate
echo ""
echo "Test 3: Sign the certificate"
if $SECUREWIPE_BIN cert sign --file valid_unsigned_cert.json --key $KEYS_DIR/dev_private.pem; then
    echo "✓ Certificate signed successfully"
else
    echo "✗ ERROR: Certificate signing failed"
    exit 1
fi

# Test 4: Validate signed certificate (should pass)
echo ""
echo "Test 4: Validate signed certificate (should pass)"
if $SECUREWIPE_BIN cert validate --file valid_unsigned_cert.json; then
    echo "✓ Signed certificate passed validation"
else
    echo "✗ ERROR: Signed certificate should validate"
    exit 1
fi

# Test 5: Verify certificate signature
echo ""
echo "Test 5: Verify certificate signature"
if $SECUREWIPE_BIN cert verify --file valid_unsigned_cert.json --pubkey $KEYS_DIR/dev_public.pem; then
    echo "✓ Certificate signature verification passed"
else
    echo "✗ ERROR: Certificate signature verification failed"
    exit 1
fi

# Test 6: Create unsupported certificate type and test validation
echo ""
echo "Test 6: Create unsupported certificate type and test validation"
cat > invalid_cert.json << 'EOF'
{
  "cert_type": "invalid_type",
  "cert_id": "invalid_cert",
  "created_at": "2023-12-05T14:30:22.123456Z"
}
EOF

if $SECUREWIPE_BIN cert validate --file invalid_cert.json; then
    echo "✗ ERROR: Invalid certificate type should not validate"
    exit 1
else
    echo "✓ Invalid certificate type correctly failed validation"
fi

# Test 7: Test double signing protection
echo ""
echo "Test 7: Test double signing protection"
cp valid_unsigned_cert.json double_sign_test.json

if $SECUREWIPE_BIN cert sign --file double_sign_test.json --key $KEYS_DIR/dev_private.pem; then
    echo "✗ ERROR: Double signing should fail without --force"
    exit 1
else
    echo "✓ Double signing correctly failed without --force"
fi

# Test 8: Test force signing
echo ""
echo "Test 8: Test force signing"
if $SECUREWIPE_BIN cert sign --file double_sign_test.json --key $KEYS_DIR/dev_private.pem --force; then
    echo "✓ Force signing succeeded"
else
    echo "✗ ERROR: Force signing should succeed"
    exit 1
fi

# Test 9: Create a wipe certificate and test
echo ""
echo "Test 9: Create and test wipe certificate"
cat > wipe_cert.json << 'EOF'
{
  "cert_type": "wipe",
  "cert_id": "test_e2e_wipe_001",
  "certificate_version": "v1.0.0",
  "created_at": "2023-12-05T15:00:30.654321Z",
  "issuer": {
    "organization": "SecureWipe E2E Test",
    "tool_name": "securewipe",
    "tool_version": "v1.0.0",
    "country": "US"
  },
  "device": {
    "model": "Test SSD 1TB E2E",
    "serial": "E2E123456",
    "bus": "NVMe",
    "capacity_bytes": 1000000000000,
    "logical_block_size": 512,
    "total_lbas": 1953125000,
    "firmware": "E2E1.0",
    "path": "/dev/nvme0n1",
    "protocol_path": "PCIe->NVMe"
  },
  "policy": {
    "nist_level": "PURGE",
    "method": "nvme_sanitize",
    "action_mapping": "Sanitize → Crypto Erase → PURGE"
  },
  "hpa_dco": {
    "cleared": true,
    "commands": ["hdparm -N p /dev/nvme0n1"]
  },
  "commands": [
    {
      "cmd": "nvme sanitize /dev/nvme0n1 --sanitize-action=0x02",
      "exit": 0,
      "ms": 30000,
      "stdout_sha256": "e2ec3d4e5f6789012345678901234567890123456789012345678901234567890",
      "stderr_sha256": "e2ed4e5f67890123456789012345678901234567890123456789012345678901a"
    }
  ],
  "verify": {
    "strategy": "random_sectors",
    "samples": 5,
    "coverage": {
      "mode": "samples",
      "samples": 5
    },
    "failures": 0,
    "result": "PASS"
  },
  "result": "PASS",
  "environment": {
    "operator": "e2e_test_user",
    "os_kernel": "Linux 6.1.0",
    "tool_version": "v1.0.0",
    "device_firmware": "E2E1.0"
  },
  "evidence": {
    "logs_sha256": "e2ee5f6789012345678901234567890123456789012345678901234567890ab12"
  },
  "linkage": {
    "backup_cert_id": "test_e2e_backup_001"
  },
  "exceptions": {
    "text": "None"
  },
  "metadata": {
    "certificate_json_sha256": "e2ef6789012345678901234567890123456789012345678901234567890abcde"
  }
}
EOF

# Sign and test wipe certificate
$SECUREWIPE_BIN cert sign --file wipe_cert.json --key $KEYS_DIR/dev_private.pem
$SECUREWIPE_BIN cert validate --file wipe_cert.json
$SECUREWIPE_BIN cert verify --file wipe_cert.json --pubkey $KEYS_DIR/dev_public.pem

echo "✓ Wipe certificate testing completed successfully"

echo ""
echo "=========================================="
echo "All tests passed! 🎉"
echo "=========================================="
echo ""
echo "Features tested:"
echo "  ✓ Certificate schema validation"
echo "  ✓ Certificate signing with Ed25519"
echo "  ✓ Certificate signature verification"
echo "  ✓ Invalid certificate rejection"
echo "  ✓ Double signing protection"
echo "  ✓ Force signing capability"
echo "  ✓ Backup certificate support"
echo "  ✓ Wipe certificate support"
echo ""
echo "Generated test files in: $TEST_DIR"

# Clean up option
echo ""
read -p "Clean up test directory? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$TEST_DIR"
    echo "✓ Test directory cleaned up"
else
    echo "Test files preserved in: $TEST_DIR"
fi
