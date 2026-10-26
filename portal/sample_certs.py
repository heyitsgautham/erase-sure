"""Certificates the portal tests start from: one valid backup and one valid wipe certificate."""

# Sample valid backup certificate
VALID_BACKUP_CERT = {
    "cert_type": "backup",
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "certificate_version": "v1.0.0",
    "device": {
        "model": "Samsung SSD 980 PRO 1TB",
        "serial": "S6DC9NL0T12345A",
        "bus": "SATA",
        "capacity_bytes": 1000204886016,
    },
    "files_summary": {
        "count": 1543,
        "personal_bytes": 4567890123,
        "included_paths": ["/home/user"],
    },
    "destination": {"type": "usb", "label": "External Drive", "fs": "exfat"},
    "crypto": {
        "alg": "AES-256-CTR",
        "manifest_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
        "key_management": "ephemeral_session_key",
    },
    "verification": {
        "strategy": "sampled_files",
        "samples": 100,
        "coverage": {"mode": "percent", "percent": 15.0},
        "failures": 0,
    },
    "policy": {"name": "NIST SP 800-88 Rev.1", "version": "2023.12"},
    "result": "PASS",
    "environment": {
        "operator": "admin",
        "os_kernel": "Linux 6.8.0-35-generic",
        "tool_version": "v1.0.0",
        "device_firmware": "test",
        "containerized": False,
    },
    "exceptions": {"items": [], "text": "None"},
    "issuer": {
        "organization": "SecureWipe (SIH)",
        "tool_name": "securewipe",
        "tool_version": "v1.0.0",
    },
    "created_at": "2023-12-05T14:30:22.123456Z",
    "metadata": {
        "certificate_json_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678"
    },
    "signature": {
        "alg": "Ed25519",
        "sig": "dGVzdF9zaWduYXR1cmVfZGF0YQ==",  # base64 encoded test data
        "pubkey_id": "sih_root_v1",
        "canonicalization": "RFC8785_JSON",
    },
}

# Sample valid wipe certificate
VALID_WIPE_CERT = {
    "cert_type": "wipe",
    "cert_id": "wipe_20231205_150030_a8b9c7d2",
    "certificate_version": "v1.0.0",
    "device": {
        "model": "Samsung SSD 980 PRO 1TB",
        "serial": "S6DC9NL0T12345A",
        "bus": "SATA",
        "capacity_bytes": 1000204886016,
    },
    "policy": {"nist_level": "PURGE", "method": "nvme_sanitize_crypto_erase"},
    "commands": [{"cmd": "nvme sanitize /dev/nvme0n1 --sanitize-action=2", "exit": 0, "ms": 45000}],
    "verify": {
        "strategy": "controller_status",
        "samples": 100,
        "coverage": {"mode": "samples", "samples": 100},
        "failures": 0,
        "result": "PASS",
    },
    "result": "PASS",
    "environment": {
        "operator": "admin",
        "os_kernel": "Linux 6.8.0-35-generic",
        "tool_version": "v1.0.0",
        "device_firmware": "test",
        "containerized": False,
    },
    "evidence": {
        "smart_snapshot_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
        "logs_sha256": "b2c3d4e5f6789012345678901234567890123456789012345678901234567890",
    },
    "linkage": {"backup_cert_id": "backup_20231205_143022_f4a2b8c1"},
    "exceptions": {"items": [], "text": "None"},
    "issuer": {
        "organization": "SecureWipe (SIH)",
        "tool_name": "securewipe",
        "tool_version": "v1.0.0",
    },
    "created_at": "2023-12-05T15:00:30.654321Z",
    "metadata": {
        "certificate_json_sha256": "f6789012345678901234567890123456789012345678901234567890abcdef12"
    },
    "signature": {
        "alg": "Ed25519",
        "sig": "dGVzdF93aXBlX3NpZ25hdHVyZQ==",  # base64 encoded test data
        "pubkey_id": "sih_root_v1",
        "canonicalization": "RFC8785_JSON",
    },
}
