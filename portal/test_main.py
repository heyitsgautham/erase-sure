"""
Unit tests for the SecureWipe Verification Portal

Tests valid and invalid certificate payloads against the FastAPI endpoints.
"""

import json
import copy
import base64
import hashlib
import pytest
from fastapi.testclient import TestClient
from pathlib import Path
from unittest.mock import patch, MagicMock

from app.main import app, validate_certificate, canonicalize_json, verify_ed25519_signature


# Test client
client = TestClient(app)


# Sample valid backup certificate
VALID_BACKUP_CERT = {
    "cert_type": "backup",
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "certificate_version": "v1.0.0",
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
    "issuer": {
        "organization": "SecureWipe (SIH)",
        "tool_name": "securewipe", 
        "tool_version": "v1.0.0"
    },
    "created_at": "2023-12-05T14:30:22.123456Z",
    "metadata": {
        "certificate_json_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678"
    },
    "signature": {
        "alg": "Ed25519",
        "sig": "dGVzdF9zaWduYXR1cmVfZGF0YQ==",  # base64 encoded test data
        "pubkey_id": "sih_root_v1",
        "canonicalization": "RFC8785_JSON"
    }
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
        "backup_cert_id": "backup_20231205_143022_f4a2b8c1"
    },
    "exceptions": {
        "items": [],
        "text": "None"
    },
    "issuer": {
        "organization": "SecureWipe (SIH)",
        "tool_name": "securewipe",
        "tool_version": "v1.0.0"
    },
    "created_at": "2023-12-05T15:00:30.654321Z",
    "metadata": {
        "certificate_json_sha256": "f6789012345678901234567890123456789012345678901234567890abcdef12"
    },
    "signature": {
        "alg": "Ed25519",
        "sig": "dGVzdF93aXBlX3NpZ25hdHVyZQ==",  # base64 encoded test data
        "pubkey_id": "sih_root_v1",
        "canonicalization": "RFC8785_JSON"
    }
}


class TestCanonicalization:
    """Test RFC 8785 JSON canonicalization"""
    
    def test_canonicalize_deterministic(self):
        """Test that canonicalization is deterministic"""
        obj1 = {
            "z_field": "value",
            "a_field": 42,
            "nested": {
                "second": 2,
                "first": 1
            }
        }
        
        obj2 = {
            "a_field": 42,
            "nested": {
                "first": 1,
                "second": 2
            },
            "z_field": "value"
        }
        
        canonical1 = canonicalize_json(obj1)
        canonical2 = canonicalize_json(obj2)
        
        assert canonical1 == canonical2
        
        # Should be sorted keys with no whitespace
        canonical_str = canonical1.decode('utf-8')
        assert canonical_str == '{"a_field":42,"nested":{"first":1,"second":2},"z_field":"value"}'


class TestSignatureVerification:
    """Test Ed25519 signature verification"""
    
    @patch('app.main.PUBLIC_KEY_BYTES', b'\x00' * 32)  # Mock public key
    @patch('app.main.VerifyKey')
    def test_verify_valid_signature(self, mock_verify_key_class):
        """Test verification of valid signature"""
        mock_verify_key = MagicMock()
        mock_verify_key.verify.return_value = None  # No exception = valid
        mock_verify_key_class.return_value = mock_verify_key
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        result = verify_ed25519_signature(cert)
        assert result is True
    
    @patch('app.main.PUBLIC_KEY_BYTES', b'\x00' * 32)
    @patch('app.main.VerifyKey')
    def test_verify_invalid_signature(self, mock_verify_key_class):
        """Test verification of invalid signature"""
        from nacl.exceptions import BadSignatureError
        
        mock_verify_key = MagicMock()
        mock_verify_key.verify.side_effect = BadSignatureError()
        mock_verify_key_class.return_value = mock_verify_key
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        result = verify_ed25519_signature(cert)
        assert result is False
    
    def test_verify_missing_signature(self):
        """Test verification when signature is missing"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        del cert["signature"]
        
        result = verify_ed25519_signature(cert)
        assert result is None
    
    def test_verify_wrong_algorithm(self):
        """Test verification with wrong algorithm"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["signature"]["alg"] = "RSA"
        
        result = verify_ed25519_signature(cert)
        assert result is False
    
    def test_verify_wrong_pubkey_id(self):
        """Test verification with wrong pubkey_id"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["signature"]["pubkey_id"] = "wrong_key"
        
        result = verify_ed25519_signature(cert)
        assert result is False


class TestValidateCertificate:
    """Test certificate validation with all features"""
    
    @patch('app.main.verify_ed25519_signature')
    def test_valid_backup_cert_with_hash(self, mock_verify_sig):
        """Test valid backup certificate with hash verification"""
        mock_verify_sig.return_value = True
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        
        # Set expected hash to match computed hash
        computed_hash = hashlib.sha256(cert_json).hexdigest()
        cert["metadata"]["certificate_json_sha256"] = computed_hash
        
        result = validate_certificate(cert, cert_json)
        
        assert result.schema_valid is True
        assert result.signature_valid is True
        assert result.hash_valid is True
        assert result.chain_valid is None
        assert result.computed["certificate_json_sha256"] == computed_hash
        assert len(result.errors) == 0
    
    @patch('app.main.verify_ed25519_signature')
    def test_backup_cert_hash_mismatch(self, mock_verify_sig):
        """Test backup certificate with hash mismatch"""
        mock_verify_sig.return_value = True
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["metadata"]["certificate_json_sha256"] = "wrong_hash"
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        
        result = validate_certificate(cert, cert_json)
        
        assert result.hash_valid is False
        assert any("hash mismatch" in error.lower() for error in result.errors)
    
    @patch('app.main.verify_ed25519_signature')
    def test_wipe_cert_with_valid_linkage(self, mock_verify_sig):
        """Test wipe certificate with valid chain linkage"""
        mock_verify_sig.return_value = True
        
        wipe_cert = copy.deepcopy(VALID_WIPE_CERT)
        backup_cert = copy.deepcopy(VALID_BACKUP_CERT)
        
        cert_json = json.dumps(wipe_cert, separators=(',', ':')).encode('utf-8')
        
        result = validate_certificate(wipe_cert, cert_json, backup_cert)
        
        assert result.schema_valid is True
        assert result.signature_valid is True
        assert result.chain_valid is True
        assert result.computed["linked_backup_sha256"] is not None
        assert len(result.errors) == 0
    
    @patch('app.main.verify_ed25519_signature')
    def test_wipe_cert_linkage_mismatch(self, mock_verify_sig):
        """Test wipe certificate with mismatched linkage"""
        mock_verify_sig.return_value = True
        
        wipe_cert = copy.deepcopy(VALID_WIPE_CERT)
        backup_cert = copy.deepcopy(VALID_BACKUP_CERT)
        
        # Mismatch the cert IDs
        backup_cert["cert_id"] = "different_backup_id"
        
        cert_json = json.dumps(wipe_cert, separators=(',', ':')).encode('utf-8')
        
        result = validate_certificate(wipe_cert, cert_json, backup_cert)
        
        assert result.chain_valid is False
        assert any("linkage validation failed" in error.lower() for error in result.errors)
    
    @patch('app.main.verify_ed25519_signature')
    def test_tampered_certificate_signature_invalid(self, mock_verify_sig):
        """Test certificate with tampered content (invalid signature)"""
        mock_verify_sig.return_value = False
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["cert_id"] = "tampered_id"  # Tamper with the content
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        
        result = validate_certificate(cert, cert_json)
        
        assert result.signature_valid is False
        assert any("invalid ed25519 signature" in error.lower() for error in result.errors)
    
    def test_certificate_no_signature(self):
        """Test certificate without signature field"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        del cert["signature"]
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        
        result = validate_certificate(cert, cert_json)
        
        assert result.signature_valid is None


class TestHealthEndpoint:
    """Test the health check endpoint"""
    
    def test_health_check(self):
        """Test health endpoint returns expected response"""
        response = client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "healthy"
        assert "schemas_loaded" in data
        assert "public_key_loaded" in data


class TestVerifyEndpoint:
    """Test the /verify POST endpoint"""
    
    @patch('app.main.verify_ed25519_signature')
    def test_verify_valid_backup_certificate(self, mock_verify_sig):
        """Test verification of a valid backup certificate"""
        mock_verify_sig.return_value = True
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        
        response = client.post("/verify", json=cert)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is True
        assert data["signature_valid"] is True
        assert "computed" in data
        assert "certificate_json_sha256" in data["computed"]
        assert data["cert_summary"]["cert_type"] == "backup"
    
    @patch('app.main.verify_ed25519_signature')
    def test_verify_wipe_with_linked_backup(self, mock_verify_sig):
        """Test verification of wipe certificate with linked backup"""
        mock_verify_sig.return_value = True
        
        wipe_cert = copy.deepcopy(VALID_WIPE_CERT)
        backup_cert = copy.deepcopy(VALID_BACKUP_CERT)
        
        # Include linked backup certificate
        request_data = wipe_cert.copy()
        request_data["linked_backup_cert"] = backup_cert
        
        response = client.post("/verify", json=request_data)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is True
        assert data["signature_valid"] is True
        assert data["chain_valid"] is True
        assert data["computed"]["linked_backup_sha256"] is not None
    
    def test_verify_invalid_json(self):
        """Test verification with invalid JSON"""
        response = client.post("/verify", data="invalid json")
        assert response.status_code == 400
        assert "Invalid JSON" in response.json()["detail"]
    
    def test_verify_empty_body(self):
        """Test verification with empty request body"""
        response = client.post("/verify", data="")
        assert response.status_code == 400
        assert "Empty request body" in response.json()["detail"]
    
    def test_verify_missing_cert_type(self):
        """Test verification with missing cert_type"""
        cert = {"cert_id": "test"}
        
        response = client.post("/verify", json=cert)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert "Missing 'cert_type' field" in data["errors"]
    
    def test_verify_unknown_cert_type(self):
        """Test verification with unknown cert_type"""
        cert = {"cert_type": "unknown", "cert_id": "test"}
        
        response = client.post("/verify", json=cert)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert any("Unknown certificate type" in error for error in data["errors"])


class TestGetCertificateInfo:
    """Test the GET /verify/{cert_id} endpoint"""
    
    def test_get_certificate_info_placeholder(self):
        """Test the placeholder GET endpoint"""
        response = client.get("/verify/test_cert_id")
        assert response.status_code == 200
        
        data = response.json()
        assert "cert_id" in data
        assert data["cert_id"] == "test_cert_id"
        assert "message" in data
        
        data = response.json()
        assert data["status"] == "healthy"
        assert "schemas_loaded" in data
        assert "version" in data


class TestHomeEndpoint:
    """Test the home HTML endpoint"""
    
    def test_home_page(self):
        """Test home page returns HTML"""
        response = client.get("/")
        assert response.status_code == 200
        assert "text/html" in response.headers["content-type"]
        assert "SecureWipe Verification Portal" in response.text
        assert "/verify" in response.text


class TestCertificateVerification:
    """Test certificate verification endpoint"""
    
    def test_verify_valid_backup_certificate(self):
        """Test verification of a valid backup certificate"""
        response = client.post(
            "/verify",
            json=VALID_BACKUP_CERT
        )
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is True
        assert len(data["errors"]) == 0
        
        summary = data["cert_summary"]
        assert summary["cert_id"] == "backup_20231205_143022_f4a2b8c1"
        assert summary["cert_type"] == "backup"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "usb" in summary["destination"]
    
    def test_verify_valid_wipe_certificate(self):
        """Test verification of a valid wipe certificate"""
        response = client.post(
            "/verify",
            json=VALID_WIPE_CERT
        )
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is True
        assert len(data["errors"]) == 0
        
        summary = data["cert_summary"]
        assert summary["cert_id"] == "wipe_20231205_150030_a8b9c7d2"
        assert summary["cert_type"] == "wipe"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "PURGE" in summary["policy_method"]
    
    def test_verify_missing_cert_type(self):
        """Test verification fails when cert_type is missing"""
        invalid_cert = {
            "cert_id": "test_cert_123",
            "device": {"model": "Test Device"}
        }
        
        response = client.post("/verify", json=invalid_cert)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert any("cert_type" in error for error in data["errors"])
    
    def test_verify_invalid_cert_type(self):
        """Test verification fails with invalid cert_type"""
        invalid_cert = {
            "cert_type": "invalid_type",
            "cert_id": "test_cert_123",
            "device": {"model": "Test Device"}
        }
        
        response = client.post("/verify", json=invalid_cert)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert any("Unknown certificate type" in error for error in data["errors"])
    
    def test_verify_missing_required_fields_backup(self):
        """Test backup certificate fails validation when required fields are missing"""
        invalid_backup = {
            "cert_type": "backup",
            "cert_id": "test_backup_123"
            # Missing required fields: device, files_summary, destination, crypto, created_at, signature
        }
        
        response = client.post("/verify", json=invalid_backup)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert len(data["errors"]) > 0
    
    def test_verify_missing_required_fields_wipe(self):
        """Test wipe certificate fails validation when required fields are missing"""
        invalid_wipe = {
            "cert_type": "wipe",
            "cert_id": "test_wipe_123"
            # Missing required fields: device, policy, commands, verify, created_at, signature
        }
        
        response = client.post("/verify", json=invalid_wipe)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert len(data["errors"]) > 0
    
    def test_verify_invalid_json(self):
        """Test endpoint handles invalid JSON gracefully"""
        response = client.post(
            "/verify",
            data="invalid json {",
            headers={"Content-Type": "application/json"}
        )
        assert response.status_code == 400
        assert "Invalid JSON" in response.json()["detail"]
    
    def test_verify_empty_body(self):
        """Test endpoint handles empty request body"""
        response = client.post(
            "/verify",
            data="",
            headers={"Content-Type": "application/json"}
        )
        assert response.status_code == 400
        assert "Empty request body" in response.json()["detail"]
    
    def test_verify_invalid_enum_values(self):
        """Test validation fails for invalid enum values"""
        invalid_backup = copy.deepcopy(VALID_BACKUP_CERT)
        invalid_backup["destination"]["type"] = "invalid_destination_type"
        
        response = client.post("/verify", json=invalid_backup)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert len(data["errors"]) > 0
    
    def test_verify_invalid_signature_algorithm(self):
        """Test validation fails for invalid signature algorithm"""
        invalid_backup = copy.deepcopy(VALID_BACKUP_CERT)
        invalid_backup["signature"]["alg"] = "RSA256"  # Invalid, must be Ed25519
        
        response = client.post("/verify", json=invalid_backup)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert len(data["errors"]) > 0
    
    def test_verify_invalid_pubkey_id(self):
        """Test validation fails for invalid pubkey_id"""
        invalid_backup = copy.deepcopy(VALID_BACKUP_CERT)
        invalid_backup["signature"]["pubkey_id"] = "wrong_key_id"  # Must be "sih_root_v1"
        
        response = client.post("/verify", json=invalid_backup)
        assert response.status_code == 200
        
        data = response.json()
        assert data["schema_valid"] is False
        assert len(data["errors"]) > 0


class TestCertificateInfoEndpoint:
    """Test the certificate info endpoint (placeholder)"""
    
    def test_get_certificate_info(self):
        """Test certificate info endpoint returns placeholder message"""
        cert_id = "test_cert_123"
        response = client.get(f"/verify/{cert_id}")
        assert response.status_code == 200
        
        data = response.json()
        assert "not implemented in MVP" in data["message"]
        assert data["cert_id"] == cert_id
        assert "hint" in data


class TestValidationFunction:
    """Test the validation function directly"""
    
    @patch('app.main.verify_ed25519_signature')
    def test_extract_backup_summary(self, mock_verify_sig):
        """Test summary extraction for backup certificate"""
        mock_verify_sig.return_value = True
        
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        result = validate_certificate(cert, cert_json)
        
        summary = result.cert_summary
        assert summary["cert_type"] == "backup"
        assert summary["cert_id"] == "backup_20231205_143022_f4a2b8c1"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
    
    @patch('app.main.verify_ed25519_signature')
    def test_extract_wipe_summary(self, mock_verify_sig):
        """Test summary extraction for wipe certificate"""
        mock_verify_sig.return_value = True
        
        cert = copy.deepcopy(VALID_WIPE_CERT)
        cert_json = json.dumps(cert, separators=(',', ':')).encode('utf-8')
        result = validate_certificate(cert, cert_json)
        
        summary = result.cert_summary
        assert summary["cert_type"] == "wipe"
        assert summary["cert_id"] == "wipe_20231205_150030_a8b9c7d2"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "PURGE" in summary["policy_method_or_destination"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
