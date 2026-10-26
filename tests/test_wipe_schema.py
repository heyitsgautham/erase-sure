"""
The wipe certificate schema in certs/schemas/wipe_schema.json accepts a
complete certificate, rejects a broken one, and pins the fields the CLI and the
portal rely on.
"""

import json
from datetime import datetime, timezone
from pathlib import Path

import jsonschema
import pytest

SCHEMA_PATH = Path(__file__).resolve().parents[1] / "certs" / "schemas" / "wipe_schema.json"


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
            "protocol_path": "PCIe->NVMe",
        },
        "policy": {
            "nist_level": "PURGE",
            "method": "nvme_sanitize_crypto_erase",
            "action_mapping": "Sanitize → Crypto Erase → PURGE",
        },
        "hpa_dco": {
            "cleared": True,
            "commands": [
                "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "nvme sanitize-log /dev/nvme0n1",
            ],
        },
        "commands": [
            {
                "cmd": "nvme sanitize /dev/nvme0n1 --crypto-erase",
                "exit": 0,
                "ms": 45780,
                "stdout_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
                "stderr_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            },
            {
                "cmd": "nvme sanitize-log /dev/nvme0n1",
                "exit": 0,
                "ms": 120,
                "stdout_sha256": "b2c3d4e5f6789012345678901234567890123456789012345678901234567890",
            },
        ],
        "verify": {
            "strategy": "controller_status",
            "samples": 50,
            "coverage": {"mode": "samples", "samples": 50},
            "failures": 0,
            "result": "PASS",
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v2.1.0",
            "device_firmware": "5B2QGXA7",
            "containerized": False,
        },
        "evidence": {
            "smart_snapshot_sha256": "c3d4e5f6789012345678901234567890123456789012345678901234567890ab",
            "nvme_identify_sha256": "d4e5f6789012345678901234567890123456789012345678901234567890abcd",
            "nvme_sanitize_status_code": "0x0000",
            "logs_sha256": "e5f6789012345678901234567890123456789012345678901234567890abcdef",
        },
        "linkage": {"backup_cert_id": "TEST_BCK_2024_001"},
        "exceptions": {"items": [], "text": "None"},
        "signature": {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA==",
            "canonicalization": "RFC8785_JSON",
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
                "sig": "YWJjZGVmMTIzNDU2Nzg5MGFiY2RlZjEyMzQ1Njc4OTBhYmNkZWYxMjM0NTY3ODkwYWJjZGVmMTIzNDU2Nzg5MA==",
            },
            "revocation_status": "Good",
        },
        "verify_url": "https://verify.securewipe.org/cert/TEST_WPE_2024_001",
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
        },
    }


@pytest.fixture(scope="module")
def schema():
    return json.loads(SCHEMA_PATH.read_text())


def test_complete_certificate_is_valid(schema):
    jsonschema.validate(instance=create_valid_wipe_certificate(), schema=schema)


def test_incomplete_certificate_is_rejected(schema):
    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(instance=create_invalid_wipe_certificate(), schema=schema)


def test_schema_examples_are_valid(schema):
    for example in schema.get("examples", []):
        jsonschema.validate(instance=example, schema=schema)


def test_every_required_field_is_in_the_complete_certificate(schema):
    assert set(schema["required"]) <= set(create_valid_wipe_certificate())


@pytest.mark.parametrize(
    ("path", "bad_value"),
    [
        ("cert_type", "backup"),
        ("policy.nist_level", "DESTROY_MAYBE"),
        ("signature.alg", "RSA"),
        ("signature.pubkey_id", "someone_else"),
        ("result", "MAYBE"),
    ],
)
def test_pinned_values_reject_anything_else(schema, path, bad_value):
    cert = create_valid_wipe_certificate()
    target = cert
    *parents, leaf = path.split(".")
    for part in parents:
        target = target[part]
    target[leaf] = bad_value

    with pytest.raises(jsonschema.ValidationError):
        jsonschema.validate(instance=cert, schema=schema)
