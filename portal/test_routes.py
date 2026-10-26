"""The portal's HTTP routes, end to end through the FastAPI test client."""

import base64
import copy
from unittest.mock import patch

from fastapi.testclient import TestClient

from app.main import app, canonicalize_json
from sample_certs import VALID_BACKUP_CERT, VALID_WIPE_CERT

# Test client
client = TestClient(app)


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

    @patch("app.verification.verify_ed25519_signature")
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

    @patch("app.verification.verify_ed25519_signature")
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

    def test_verify_rejects_non_object_json(self):
        """A JSON array or scalar is not a certificate and gets a 400, not a 500"""
        for body in ("[]", '"cert"', "42"):
            response = client.post(
                "/verify", content=body, headers={"Content-Type": "application/json"}
            )
            assert response.status_code == 400, body
            assert response.json()["detail"] == "Certificate must be a JSON object"

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
        assert "POST" in data["hint"]


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

    @staticmethod
    def _sign(cert, signing_key):
        """Sign cert the way core/src/signer.rs does: Ed25519 over the canonical JSON without the signature."""
        unsigned = {k: v for k, v in cert.items() if k != "signature"}
        sig = signing_key.sign(canonicalize_json(unsigned)).signature
        signed = copy.deepcopy(unsigned)
        signed["signature"] = {
            "alg": "Ed25519",
            "pubkey_id": "sih_root_v1",
            "sig": base64.b64encode(sig).decode(),
        }
        return signed

    def test_verify_valid_backup_certificate(self, signing_key):
        """A backup certificate signed with the trusted key verifies end to end, no mocks"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        # No producer writes metadata.certificate_json_sha256 yet, so leave it out.
        cert["metadata"].pop("certificate_json_sha256", None)

        response = client.post("/verify", json=self._sign(cert, signing_key))
        assert response.status_code == 200

        data = response.json()
        assert data["schema_valid"] is True
        assert data["signature_valid"] is True
        assert data["errors"] == []

        summary = data["cert_summary"]
        assert summary["cert_id"] == "backup_20231205_143022_f4a2b8c1"
        assert summary["cert_type"] == "backup"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "usb" in summary["policy_method_or_destination"]

    def test_verify_valid_wipe_certificate(self, signing_key):
        """A wipe certificate signed with the trusted key verifies end to end, no mocks"""
        response = client.post(
            "/verify", json=self._sign(copy.deepcopy(VALID_WIPE_CERT), signing_key)
        )
        assert response.status_code == 200

        data = response.json()
        assert data["schema_valid"] is True
        assert data["signature_valid"] is True
        assert data["errors"] == []

        summary = data["cert_summary"]
        assert summary["cert_id"] == "wipe_20231205_150030_a8b9c7d2"
        assert summary["cert_type"] == "wipe"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "PURGE" in summary["policy_method_or_destination"]

    def test_tampered_signed_certificate_fails_signature(self, signing_key):
        """Changing any field after signing breaks the real signature check"""
        signed = self._sign(copy.deepcopy(VALID_WIPE_CERT), signing_key)
        signed["device"]["serial"] = "SOMEONE-ELSES-DISK"

        data = client.post("/verify", json=signed).json()
        assert data["signature_valid"] is False
        assert "Invalid Ed25519 signature" in data["errors"]

    def test_certificate_signed_by_another_key_fails(self):
        """A signature from an untrusted key is rejected"""
        from nacl.signing import SigningKey

        signed = self._sign(copy.deepcopy(VALID_WIPE_CERT), SigningKey.generate())

        data = client.post("/verify", json=signed).json()
        assert data["signature_valid"] is False

    def test_verify_missing_cert_type(self):
        """Test verification fails when cert_type is missing"""
        invalid_cert = {"cert_id": "test_cert_123", "device": {"model": "Test Device"}}

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
            "device": {"model": "Test Device"},
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
            "cert_id": "test_backup_123",
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
            "cert_id": "test_wipe_123",
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
            "/verify", data="invalid json {", headers={"Content-Type": "application/json"}
        )
        assert response.status_code == 400
        assert "Invalid JSON" in response.json()["detail"]

    def test_verify_empty_body(self):
        """Test endpoint handles empty request body"""
        response = client.post("/verify", data="", headers={"Content-Type": "application/json"})
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
