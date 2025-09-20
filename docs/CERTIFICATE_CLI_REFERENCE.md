# SecureWipe Certificate Commands - Quick Reference

## Command Overview

SecureWipe now includes comprehensive certificate handling capabilities with Ed25519 signing and JSON Schema validation.

## Available Commands

### `securewipe cert validate`
Validates a certificate against its JSON schema.

```bash
securewipe cert validate --file <certificate.json>
```

**Example:**
```bash
securewipe cert validate --file /path/to/backup_cert.json
```

**Success Response:**
```json
{
  "file": "/path/to/backup_cert.json",
  "schema_valid": true,
  "validation_details": {
    "cert_type": "backup",
    "cert_id": "backup_001",
    "errors": []
  }
}
```

---

### `securewipe cert sign`
Signs a certificate with an Ed25519 private key.

```bash
securewipe cert sign --file <certificate.json> --key <private_key.pem> [--force]
```

**Example:**
```bash
securewipe cert sign --file backup_cert.json --key keys/dev_private.pem

# Override existing signature
securewipe cert sign --file backup_cert.json --key keys/dev_private.pem --force
```

**Success Response:**
```json
{
  "file": "backup_cert.json",
  "signed": true,
  "signature_details": {
    "algorithm": "Ed25519",
    "public_key_id": "dev_key_001",
    "signed_at": "2023-12-05T14:30:22.123456Z"
  }
}
```

---

### `securewipe cert verify`
Verifies a certificate's signature and schema validity.

```bash
securewipe cert verify --file <certificate.json> --pubkey <public_key.pem>
```

**Example:**
```bash
securewipe cert verify --file backup_cert.json --pubkey keys/dev_public.pem
```

**Success Response:**
```json
{
  "file": "backup_cert.json",
  "signature_valid": true,
  "schema_valid": true,
  "verification_details": {
    "algorithm": "Ed25519",
    "public_key_id": "dev_key_001",
    "signed_at": "2023-12-05T14:30:22.123456Z",
    "cert_type": "backup",
    "cert_id": "backup_001"
  }
}
```

## Quick Workflow

### Standard Certificate Workflow
```bash
# 1. Create unsigned certificate (JSON file)
# 2. Validate certificate structure
securewipe cert validate --file mycert.json

# 3. Sign certificate
securewipe cert sign --file mycert.json --key private.pem

# 4. Verify certificate
securewipe cert verify --file mycert.json --pubkey public.pem
```

### Development Workflow
```bash
# Using development keys
securewipe cert sign --file test_cert.json --key keys/dev_private.pem
securewipe cert verify --file test_cert.json --pubkey keys/dev_public.pem
```

## Common Options

- `--file` or `-f`: Path to certificate JSON file
- `--key` or `-k`: Path to private key PEM file (for signing)
- `--pubkey` or `-p`: Path to public key PEM file (for verification)
- `--force`: Override existing signatures when signing

## Error Examples

### Validation Errors
```json
{
  "file": "invalid_cert.json",
  "schema_valid": false,
  "validation_details": {
    "cert_type": "unknown",
    "errors": [
      "Missing required field: cert_id",
      "Invalid timestamp format in created_at"
    ]
  }
}
```

### Signing Errors
```json
{
  "file": "already_signed.json",
  "signed": false,
  "error": "Certificate already contains signature. Use --force to override."
}
```

### Verification Errors
```json
{
  "file": "tampered_cert.json",
  "signature_valid": false,
  "schema_valid": true,
  "error": "Signature verification failed: Invalid signature"
}
```

## Key Management

### Development Keys
- **Private**: `keys/dev_private.pem`
- **Public**: `keys/dev_public.pem`
- **Usage**: Development and testing only

### Production Keys
- Store private keys securely
- Use appropriate file permissions (600)
- Consider hardware security modules (HSMs)

## Certificate Types

### Backup Certificates
- **Type**: `"cert_type": "backup"`
- **Schema**: `certs/schemas/backup_schema.json`
- **Purpose**: Document file backup operations

### Wipe Certificates
- **Type**: `"cert_type": "wipe"`
- **Schema**: `certs/schemas/wipe_schema.json`
- **Purpose**: Document disk wiping operations

## Integration Notes

### With Backup Operations
```bash
# Backup automatically validates certificates
securewipe backup --cert backup_cert.json ...
```

### With Wipe Operations
```bash
# Wipe automatically validates certificates
securewipe wipe --cert wipe_cert.json ...
```

### With Portal API
The Python portal integrates with these commands for web-based certificate validation.

## Testing

### End-to-End Test
```bash
./test_certificate_handling.sh
```

### Individual Tests
```bash
cd core && cargo test schema
cd core && cargo test integration
```

## Help Commands

```bash
securewipe cert --help
securewipe cert validate --help
securewipe cert sign --help
securewipe cert verify --help
```
