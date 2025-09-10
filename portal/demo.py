#!/usr/bin/env python3
"""
SecureWipe Portal Demo
Shows Ed25519 signature verification, hash recomputation, and chain linkage validation
"""

import json
import sys
from fastapi.testclient import TestClient
from app.main import app

def demo_verification():
    """Demonstrate the verification portal functionality"""
    client = TestClient(app)
    
    print("🔒 SecureWipe Verification Portal Demo")
    print("=" * 50)
    
    # Test 1: Health check
    print("\n1️⃣  Health Check")
    response = client.get("/health")
    health = response.json()
    print(f"   Status: {health['status']}")
    print(f"   Schemas loaded: {health['schemas_loaded']}")
    print(f"   Public key loaded: {health['public_key_loaded']}")
    
    # Test 2: Valid backup certificate (schema validation)
    print("\n2️⃣  Schema Validation - Valid Backup Certificate")
    valid_backup = {
        "cert_type": "backup",
        "cert_id": "demo_backup_001",
        "certificate_version": "v1.0.0",
        "created_at": "2023-12-05T14:30:22.123456Z",
        "issuer": {
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v1.0.0"
        },
        "device": {
            "model": "Samsung SSD 980 PRO 1TB",
            "serial": "S6DC9NL0T12345A",
            "bus": "SATA",
            "capacity_bytes": 1000204886016
        },
        "files_summary": {
            "count": 1543,
            "personal_bytes": 4567890123,
            "included_paths": ["/home/user"]
        },
        "destination": {
            "type": "usb",
            "label": "External Drive",
            "fs": "exfat"
        },
        "crypto": {
            "alg": "AES-256-CTR",
            "manifest_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
            "key_management": "ephemeral_session_key"
        },
        "verification": {
            "strategy": "sampled_files",
            "samples": 100,
            "coverage": {"mode": "percent", "percent": 15.0},
            "failures": 0
        },
        "policy": {
            "name": "NIST SP 800-88 Rev.1",
            "version": "2023.12"
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v1.0.0",
            "device_firmware": "test",
            "containerized": False
        },
        "exceptions": {
            "items": [],
            "text": "None"
        },
        "metadata": {
            "certificate_json_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678"
        },
        "signature": {
            "alg": "Ed25519",
            "sig": "dGVzdF9zaWduYXR1cmVfZGF0YQ==",
            "pubkey_id": "sih_root_v1",
            "canonicalization": "RFC8785_JSON"
        }
    }
    
    response = client.post("/verify", json=valid_backup)
    result = response.json()
    
    print(f"   ✅ Schema Valid: {result['schema_valid']}")
    print(f"   🔐 Signature Valid: {result['signature_valid']}")
    print(f"   🧮 Hash Valid: {result['hash_valid']}")
    print(f"   📋 Certificate ID: {result['cert_summary']['cert_id']}")
    print(f"   📊 Computed Hash: {result['computed']['certificate_json_sha256'][:16]}...")
    
    # Test 3: Invalid signature
    print("\n3️⃣  Signature Verification - Invalid Signature")
    tampered_cert = valid_backup.copy()
    tampered_cert["cert_id"] = "tampered_certificate"  # This will make signature invalid
    
    response = client.post("/verify", json=tampered_cert)
    result = response.json()
    
    print(f"   ✅ Schema Valid: {result['schema_valid']}")
    print(f"   ❌ Signature Valid: {result['signature_valid']}")
    print(f"   ⚠️  Error: {result['errors'][0] if result['errors'] else 'None'}")
    
    # Test 4: Chain linkage validation
    print("\n4️⃣  Chain Linkage Validation")
    wipe_cert = {
        "cert_type": "wipe",
        "cert_id": "demo_wipe_001",
        "certificate_version": "v1.0.0",
        "created_at": "2023-12-05T15:00:30.654321Z",
        "issuer": {
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v1.0.0"
        },
        "device": {
            "model": "Samsung SSD 980 PRO 1TB",
            "serial": "S6DC9NL0T12345A",
            "bus": "SATA",
            "capacity_bytes": 1000204886016
        },
        "policy": {
            "nist_level": "PURGE",
            "method": "nvme_sanitize_crypto_erase"
        },
        "commands": [
            {
                "cmd": "nvme sanitize /dev/nvme0n1 --sanitize-action=2",
                "exit": 0,
                "ms": 45000
            }
        ],
        "verify": {
            "strategy": "controller_status",
            "samples": 100,
            "coverage": {"mode": "samples", "samples": 100},
            "failures": 0,
            "result": "PASS"
        },
        "result": "PASS",
        "environment": {
            "operator": "admin",
            "os_kernel": "Linux 6.8.0-35-generic",
            "tool_version": "v1.0.0",
            "device_firmware": "test",
            "containerized": False
        },
        "evidence": {
            "smart_snapshot_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
            "logs_sha256": "b2c3d4e5f6789012345678901234567890123456789012345678901234567890"
        },
        "linkage": {
            "backup_cert_id": "demo_backup_001"  # Links to the backup cert above
        },
        "exceptions": {
            "items": [],
            "text": "None"
        },
        "metadata": {
            "certificate_json_sha256": "f6789012345678901234567890123456789012345678901234567890abcdef12"
        },
        "signature": {
            "alg": "Ed25519",
            "sig": "dGVzdF93aXBlX3NpZ25hdHVyZQ==",
            "pubkey_id": "sih_root_v1",
            "canonicalization": "RFC8785_JSON"
        },
        "linked_backup_cert": valid_backup
    }
    
    response = client.post("/verify", json=wipe_cert)
    result = response.json()
    
    print(f"   ✅ Schema Valid: {result['schema_valid']}")
    print(f"   🔐 Signature Valid: {result['signature_valid']}")
    print(f"   🔗 Chain Valid: {result['chain_valid']}")
    print(f"   🧮 Linked Backup Hash: {result['computed']['linked_backup_sha256'][:16]}...")
    
    print("\n✨ Demo Complete!")
    print("\nKey Features Demonstrated:")
    print("• ✅ Schema validation (backup/wipe certificates)")
    print("• 🔐 Ed25519 signature verification with RFC 8785 canonicalization")  
    print("• 🧮 SHA256 hash recomputation and validation")
    print("• 🔗 Chain linkage validation between wipe and backup certificates")
    print("• ⚡ Real-time validation with detailed error reporting")

if __name__ == "__main__":
    demo_verification()
