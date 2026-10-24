"""
Test setup for the verification portal.

app.main loads an Ed25519 public key at import time and refuses to start
without one. The real dev key lives in the gitignored keys/ folder, so a fresh
clone has none. This file creates a throwaway key pair before any test module
imports the app and points SECUREWIPE_PUBKEY_PATH at its public half.
"""

import os
import tempfile
from pathlib import Path

import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from nacl.signing import SigningKey

_SIGNING_KEY = SigningKey.generate()
_KEY_DIR = Path(tempfile.mkdtemp(prefix="securewipe-portal-test-"))
_PUBKEY_PATH = _KEY_DIR / "test_public.pem"
_PUBKEY_PATH.write_bytes(
    Ed25519PublicKey.from_public_bytes(bytes(_SIGNING_KEY.verify_key)).public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )
)
os.environ["SECUREWIPE_PUBKEY_PATH"] = str(_PUBKEY_PATH)


@pytest.fixture
def signing_key() -> SigningKey:
    """Private half of the key pair the app under test trusts."""
    return _SIGNING_KEY
