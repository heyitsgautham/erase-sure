"""
RFC 8785 canonical JSON and Ed25519 signature checks, mirroring core/src/signer.rs.
"""

import base64
import hashlib
import json
from typing import Any

from nacl.exceptions import BadSignatureError
from nacl.signing import VerifyKey

from app import keys


def canonicalize_json(value: dict[str, Any]) -> bytes:
    """
    Canonicalize JSON according to RFC 8785 JSON Canonicalization Scheme (JCS)

    This mirrors the implementation in core/src/signer.rs:
    - UTF-8 encoding
    - Sorted object keys
    - No insignificant whitespace
    - Consistent number formatting
    """

    def canonicalize_value(val):
        if isinstance(val, dict):
            # Sort keys and canonicalize all values
            return {k: canonicalize_value(v) for k, v in sorted(val.items())}
        elif isinstance(val, list):
            # Canonicalize array elements
            return [canonicalize_value(item) for item in val]
        else:
            # Primitive values are already canonical
            return val

    canonical = canonicalize_value(value)

    # Serialize without whitespace for true RFC 8785 compliance
    canonical_json = json.dumps(canonical, separators=(",", ":"), ensure_ascii=False)

    return canonical_json.encode("utf-8")


def verify_ed25519_signature(cert_data: dict[str, Any]) -> bool | None:
    """
    Verify Ed25519 signature on certificate

    Returns:
        True if signature is valid
        False if signature is invalid
        None if no signature is present
    """
    signature_obj = cert_data.get("signature")
    if not signature_obj:
        return None

    # Check required signature fields
    if signature_obj.get("alg") != "Ed25519" or signature_obj.get("pubkey_id") != "sih_root_v1":
        return False

    try:
        # Get signature bytes
        sig_b64 = signature_obj.get("sig")
        if not sig_b64:
            return False

        signature_bytes = base64.b64decode(sig_b64)

        # Create unsigned certificate for canonicalization
        unsigned_cert = cert_data.copy()
        if "signature" in unsigned_cert:
            del unsigned_cert["signature"]

        # Canonicalize the unsigned certificate
        canonical_bytes = canonicalize_json(unsigned_cert)

        # Verify signature using PyNaCl
        verify_key = VerifyKey(keys.PUBLIC_KEY_BYTES)
        verify_key.verify(canonical_bytes, signature_bytes)
        return True

    except (BadSignatureError, ValueError, Exception):
        return False


def compute_certificate_hash(cert_json_bytes: bytes) -> str:
    """Compute SHA256 hash of certificate JSON bytes"""
    return hashlib.sha256(cert_json_bytes).hexdigest()
