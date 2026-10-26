"""validate_certificate: schema, signature, hash and chain results."""

import copy
import hashlib
import json
from unittest.mock import patch

import pytest

from app.main import validate_certificate
from sample_certs import VALID_BACKUP_CERT, VALID_WIPE_CERT


class TestValidateCertificate:
    """Test certificate validation with all features"""

    @patch("app.verification.verify_ed25519_signature")
    def test_valid_backup_cert_with_hash(self, mock_verify_sig):
        """Test valid backup certificate with hash verification"""
        mock_verify_sig.return_value = True

        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")

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

    @patch("app.verification.verify_ed25519_signature")
    def test_backup_cert_hash_mismatch(self, mock_verify_sig):
        """Test backup certificate with hash mismatch"""
        mock_verify_sig.return_value = True

        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["metadata"]["certificate_json_sha256"] = "wrong_hash"
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")

        result = validate_certificate(cert, cert_json)

        assert result.hash_valid is False
        assert any("hash mismatch" in error.lower() for error in result.errors)

    @patch("app.verification.verify_ed25519_signature")
    def test_wipe_cert_with_valid_linkage(self, mock_verify_sig):
        """Test wipe certificate with valid chain linkage"""
        mock_verify_sig.return_value = True

        wipe_cert = copy.deepcopy(VALID_WIPE_CERT)
        backup_cert = copy.deepcopy(VALID_BACKUP_CERT)

        cert_json = json.dumps(wipe_cert, separators=(",", ":")).encode("utf-8")

        result = validate_certificate(wipe_cert, cert_json, backup_cert)

        assert result.schema_valid is True
        assert result.signature_valid is True
        assert result.chain_valid is True
        assert result.computed["linked_backup_sha256"] is not None
        assert len(result.errors) == 0

    @patch("app.verification.verify_ed25519_signature")
    def test_wipe_cert_linkage_mismatch(self, mock_verify_sig):
        """Test wipe certificate with mismatched linkage"""
        mock_verify_sig.return_value = True

        wipe_cert = copy.deepcopy(VALID_WIPE_CERT)
        backup_cert = copy.deepcopy(VALID_BACKUP_CERT)

        # Mismatch the cert IDs
        backup_cert["cert_id"] = "different_backup_id"

        cert_json = json.dumps(wipe_cert, separators=(",", ":")).encode("utf-8")

        result = validate_certificate(wipe_cert, cert_json, backup_cert)

        assert result.chain_valid is False
        assert any("linkage validation failed" in error.lower() for error in result.errors)

    @patch("app.verification.verify_ed25519_signature")
    def test_tampered_certificate_signature_invalid(self, mock_verify_sig):
        """Test certificate with tampered content (invalid signature)"""
        mock_verify_sig.return_value = False

        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert["cert_id"] = "tampered_id"  # Tamper with the content
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")

        result = validate_certificate(cert, cert_json)

        assert result.signature_valid is False
        assert any("invalid ed25519 signature" in error.lower() for error in result.errors)

    def test_certificate_no_signature(self):
        """Test certificate without signature field"""
        cert = copy.deepcopy(VALID_BACKUP_CERT)
        del cert["signature"]
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")

        result = validate_certificate(cert, cert_json)

        assert result.signature_valid is None


class TestValidationFunction:
    """Test the validation function directly"""

    @patch("app.verification.verify_ed25519_signature")
    def test_extract_backup_summary(self, mock_verify_sig):
        """Test summary extraction for backup certificate"""
        mock_verify_sig.return_value = True

        cert = copy.deepcopy(VALID_BACKUP_CERT)
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")
        result = validate_certificate(cert, cert_json)

        summary = result.cert_summary
        assert summary["cert_type"] == "backup"
        assert summary["cert_id"] == "backup_20231205_143022_f4a2b8c1"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"

    @patch("app.verification.verify_ed25519_signature")
    def test_extract_wipe_summary(self, mock_verify_sig):
        """Test summary extraction for wipe certificate"""
        mock_verify_sig.return_value = True

        cert = copy.deepcopy(VALID_WIPE_CERT)
        cert_json = json.dumps(cert, separators=(",", ":")).encode("utf-8")
        result = validate_certificate(cert, cert_json)

        summary = result.cert_summary
        assert summary["cert_type"] == "wipe"
        assert summary["cert_id"] == "wipe_20231205_150030_a8b9c7d2"
        assert summary["device_model"] == "Samsung SSD 980 PRO 1TB"
        assert "PURGE" in summary["policy_method_or_destination"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
