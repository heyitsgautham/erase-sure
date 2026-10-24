"""Tests for keygen.py, the development key pair helper."""

import stat

import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

from keygen import PRIVATE_NAME, PUBLIC_NAME, ensure_dev_keys, main


def test_writes_a_matching_ed25519_pair(tmp_path):
    assert ensure_dev_keys(tmp_path) is True

    private = serialization.load_pem_private_key(
        (tmp_path / PRIVATE_NAME).read_bytes(), password=None
    )
    public = serialization.load_pem_public_key((tmp_path / PUBLIC_NAME).read_bytes())
    assert isinstance(private, Ed25519PrivateKey)
    assert isinstance(public, Ed25519PublicKey)

    signature = private.sign(b"certificate bytes")
    public.verify(signature, b"certificate bytes")  # raises if the halves do not match


def test_private_key_is_owner_only(tmp_path):
    ensure_dev_keys(tmp_path)
    mode = stat.S_IMODE((tmp_path / PRIVATE_NAME).stat().st_mode)
    assert mode == 0o600


def test_existing_pair_is_left_alone(tmp_path):
    ensure_dev_keys(tmp_path)
    before = (tmp_path / PRIVATE_NAME).read_bytes()

    assert ensure_dev_keys(tmp_path) is False
    assert (tmp_path / PRIVATE_NAME).read_bytes() == before


def test_refuses_to_complete_half_a_pair(tmp_path):
    (tmp_path / PUBLIC_NAME).write_text("orphan public key")

    with pytest.raises(SystemExit, match="Only one half"):
        ensure_dev_keys(tmp_path)
    assert not (tmp_path / PRIVATE_NAME).exists()


def test_cli_creates_the_output_directory(tmp_path):
    out = tmp_path / "nested" / "keys"
    assert main(["--out", str(out)]) == 0
    assert (out / PRIVATE_NAME).exists() and (out / PUBLIC_NAME).exists()
