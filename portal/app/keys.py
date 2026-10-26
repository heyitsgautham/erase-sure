"""
Trust material the portal verifies against: the Ed25519 public key and the
certificate JSON schemas. Both load once at import, and the portal refuses to
start without them.
"""

import json
import os
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

BACKUP_SCHEMA = None
WIPE_SCHEMA = None
PUBLIC_KEY_BYTES = None


def load_public_key():
    """Load Ed25519 public key from PEM file"""
    global PUBLIC_KEY_BYTES

    # Get public key path from environment or use default
    pubkey_path = os.environ.get("SECUREWIPE_PUBKEY_PATH", "keys/dev_public.pem")

    # Try keys/ folder as fallback
    if not os.path.exists(pubkey_path):
        current_dir = Path(__file__).parent.parent.parent
        fallback_path = current_dir / "keys" / "dev_public.pem"
        if fallback_path.exists():
            pubkey_path = str(fallback_path)

    try:
        with open(pubkey_path) as f:
            pem_content = f.read()

        # Parse PEM public key
        public_key = serialization.load_pem_public_key(pem_content.encode())

        if not isinstance(public_key, Ed25519PublicKey):
            raise RuntimeError(f"Public key is not Ed25519: {type(public_key)}")

        # Extract raw 32-byte key
        PUBLIC_KEY_BYTES = public_key.public_bytes(
            encoding=serialization.Encoding.Raw, format=serialization.PublicFormat.Raw
        )

    except FileNotFoundError:
        raise RuntimeError(f"Public key file not found: {pubkey_path}") from None
    except Exception as e:
        raise RuntimeError(f"Failed to load public key: {e}") from e


def load_schemas():
    """Load JSON schemas from files"""
    global BACKUP_SCHEMA, WIPE_SCHEMA

    # Get the project root directory
    current_dir = Path(__file__).parent.parent.parent
    backup_schema_path = current_dir / "certs" / "schemas" / "backup_schema.json"
    wipe_schema_path = current_dir / "certs" / "schemas" / "wipe_schema.json"

    try:
        with open(backup_schema_path) as f:
            BACKUP_SCHEMA = json.load(f)

        with open(wipe_schema_path) as f:
            WIPE_SCHEMA = json.load(f)

    except FileNotFoundError as e:
        raise RuntimeError(f"Schema file not found: {e}") from e
    except json.JSONDecodeError as e:
        raise RuntimeError(f"Invalid JSON in schema file: {e}") from e


load_schemas()
load_public_key()
