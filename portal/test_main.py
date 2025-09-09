"""
Unit tests for the SecureWipe Verification Portal

Tests valid and invalid certificate payloads against the FastAPI endpoints.
"""

import json
import copy
import pytest
from fastapi.testclient import TestClient
from pathlib import Path

from app.main import app, validate_certificate


# Test client
client = TestClient(app)


# Sample valid backup certificate
VALID_BACKUP_CERT = {
    "cert_type": "backup",
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "device": {
        "model": "Samsung SSD 980 PRO 1TB",
        "serial": "S6DC9NL0T12345A",
        "bus": "nvme",
        "capacity_bytes": 1000204886016
    },
    "files_summary": {
        "count": 1543,
        "personal_bytes": 4567890123
    },
    "destination": {
        "type": "usb",
        "label": "External Drive",
        "fs": "exfat"
    },
    "crypto": {
        "alg": "AES-256-CTR",
        "manifest_sha256": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678"
    },
    "created_at": "2023-12-05T14:30:22.123456Z",
    "signature": {
        "alg": "Ed25519",
        "sig": "signature_data_here",
        "pubkey_id": "sih_root_v1"
    }
}

# Sample valid wipe certificate
VALID_WIPE_CERT = {
    "cert_type": "wipe",
    "cert_id": "wipe_20231205_150030_a8b9c7d2",
    "device": {
        "model": "Samsung SSD 980 PRO 1TB",
        "serial": "S6DC9NL0T12345A",
        "bus": "nvme",
        "capacity_bytes": 1000204886016
    },
    "policy": {
        "nist_level": "PURGE",
        "method": "nvme_sanitize_crypto_erase"
    },
    "hpa_dco": {
        "cleared": True,
        "commands": ["hdparm --sanitize-frozen-device /dev/nvme0n1"]
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
        "failures": 0
    },
    "linkage": {
        "backup_cert_id": "backup_20231205_143022_f4a2b8c1"
    },
    "created_at": "2023-12-05T15:00:30.654321Z",
    "signature": {
        "alg": "Ed25519",
        "sig": "signature_data_here",
        "pubkey_id": "sih_root_v1"
    }
}


class TestHealthEndpoint:
    """Test the health check endpoint"""
    
    def test_health_check(self):
        """Test health endpoint returns expected response"""
        response = client.get("/health")
        assert response.status_code == 200
        
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
    
    def test_extract_backup_summary(self):
        """Test summary extraction for backup certificate"""
        result = validate_certificate(VALID_BACKUP_CERT)
        
        assert result.schema_valid is True
        assert len(result.errors) == 0
        
        summary = result.cert_summary
        assert summary["cert_type"] == "backup"
        assert summary["cert_id"] == "backup_20231205_143022_f4a2b8c1"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "destination" in summary
        assert "policy_method" not in summary  # Only for wipe certs
    
    def test_extract_wipe_summary(self):
        """Test summary extraction for wipe certificate"""
        result = validate_certificate(VALID_WIPE_CERT)
        
        assert result.schema_valid is True
        assert len(result.errors) == 0
        
        summary = result.cert_summary
        assert summary["cert_type"] == "wipe"
        assert summary["cert_id"] == "wipe_20231205_150030_a8b9c7d2"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "policy_method" in summary
        assert "PURGE" in summary["policy_method"]
        assert "destination" not in summary  # Only for backup certs


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
