"""
Certificates signed by the Rust CLI must verify in the portal.

fixtures/cli_signed_backup_cert.json was signed with
`securewipe cert sign --key <throwaway key>`, and fixtures/cli_signer_public.pem
is the public half of that throwaway key. The private half was discarded.
"""

import json
from pathlib import Path
from unittest.mock import patch

from cryptography.hazmat.primitives import serialization
from fastapi.testclient import TestClient

from app.main import app, verify_ed25519_signature

FIXTURES = Path(__file__).parent / "fixtures"
CERT = json.loads((FIXTURES / "cli_signed_backup_cert.json").read_text())
CLI_PUBKEY = serialization.load_pem_public_key(
    (FIXTURES / "cli_signer_public.pem").read_bytes()
).public_bytes(encoding=serialization.Encoding.Raw, format=serialization.PublicFormat.Raw)

client = TestClient(app)


def test_fixture_has_spaces_inside_string_values():
    # The interop bug only showed with whitespace inside strings, so keep the fixture honest.
    assert " " in CERT["issuer"]["organization"]


@patch("app.keys.PUBLIC_KEY_BYTES", CLI_PUBKEY)
def test_cli_signature_verifies_in_portal():
    assert verify_ed25519_signature(CERT) is True


@patch("app.keys.PUBLIC_KEY_BYTES", CLI_PUBKEY)
def test_cli_signed_certificate_through_the_endpoint():
    data = client.post("/verify", json=CERT).json()
    assert data["schema_valid"] is True
    assert data["signature_valid"] is True


@patch("app.keys.PUBLIC_KEY_BYTES", CLI_PUBKEY)
def test_editing_a_cli_signed_certificate_breaks_the_signature():
    tampered = json.loads(json.dumps(CERT))
    tampered["issuer"]["organization"] = tampered["issuer"]["organization"].replace(" ", "")
    assert verify_ed25519_signature(tampered) is False
