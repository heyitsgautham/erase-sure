"""
The backup certificate schema in certs/schemas/backup_schema.json accepts a
complete certificate, rejects a broken one, and pins the fields the CLI and the
portal rely on.
"""

import json
from datetime import datetime, timezone
from pathlib import Path

import jsonschema
import pytest


def load_schema():
    """Load the backup certificate schema."""
    schema_path = Path(__file__).parent.parent / "certs" / "schemas" / "backup_schema.json"

    if not schema_path.exists():
        raise FileNotFoundError(f"Schema not found: {schema_path}")

    with open(schema_path) as f:
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
            "country": "IN",
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
        },
        "files_summary": {
            "count": 2847,
            "personal_bytes": 52428800,
            "included_paths": ["/home/user/Documents", "/home/user/Pictures"],
            "excluded_paths": ["/home/user/.cache", "/home/user/.tmp"],
        },
        "destination": {
            "type": "usb",
            "label": "BACKUP_USB_2024",
            "fs": "exfat",
            "mountpoint": "/media/backup",
            "path": "/media/backup/securewipe_backup_20240115",
        },
        "crypto": {
            "alg": "AES-256-CTR",
            "manifest_sha256": "d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2",
            "key_management": "ephemeral_session_key",
        },
        "verification": {
            "strategy": "sampled_files",
            "samples": 142,
            "coverage": {"mode": "percent", "percent": 15.5},
            "failures": 0,
            "notes": "All sampled files verified successfully against manifest hashes",
        },
        "policy": {"name": "NIST SP 800-88 Rev.1", "version": "2023.12"},
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v2.1.0",
            "device_firmware": "5B2QGXA7",
            "containerized": False,
        },
        "exceptions": {"items": [], "text": "None"},
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA==",
            "canonicalization": "RFC8785_JSON",
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
                "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA==",
            },
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_BCK_2024_001",
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


@pytest.fixture(scope="module")
def schema():
    return load_schema()


def test_complete_certificate_is_valid(schema):
    jsonschema.validate(instance=create_test_certificate(), schema=schema)


def test_broken_certificate_is_rejected(schema):
    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(instance=create_failing_certificate(), schema=schema)


def test_schema_examples_are_valid(schema):
    for example in schema.get("examples", []):
        jsonschema.validate(instance=example, schema=schema)


def test_required_fields_match_what_the_portal_reads(schema):
    required = set(schema["required"])
    for field in ["cert_type", "cert_id", "created_at", "device", "signature"]:
        assert field in required
    assert required <= set(create_test_certificate())


@pytest.mark.parametrize(
    ("path", "bad_value"),
    [
        ("cert_type", "wipe"),
        ("crypto.alg", "AES-128-CBC"),
        ("signature.alg", "RSA"),
        ("signature.pubkey_id", "someone_else"),
        ("device.bus", "FLOPPY"),
        ("result", "MAYBE"),
    ],
)
def test_pinned_values_reject_anything_else(schema, path, bad_value):
    cert = create_test_certificate()
    target = cert
    *parents, leaf = path.split(".")
    for part in parents:
        target = target[part]
    target[leaf] = bad_value

    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(instance=cert, schema=schema)
