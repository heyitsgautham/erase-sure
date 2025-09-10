#!/usr/bin/env python3
"""
Test JSON schema validation for backup certificates using the new audit-ready schema.
This test validates that certificate data conforms to the updated backup_schema.json.
"""

import json
import sys
from pathlib import Path
from datetime import datetime, timezone

try:
    import jsonschema
except ImportError:
    print("Error: jsonschema not installed. Run: pip install jsonschema")
    sys.exit(1)


def load_schema():
    """Load the backup certificate schema."""
    schema_path = Path(__file__).parent.parent / "certs" / "schemas" / "backup_schema.json"
    
    if not schema_path.exists():
        raise FileNotFoundError(f"Schema not found: {schema_path}")
    
    with open(schema_path, 'r') as f:
        return json.load(f)


def create_test_certificate():
    """Create a test certificate matching the new schema."""
    return {
        "cert_type": "backup",
        "cert_id": "TEST_BCK_2024_001",
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
            "total_lbas": 1953525168
        },
        "files_summary": {
            "count": 2847,
            "personal_bytes": 52428800,
            "included_paths": ["/home/user/Documents", "/home/user/Pictures"],
            "excluded_paths": ["/home/user/.cache", "/home/user/.tmp"]
        },
        "destination": {
            "type": "usb",
            "label": "BACKUP_USB_2024",
            "fs": "exfat",
            "mountpoint": "/media/backup",
            "path": "/media/backup/securewipe_backup_20240115"
        },
        "crypto": {
            "alg": "AES-256-CTR",
            "manifest_sha256": "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2",
            "key_management": "ephemeral_session_key"
        },
        "verification": {
            "strategy": "sampled_files",
            "samples": 142,
            "coverage": {
                "mode": "percent",
                "percent": 15.5
            },
            "failures": 0,
            "notes": "All sampled files verified successfully against manifest hashes"
        },
        "policy": {
            "name": "NIST SP 800-88 Rev.1",
            "version": "2023.12"
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v2.1.0",
            "device_firmware": "5B2QGXA7",
            "containerized": False
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
            "certificate_json_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
            "logs_sha256": "b2c3d4e5f6789012345678901234567890123456789012345678901234567890",
            "qr_payload": {
                "cert_id": "TEST_BCK_2024_001",
                "issued_at": datetime.now(timezone.utc).isoformat(),
                "device_model": "Samsung SSD 980 PRO 1TB",
                "result": "PASS",
                "nist_level": "CLEAR",
                "method": "backup_aes256",
                "sha256_cert_json": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
                "verify_url": "https://verify.securewipe.org/cert/TEST_BCK_2024_001",
                "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA=="
            }
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_BCK_2024_001"
    }


def create_failing_certificate():
    """Create a certificate that should fail validation."""
    cert = create_test_certificate()
    
    # Introduce validation errors
    cert["cert_type"] = "invalid_type"  # Should be "backup"
    cert["device"]["bus"] = "INVALID_BUS"  # Not in enum
    cert["crypto"]["alg"] = "AES-128"  # Should be "AES-256-CTR"
    cert["signature"]["alg"] = "RSA"  # Should be "Ed25519"
    del cert["certificate_version"]  # Required field
    
    return cert


def test_schema_validation():
    """Test certificate validation against the schema."""
    print("Loading backup certificate schema...")
    
    try:
        schema = load_schema()
        print(f"✓ Schema loaded successfully")
        print(f"  Schema ID: {schema.get('$id', 'N/A')}")
        print(f"  Title: {schema.get('title', 'N/A')}")
    except Exception as e:
        print(f"✗ Failed to load schema: {e}")
        return False
    
    print("\nTesting valid certificate...")
    valid_cert = create_test_certificate()
    
    try:
        jsonschema.validate(valid_cert, schema)
        print("✓ Valid certificate passed validation")
    except jsonschema.exceptions.ValidationError as e:
        print(f"✗ Valid certificate failed validation: {e}")
        print(f"  Error path: {' -> '.join(str(p) for p in e.absolute_path)}")
        return False
    except Exception as e:
        print(f"✗ Unexpected error validating certificate: {e}")
        return False
    
    print("\nTesting invalid certificate...")
    invalid_cert = create_failing_certificate()
    
    try:
        jsonschema.validate(invalid_cert, schema)
        print("✗ Invalid certificate incorrectly passed validation")
        return False
    except jsonschema.exceptions.ValidationError as e:
        print(f"✓ Invalid certificate correctly failed validation")
        print(f"  First error: {e.message}")
        print(f"  Error path: {' -> '.join(str(p) for p in e.absolute_path)}")
    except Exception as e:
        print(f"✗ Unexpected error: {e}")
        return False
    
    print("\nTesting schema example...")
    if "examples" in schema and len(schema["examples"]) > 0:
        example = schema["examples"][0]
        try:
            jsonschema.validate(example, schema)
            print("✓ Schema example is valid")
        except jsonschema.exceptions.ValidationError as e:
            print(f"✗ Schema example failed validation: {e}")
            return False
    else:
        print("⚠ No examples found in schema")
    
    return True


def test_certificate_structure():
    """Test that the certificate structure matches expectations."""
    print("\nTesting certificate structure...")
    
    cert = create_test_certificate()
    
    # Check required top-level fields
    required_fields = [
        "cert_type", "cert_id", "certificate_version", "created_at",
        "issuer", "device", "files_summary", "destination", "crypto",
        "verification", "policy", "result", "environment", "exceptions",
        "signature", "metadata"
    ]
    
    missing_fields = []
    for field in required_fields:
        if field not in cert:
            missing_fields.append(field)
    
    if missing_fields:
        print(f"✗ Missing required fields: {', '.join(missing_fields)}")
        return False
    
    print(f"✓ All {len(required_fields)} required fields present")
    
    # Check specific field types and constraints
    checks = [
        ("cert_type", lambda x: x == "backup", "must be 'backup'"),
        ("certificate_version", lambda x: x.startswith("v") and "." in x, "must be semver format"),
        ("result", lambda x: x in ["PASS", "FAIL"], "must be PASS or FAIL"),
        ("crypto.alg", lambda x: x == "AES-256-CTR", "must be AES-256-CTR"),
        ("signature.alg", lambda x: x == "Ed25519", "must be Ed25519"),
        ("signature.pubkey_id", lambda x: x == "sih_root_v1", "must be sih_root_v1"),
        ("device.bus", lambda x: x in ["SATA", "NVMe", "USB", "SAS", "VIRTIO", "UNKNOWN"], "must be valid bus type")
    ]
    
    for field_path, check_func, error_msg in checks:
        try:
            # Navigate nested fields
            value = cert
            for part in field_path.split('.'):
                value = value[part]
            
            if check_func(value):
                print(f"✓ {field_path}: {value}")
            else:
                print(f"✗ {field_path}: {value} - {error_msg}")
                return False
        except KeyError:
            print(f"✗ {field_path}: missing field")
            return False
    
    return True


def save_test_certificate():
    """Save a test certificate for manual inspection."""
    cert = create_test_certificate()
    output_path = Path(__file__).parent / "sample_backup_certificate.json"
    
    with open(output_path, 'w') as f:
        json.dump(cert, f, indent=2)
    
    print(f"\n📄 Sample certificate saved to: {output_path}")
    print(f"   File size: {output_path.stat().st_size} bytes")


def main():
    """Run all tests."""
    print("SecureWipe Backup Certificate Schema Test")
    print("=" * 50)
    
    success = True
    
    try:
        success &= test_schema_validation()
        success &= test_certificate_structure()
        
        if success:
            save_test_certificate()
        
    except Exception as e:
        print(f"✗ Test suite failed with error: {e}")
        success = False
    
    print("\n" + "=" * 50)
    if success:
        print("🎉 ALL TESTS PASSED")
        print("\nNext steps:")
        print("1. Install PDF dependencies: pip install -r tests/requirements-test.txt")
        print("2. Run PDF generation test: python tests/test_pdf_certificates.py")
        print("3. Verify certificate in portal: POST to /verify endpoint")
    else:
        print("❌ SOME TESTS FAILED")
        print("\nPlease fix the issues above before proceeding.")
    
    return success


if __name__ == "__main__":
    exit(0 if main() else 1)