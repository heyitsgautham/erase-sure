"""Canonical JSON and Ed25519 signature checks."""

import copy
from unittest.mock import MagicMock, patch

from app.main import canonicalize_json, verify_ed25519_signature
from sample_certs import VALID_BACKUP_CERT


class TestCanonicalization:
    """Test RFC 8785 JSON canonicalization"""

    def test_canonicalize_deterministic(self):
        """Test that canonicalization is deterministic"""
        obj1 = {"z_field": "value", "a_field": 42, "nested": {"second": 2, "first": 1}}

        obj2 = {"a_field": 42, "nested": {"first": 1, "second": 2}, "z_field": "value"}

        canonical1 = canonicalize_json(obj1)
        canonical2 = canonicalize_json(obj2)

        assert canonical1 == canonical2

        # Should be sorted keys with no whitespace
        canonical_str = canonical1.decode("utf-8")
        assert canonical_str == '{"a_field":42,"nested":{"first":1,"second":2},"z_field":"value"}'


class TestSignatureVerification:
    """Test Ed25519 signature verification"""

    @patch("app.keys.PUBLIC_KEY_BYTES", b"\x00" * 32)  # Mock public key
    @patch("app.signature.VerifyKey")
    def test_verify_valid_signature(self, mock_verify_key_class):
        """Test verification of valid signature"""
        mock_verify_key = MagicMock()
        mock_verify_key.verify.return_value = None  # No exception = valid
        mock_verify_key_class.return_value = mock_verify_key

        cert = copy.deepcopy(VALID_BACKUP_CERT)
        result = verify_ed25519_signature(cert)
        assert result is True

    @patch("app.keys.PUBLIC_KEY_BYTES", b"\x00" * 32)
    @patch("app.signature.VerifyKey")
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
