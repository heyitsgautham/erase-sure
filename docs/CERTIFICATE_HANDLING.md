# Certificate Handling Enhancement - Implementation Guide

## Overview

This document describes the comprehensive certificate handling enhancements implemented for SecureWipe, including Ed25519 signing, verification, and JSON Schema validation.

## Features Implemented

### 1. JSON Schema Validation
- **Purpose**: Ensures all certificates conform to predefined schemas before processing
- **Schemas**: `backup_schema.json` and `wipe_schema.json` in `certs/schemas/`
- **Integration**: Automatic validation during signing, manual validation via CLI

### 2. Certificate Signing
- **Algorithm**: Ed25519 (RFC 8032)
- **Key Format**: PEM-encoded private keys
- **Canonicalization**: RFC 8785 JSON Canonical Form for deterministic signatures
- **Protection**: Prevents accidental double-signing (use `--force` to override)

### 3. Certificate Verification
- **Process**: Validates both signature authenticity and schema compliance
- **Key Format**: PEM-encoded public keys
- **Output**: JSON response with detailed validation results

## CLI Commands

### Certificate Validation
```bash
securewipe cert validate --file <certificate.json>
```

**Response Format:**
```json
{
  "file": "/path/to/certificate.json",
  "schema_valid": true,
  "validation_details": {
    "cert_type": "backup",
    "cert_id": "example_001",
    "errors": []
  }
}
```

### Certificate Signing
```bash
securewipe cert sign --file <certificate.json> --key <private_key.pem> [--force]
```

**Response Format:**
```json
{
  "file": "/path/to/certificate.json",
  "signed": true,
  "signature_details": {
    "algorithm": "Ed25519",
    "public_key_id": "dev_key_001",
    "signed_at": "2023-12-05T14:30:22.123456Z"
  }
}
```

**Options:**
- `--force`: Override protection against double-signing existing signatures

### Certificate Verification
```bash
securewipe cert verify --file <certificate.json> --pubkey <public_key.pem>
```

**Response Format:**
```json
{
  "file": "/path/to/certificate.json",
  "signature_valid": true,
  "schema_valid": true,
  "verification_details": {
    "algorithm": "Ed25519",
    "public_key_id": "dev_key_001",
    "signed_at": "2023-12-05T14:30:22.123456Z",
    "cert_type": "backup",
    "cert_id": "example_001"
  }
}
```

## Integration Points

### Rust CLI Core
- **Module**: `core/src/schema.rs` - Schema validation logic
- **Commands**: `core/src/cmd.rs` - CLI command handlers
- **Signing**: `core/src/signer.rs` - Cryptographic operations

### Python Portal
- **Endpoint**: `/verify` - Certificate verification API
- **Integration**: Calls Rust CLI for comprehensive validation
- **Response**: JSON format matching CLI responses

## Error Handling

### Schema Validation Errors
```json
{
  "file": "/path/to/certificate.json",
  "schema_valid": false,
  "validation_details": {
    "cert_type": "unknown",
    "errors": [
      "Missing required field: cert_id",
      "Invalid timestamp format in created_at",
      "Device capacity_bytes must be a positive integer"
    ]
  }
}
```

### Signing Errors
```json
{
  "file": "/path/to/certificate.json",
  "signed": false,
  "error": "Certificate already contains signature. Use --force to override."
}
```

### Verification Errors
```json
{
  "file": "/path/to/certificate.json",
  "signature_valid": false,
  "schema_valid": true,
  "error": "Signature verification failed: Invalid signature"
}
```

## Development Setup

### Dependencies
```toml
[dependencies]
jsonschema = "0.17"
ed25519-dalek = "2.2.0"
serde_json = "1.0"
base64 = "0.22"
sha2 = "0.10"
```

### Key Management
- **Development Keys**: `keys/dev_private.pem` and `keys/dev_public.pem`
- **Production**: Use secure key management system
- **Format**: PKCS#8 PEM format for private keys, SubjectPublicKeyInfo for public keys

## Testing

### Automated Tests
```bash
# Run all schema validation tests
cd core && cargo test schema

# Run integration tests
cd core && cargo test integration

# Run end-to-end test script
./test_certificate_handling.sh
```

### Manual Testing
```bash
# Create test certificate
cat > test_cert.json << 'EOF'
{
  "cert_type": "backup",
  "cert_id": "test_001",
  // ... complete certificate structure
}
EOF

# Test validation
securewipe cert validate --file test_cert.json

# Test signing
securewipe cert sign --file test_cert.json --key keys/dev_private.pem

# Test verification
securewipe cert verify --file test_cert.json --pubkey keys/dev_public.pem
```

## Schema Requirements

### Backup Certificates
- **Required Fields**: `cert_type`, `cert_id`, `certificate_version`, `created_at`, `issuer`, `device`, `files_summary`, `destination`, `crypto`, `verification`, `policy`, `result`, `environment`, `exceptions`, `metadata`
- **Signature Field**: Added automatically during signing process
- **Validation**: Must conform to `backup_schema.json`

### Wipe Certificates
- **Required Fields**: `cert_type`, `cert_id`, `certificate_version`, `created_at`, `issuer`, `device`, `policy`, `hpa_dco`, `commands`, `verify`, `result`, `environment`, `evidence`, `exceptions`, `metadata`
- **Optional Fields**: `linkage` (for connecting to backup certificates)
- **Validation**: Must conform to `wipe_schema.json`

## Security Considerations

### Key Security
- Private keys must be stored securely and never committed to version control
- Use appropriate file permissions (600) for private key files
- Consider hardware security modules (HSMs) for production environments

### Signature Security
- Ed25519 provides 128-bit security level
- JSON canonicalization ensures signature consistency across implementations
- Signature includes all certificate fields except the signature itself

### Validation Security
- Schema validation prevents malformed certificates from being processed
- Comprehensive error messages aid in debugging without exposing sensitive information
- Double-signing protection prevents accidental signature overwrites

## Workflow Integration

### Standard Certificate Lifecycle
1. **Generation**: Create unsigned certificate JSON
2. **Validation**: Validate against schema (`securewipe cert validate`)
3. **Signing**: Sign with private key (`securewipe cert sign`)
4. **Storage**: Store signed certificate securely
5. **Verification**: Verify when needed (`securewipe cert verify`)

### Integration with Backup/Wipe Operations
- Certificates are automatically validated during backup/wipe operations
- Invalid certificates cause operations to fail with clear error messages
- Signed certificates provide tamper-evident audit trail

## API Reference

### CertificateValidator Struct
```rust
impl CertificateValidator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>
    pub fn validate_certificate(&self, cert_path: &str) -> ValidationResult
    pub fn validate_for_signing(&self, cert: &serde_json::Value) -> Result<(), String>
}
```

### ValidationResult Struct
```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub cert_type: Option<String>,
    pub cert_id: Option<String>,
    pub errors: Vec<String>,
}
```

## Troubleshooting

### Common Issues

**"Schema file not found"**
- Ensure `certs/schemas/` directory contains required schema files
- Check file permissions and paths

**"Invalid PEM key format"**
- Verify key files are in correct PEM format
- Ensure private keys use PKCS#8 format
- Check for proper BEGIN/END markers

**"JSON canonicalization failed"**
- Verify certificate JSON is valid
- Check for circular references or unsupported data types

**"Signature verification failed"**
- Ensure public key matches the private key used for signing
- Verify certificate hasn't been modified after signing
- Check that signature field is properly formatted

## Performance Considerations

- Schema validation is fast but scales with certificate size
- Ed25519 signing/verification is very fast (~50k ops/sec)
- JSON canonicalization adds minimal overhead
- Consider caching compiled schemas for high-volume operations

## Future Enhancements

### Planned Features
- Certificate revocation lists (CRL) support
- Multiple signature algorithms support
- Batch certificate processing
- Certificate chain validation
- Hardware security module (HSM) integration

### Configuration Options
- Configurable schema validation strictness
- Custom error message formatting
- Alternative canonicalization methods
- Signature algorithm selection
