"""
Certificate verification: schema, signature, hash and chain checks combined
into one VerificationResult.
"""

import json
from typing import Any

from jsonschema import ValidationError as JsonSchemaValidationError
from jsonschema import validate
from pydantic import BaseModel

from app import keys
from app.signature import compute_certificate_hash, verify_ed25519_signature


class VerificationResult(BaseModel):
    """Response model for certificate verification"""

    schema_valid: bool
    signature_valid: bool | None = None
    hash_valid: bool | None = None
    chain_valid: bool | None = None
    cert_summary: dict[str, Any]
    computed: dict[str, str | None]
    errors: list[str]


class CertificateSummary(BaseModel):
    """Summary of key certificate fields"""

    cert_id: str
    cert_type: str
    device_model: str
    policy_method: str | None = None  # For wipe certificates
    destination: str | None = None  # For backup certificates


def extract_cert_summary(cert_data: dict[str, Any]) -> dict[str, Any]:
    """Extract key fields for certificate summary"""
    summary = {
        "cert_id": cert_data.get("cert_id", "unknown"),
        "cert_type": cert_data.get("cert_type", "unknown"),
        "device_model": cert_data.get("device", {}).get("model", "unknown"),
    }

    # Add type-specific fields
    if cert_data.get("cert_type") == "wipe":
        policy = cert_data.get("policy", {})
        summary["policy_method_or_destination"] = (
            f"{policy.get('nist_level', 'unknown')} - {policy.get('method', 'unknown')}"
        )
    elif cert_data.get("cert_type") == "backup":
        destination = cert_data.get("destination", {})
        if destination:
            summary["policy_method_or_destination"] = (
                f"{destination.get('type', 'unknown')} ({destination.get('label', 'unlabeled')})"
            )
        else:
            summary["policy_method_or_destination"] = "backup"

    return summary


def validate_chain_linkage(
    wipe_cert: dict[str, Any], linked_backup_cert: dict[str, Any] | None
) -> bool | None:
    """
    Validate chain linkage between wipe and backup certificates

    Returns:
        True if linkage is valid
        False if linkage is invalid
        None if no linkage information provided
    """
    if not linked_backup_cert:
        return None

    linkage = wipe_cert.get("linkage", {})
    backup_cert_id = linkage.get("backup_cert_id")

    if not backup_cert_id:
        return None

    # Check if backup cert ID matches
    linked_cert_id = linked_backup_cert.get("cert_id")
    return backup_cert_id == linked_cert_id


def validate_certificate(
    cert_data: dict[str, Any],
    cert_json_bytes: bytes,
    linked_backup_cert: dict[str, Any] | None = None,
) -> VerificationResult:
    """
    Validate a certificate against the appropriate schema and verify signature

    Args:
        cert_data: The certificate JSON data
        cert_json_bytes: Raw JSON bytes as received
        linked_backup_cert: Optional linked backup certificate for chain validation

    Returns:
        VerificationResult with validation status and summary
    """
    errors = []
    schema_valid = False
    signature_valid = None
    hash_valid = None
    chain_valid = None
    cert_summary = {}
    computed = {
        "certificate_json_sha256": compute_certificate_hash(cert_json_bytes),
        "linked_backup_sha256": None,
    }

    try:
        # Detect certificate type
        cert_type = cert_data.get("cert_type")
        if not cert_type:
            errors.append("Missing 'cert_type' field")
            return VerificationResult(
                schema_valid=False,
                signature_valid=None,
                hash_valid=None,
                chain_valid=None,
                errors=errors,
                cert_summary=cert_summary,
                computed=computed,
            )

        # Select appropriate schema
        if cert_type == "backup":
            schema = keys.BACKUP_SCHEMA
        elif cert_type == "wipe":
            schema = keys.WIPE_SCHEMA
        else:
            errors.append(f"Unknown certificate type: {cert_type}")
            return VerificationResult(
                schema_valid=False,
                signature_valid=None,
                hash_valid=None,
                chain_valid=None,
                errors=errors,
                cert_summary=cert_summary,
                computed=computed,
            )

        # Validate against schema
        try:
            validate(instance=cert_data, schema=schema)
            schema_valid = True
        except JsonSchemaValidationError as e:
            errors.append(f"Schema validation error: {e.message}")
            if hasattr(e, "path") and e.path:
                errors.append(f"At path: {' -> '.join(str(p) for p in e.path)}")

        # Verify Ed25519 signature
        try:
            signature_valid = verify_ed25519_signature(cert_data)
            if signature_valid is False:
                errors.append("Invalid Ed25519 signature")
        except Exception as e:
            signature_valid = False
            errors.append(f"Signature verification error: {str(e)}")

        # Hash verification for backup certificates
        if cert_type == "backup":
            metadata = cert_data.get("metadata", {})
            expected_hash = metadata.get("certificate_json_sha256")
            if expected_hash:
                computed_hash = computed["certificate_json_sha256"]
                hash_valid = expected_hash.lower() == computed_hash.lower()
                if not hash_valid:
                    errors.append(
                        f"Certificate hash mismatch: expected {expected_hash}, got {computed_hash}"
                    )

        # Chain linkage validation for wipe certificates
        if cert_type == "wipe" and linked_backup_cert:
            try:
                # Validate linked backup cert against backup schema
                validate(instance=linked_backup_cert, schema=keys.BACKUP_SCHEMA)

                # Compute linked backup hash
                linked_backup_bytes = json.dumps(linked_backup_cert, separators=(",", ":")).encode(
                    "utf-8"
                )
                computed["linked_backup_sha256"] = compute_certificate_hash(linked_backup_bytes)

                # Validate linkage
                chain_valid = validate_chain_linkage(cert_data, linked_backup_cert)
                if chain_valid is False:
                    errors.append("Chain linkage validation failed: backup_cert_id mismatch")

            except JsonSchemaValidationError as e:
                chain_valid = False
                errors.append(f"Linked backup certificate schema error: {e.message}")
            except Exception as e:
                chain_valid = False
                errors.append(f"Chain validation error: {str(e)}")

        # Extract summary (even if validation failed, try to get what we can)
        cert_summary = extract_cert_summary(cert_data)

    except Exception as e:
        errors.append(f"Unexpected error during validation: {str(e)}")

    return VerificationResult(
        schema_valid=schema_valid,
        signature_valid=signature_valid,
        hash_valid=hash_valid,
        chain_valid=chain_valid,
        errors=errors,
        cert_summary=cert_summary,
        computed=computed,
    )
