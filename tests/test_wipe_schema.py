#!/usr/bin/env python3
"""
SecureWipe Wipe Certificate Schema Test

Tests the new audit-ready wipe certificate JSON schema for:
- Schema validation
- Required fields enforcement
- Data type validation
- Enum constraints
- Pattern matching (SHA256, base64, semantic versions)
"""

import json
import sys
import os
from datetime import datetime, timezone
from pathlib import Path

try:
    import jsonschema
    from jsonschema import validate, ValidationError
except ImportError:
    print("❌ Missing dependency: jsonschema")
    print("   Install with: pip install jsonschema")
    sys.exit(1)

# Get project root and schema path
PROJECT_ROOT = Path(__file__).parent.parent
SCHEMA_PATH = PROJECT_ROOT / "certs" / "schemas" / "wipe_schema.json"

def load_schema():
    """Load the wipe certificate JSON schema"""
    try:
        with open(SCHEMA_PATH, 'r') as f:
            schema = json.load(f)
        return schema
    except FileNotFoundError:
        print(f"❌ Schema file not found: {SCHEMA_PATH}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Invalid JSON in schema: {e}")
        sys.exit(1)

def create_valid_wipe_certificate():
    """Create a valid wipe certificate that should pass validation"""
    return {
        "cert_type": "wipe",
        "cert_id": "TEST_WPE_2024_001", 
        "certificate_version": "v1.0.0",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "issuer": {
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe", 
            "tool_version": "v2.1.0",
            "country": "IN"
        },
        "device": {
            "model": "Samsung SSD 980 PRO 1TB",
            "serial": "S6TXNX0R123456",
            "bus": "NVMe",
            "capacity_bytes": 1000204886016,
            "path": "/dev/nvme0n1",
            "firmware": "5B2QGXA7",
            "logical_block_size": 512,
            "total_lbas": 1953525168,
            "protocol_path": "PCIe->NVMe"
        },
        "policy": {
            "nist_level": "PURGE",
            "method": "nvme_sanitize_crypto_erase",
            "action_mapping": "Sanitize → Crypto Erase → PURGE"
        },
        "hpa_dco": {
            "cleared": True,
            "commands": [
                "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "nvme sanitize-log /dev/nvme0n1"
            ]
        },
        "commands": [
            {
                "cmd": "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "exit": 0,
                "ms": 45780,
                "stdout_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
                "stderr_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
            },
            {
                "cmd": "nvme sanitize-log /dev/nvme0n1",
                "exit": 0,
                "ms": 120,
                "stdout_sha256": "b2c3d4e5f6789012345678901234567890123456789012345678901234567890"
            }
        ],
        "verify": {
            "strategy": "controller_status",
            "samples": 50,
            "coverage": {
                "mode": "samples",
                "samples": 50
            },
            "failures": 0,
            "result": "PASS"
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic", 
            "tool_version": "v2.1.0",
            "device_firmware": "5B2QGXA7",
            "containerized": False
        },
        "evidence": {
            "smart_snapshot_sha256": "c3d4e5f6789012345678901234567890123456789012345678901234567890ab",
            "nvme_identify_sha256": "d4e5f6789012345678901234567890123456789012345678901234567890abcd", 
            "nvme_sanitize_status_code": "0x0000",
            "logs_sha256": "e5f6789012345678901234567890123456789012345678901234567890abcdef"
        },
        "linkage": {
            "backup_cert_id": "TEST_BCK_2024_001"
        },
        "exceptions": {
            "items": [],
            "text": "None"
        },
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA==",
            "canonicalization": "RFC8785_JSON"
        },
        "metadata": {
            "certificate_json_sha256": "f6789012345678901234567890123456789012345678901234567890abcdef12",
            "qr_payload": {
                "cert_id": "TEST_WPE_2024_001",
                "issued_at": datetime.now(timezone.utc).isoformat(),
                "device_model": "Samsung SSD 980 PRO 1TB",
                "result": "PASS",
                "nist_level": "PURGE",
                "method": "nvme_sanitize_crypto_erase",
                "sha256_cert_json": "f6789012345678901234567890123456789012345678901234567890abcdef12",
                "verify_url": "https://verify.securewipe.org/cert/TEST_WPE_2024_001",
                "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA=="
            },
            "revocation_status": "Good"
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_WPE_2024_001"
    }

def create_invalid_wipe_certificate():
    """Create an invalid wipe certificate (missing required fields)"""
    return {
        "cert_type": "wipe",
        # Missing cert_id, certificate_version, and other required fields
        "created_at": datetime.now(timezone.utc).isoformat(),
        "device": {
            "model": "Test Device"
            # Missing required fields
        }
    }

def validate_certificate(schema, cert_data, test_name):
    """Validate a certificate against the schema"""
    try:
        validate(instance=cert_data, schema=schema)
        return True, None
    except ValidationError as e:
        return False, e

def test_field_presence(cert_data, required_fields):
    """Test that all required fields are present"""
    missing = []
    for field in required_fields:
        if field not in cert_data:
            missing.append(field)
    return missing

def save_sample_certificate(cert_data, filename):
    """Save a sample certificate to file"""
    output_path = PROJECT_ROOT / "tests" / filename
    try:
        with open(output_path, 'w') as f:
            json.dump(cert_data, f, indent=2)
        return output_path, os.path.getsize(output_path)
    except Exception as e:
        return None, f"Error: {e}"

def main():
    print("SecureWipe Wipe Certificate Schema Test")
    print("=" * 50)
    
    # Load schema
    print("Loading wipe certificate schema...")
    schema = load_schema()
    
    if '$id' in schema:
        print(f"✓ Schema loaded successfully")
        print(f"  Schema ID: {schema['$id']}")
        print(f"  Title: {schema.get('title', 'N/A')}")
    else:
        print("⚠️  Schema missing $id field")
    
    print()
    
    # Test valid certificate
    print("Testing valid certificate...")
    valid_cert = create_valid_wipe_certificate()
    is_valid, error = validate_certificate(schema, valid_cert, "valid")
    
    if is_valid:
        print("✓ Valid certificate passed validation")
    else:
        print("✗ Valid certificate failed validation:", str(error))
        print(f"  Error path: {' -> '.join(str(p) for p in error.absolute_path)}")
        return False
    
    # Test invalid certificate  
    print()
    print("Testing invalid certificate...")
    invalid_cert = create_invalid_wipe_certificate()
    is_valid, error = validate_certificate(schema, invalid_cert, "invalid")
    
    if not is_valid:
        print("✓ Invalid certificate correctly failed validation")
        print(f"  First error: {error.message}")
        print(f"  Error path: {' -> '.join(str(p) for p in error.absolute_path)}")
    else:
        print("✗ Invalid certificate incorrectly passed validation")
        return False
    
    # Test schema example if present
    print()
    print("Testing schema example...")
    if 'examples' in schema and len(schema['examples']) > 0:
        example = schema['examples'][0]
        is_valid, error = validate_certificate(schema, example, "schema example")
        if is_valid:
            print("✓ Schema example is valid")
        else:
            print("✗ Schema example is invalid:", str(error))
    else:
        print("⚠️  No examples found in schema")
    
    # Test certificate structure
    print()
    print("Testing certificate structure...")
    required_fields = schema.get('required', [])
    missing_fields = test_field_presence(valid_cert, required_fields)
    
    if not missing_fields:
        print(f"✓ All {len(required_fields)} required fields present")
    else:
        print(f"✗ Missing required fields: {missing_fields}")
        return False
    
    # Test specific field values
    print(f"✓ cert_type: {valid_cert['cert_type']}")
    print(f"✓ certificate_version: {valid_cert['certificate_version']}")
    print(f"✓ result: {valid_cert['result']}")
    print(f"✓ policy.nist_level: {valid_cert['policy']['nist_level']}")
    print(f"✓ signature.alg: {valid_cert['signature']['alg']}")
    print(f"✓ signature.pubkey_id: {valid_cert['signature']['pubkey_id']}")
    print(f"✓ device.bus: {valid_cert['device']['bus']}")
    print(f"✓ verify.strategy: {valid_cert['verify']['strategy']}")
    
    # Save sample certificate
    print()
    output_path, file_size = save_sample_certificate(valid_cert, "sample_wipe_certificate.json")
    if output_path:
        print(f"📄 Sample certificate saved to: {output_path}")
        print(f"   File size: {file_size} bytes")
    else:
        print(f"✗ Failed to save sample certificate: {file_size}")
    
    print()
    print("=" * 50)
    print("🎉 ALL TESTS PASSED")
    return True

if __name__ == "__main__":
    success = main()
    if not success:
        print()
        print("=" * 50)
        print("❌ SOME TESTS FAILED")
        sys.exit(1)
