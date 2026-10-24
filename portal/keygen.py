#!/usr/bin/env python3
"""
Create a development Ed25519 key pair for signing and verifying certificates.

Writes <out>/dev_private.pem (PKCS#8) and <out>/dev_public.pem (SPKI), the same
files `openssl genpkey -algorithm Ed25519` produces. An existing pair is left
alone, so this is safe to run on every start.

Usage:
    python keygen.py [--out ../keys]
"""

import argparse
import logging
import os
import sys
from pathlib import Path

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

logger = logging.getLogger("securewipe.keygen")

PRIVATE_NAME = "dev_private.pem"
PUBLIC_NAME = "dev_public.pem"


def ensure_dev_keys(out_dir: Path) -> bool:
    """Create the key pair in out_dir unless both files exist. Returns True if it wrote new keys."""
    private_path = out_dir / PRIVATE_NAME
    public_path = out_dir / PUBLIC_NAME

    if private_path.exists() and public_path.exists():
        logger.info("Key pair already present in %s", out_dir)
        return False
    if private_path.exists() or public_path.exists():
        raise SystemExit(
            f"Only one half of the key pair exists in {out_dir}. "
            "Remove it or restore the other half; refusing to overwrite."
        )

    out_dir.mkdir(parents=True, exist_ok=True)
    key = Ed25519PrivateKey.generate()
    private_pem = key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )
    public_pem = key.public_key().public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )

    # Create the private key with owner-only permissions from the start.
    fd = os.open(private_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "wb") as f:
        f.write(private_pem)
    public_path.write_bytes(public_pem)

    logger.info("Wrote new development key pair to %s", out_dir)
    return True


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    parser.add_argument(
        "--out",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "keys",
        help="Directory for the key pair (default: <repo>/keys)",
    )
    args = parser.parse_args(argv)
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s: %(message)s")
    ensure_dev_keys(args.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
