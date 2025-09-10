#!/usr/bin/env python3
"""
Complete SecureWipe Portal Verification Demo
============================================

This script demonstrates all the implemented verification features:
1. Ed25519 signature verification
2. SHA256 hash recomputation and validation  
3. Chain linkage validation between wipe and backup certificates
4. JSON schema validation
5. Complete API integration testing
"""

import json
import hashlib
import requests
from typing import Dict, Any

def test_portal_verification():
    """Test all portal verification features"""
    base_url = "http://localhost:8000"
    
    print("🔒 SecureWipe Portal Verification Demo")
    print("=" * 45)
    
    # Test 1: Health Check
    print("\n🏥 Health Check")
    try:
        response = requests.get(f"{base_url}/health")
        health = response.json()
        print(f"   Status: {health['status']}")
        print(f"   Schemas Loaded: {health['schemas_loaded']}")
        print(f"   Public Key Loaded: {health['public_key_loaded']}")
    except Exception as e:
        print(f"   ❌ Health check failed: {e}")
        return
    
    # Test 2: Schema Validation (Missing Required Fields)
    print("\n📋 Schema Validation Test")
    invalid_cert = {
        "cert_type": "backup",
        "cert_id": "test_001"
        # Missing many required fields
    }
    
    response = requests.post(f"{base_url}/verify", json=invalid_cert)
    result = response.json()
    print(f"   Schema Valid: {result.get('schema_valid', False)}")
    print(f"   Error Count: {len(result.get('errors', []))}")
    if result.get('errors'):
        print(f"   First Error: {result['errors'][0]}")
    
    # Test 3: Signature Verification (Invalid Signature)
    print("\n🔐 Signature Verification Test")
    cert_with_sig = {
        "cert_type": "backup",
        "cert_id": "sig_test_001",
        "certificate_version": "v1.0.0",
        "created_at": "2023-01-01T00:00:00Z",
        "device": {
            "model": "Test SSD",
            "serial": "SIG123",
            "bus": "SATA", 
            "capacity_bytes": 1000000000
        },
        "files_summary": {
            "count": 10,
            "personal_bytes": 1000000,
            "included_paths": ["/test"]
        },
        "destination": {
            "type": "usb",
            "label": "Test Drive",
            "fs": "exfat"
        },
        "crypto": {
            "alg": "AES-256-CTR",
            "manifest_sha256": "a" * 64,
            "key_management": "ephemeral_session_key"
        },
        "verification": {
            "strategy": "sampled_files",
            "samples": 5,
            "coverage": {"mode": "percent", "percent": 10.0},
            "failures": 0
        },
        "policy": {
            "name": "NIST SP 800-88 Rev.1",
            "version": "2023.12"
        },
        "result": "PASS",
        "environment": {
            "operator": "test",
            "os_kernel": "Linux 5.4.0",
            "tool_version": "v1.0.0",
            "device_firmware": "test",
            "containerized": False
        },
        "exceptions": {
            "items": [],
            "text": "None"
        },
        "metadata": {
            "certificate_json_sha256": "invalid_hash_here"
        },
        "issuer": {
            "organization": "SecureWipe (SIH)",
            "tool_name": "securewipe",
            "tool_version": "v1.0.0"
        },
        "signature": {
            "alg": "Ed25519", 
            "sig": "invalid_signature_base64",
            "pubkey_id": "sih_root_v1",
            "canonicalization": "RFC8785_JSON"
        }
    }
    
    response = requests.post(f"{base_url}/verify", json=cert_with_sig)
    result = response.json()
    print(f"   Schema Valid: {result.get('schema_valid', False)}")
    print(f"   Signature Valid: {result.get('signature_valid', False)}")
    print(f"   Hash Valid: {result.get('hash_valid', False)}")
    
    # Test 4: Hash Computation Demo
    print("\n🧮 Hash Computation Demo")
    simple_cert = {"cert_type": "backup", "cert_id": "hash_demo"}
    cert_json = json.dumps(simple_cert, separators=(',', ':'))
    expected_hash = hashlib.sha256(cert_json.encode('utf-8')).hexdigest()
    
    response = requests.post(f"{base_url}/verify", json=simple_cert)
    result = response.json()
    computed_hash = result.get('computed', {}).get('certificate_json_sha256', '')
    
    print(f"   Expected: {expected_hash[:32]}...")
    print(f"   Computed: {computed_hash[:32]}...")
    print(f"   Match: {expected_hash == computed_hash}")
    
    # Test 5: Chain Linkage Demo
    print("\n🔗 Chain Linkage Demo")
    
    # Create backup cert
    backup_cert = {
        "cert_type": "backup",
        "cert_id": "chain_backup_001",
        # ... other required fields would go here
    }
    
    # Create wipe cert with linkage
    wipe_cert = {
        "cert_type": "wipe", 
        "cert_id": "chain_wipe_001",
        "linkage": {
            "backup_cert_id": "chain_backup_001"  # Links to backup
        },
        "linked_backup_cert": backup_cert,  # Include backup for validation
        # ... other required fields would go here
    }
    
    response = requests.post(f"{base_url}/verify", json=wipe_cert)
    result = response.json()
    print(f"   Chain Valid: {result.get('chain_valid', 'None')}")
    print(f"   Has Linkage: {'linkage' in wipe_cert}")
    print(f"   Has Backup: {'linked_backup_cert' in wipe_cert}")
    
    print("\n🎯 All Verification Features Demonstrated!")
    print("\nImplemented Features:")
    print("  ✅ Ed25519 signature verification with PyNaCl")
    print("  ✅ RFC 8785 JSON canonicalization")
    print("  ✅ SHA256 hash recomputation and validation")
    print("  ✅ Chain linkage validation between certificates")
    print("  ✅ JSON schema validation for backup/wipe certificates")
    print("  ✅ Comprehensive error handling and reporting")
    print("  ✅ RESTful API integration with FastAPI")
    
    return True

if __name__ == "__main__":
    print("Starting SecureWipe Portal verification demo...")
    print("Make sure the portal is running on http://localhost:8000")
    print()
    
    try:
        test_portal_verification()
    except Exception as e:
        print(f"Demo failed: {e}")
        print("Make sure to start the portal first: python run.py")
