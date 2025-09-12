# Ed25519 Certificate Signing Implementation (PEM-Only)

## Overview

Successfully implemented Ed25519 signing for SecureWipe certificates with full RFC 8785 JSON canonicalization support. **PEM-only implementation** - accepts only PKCS#8 Ed25519 PEM keys for maximum compatibility and security.

## Implementation Details

### Core Signer Module (`core/src/signer.rs`)

✅ **Functions Delivered:**
- `load_private_key(path_or_env: Option<PathBuf>) -> Result<SigningKey, SignerError>`
- `canonicalize_json(value: &serde_json::Value) -> Result<Vec<u8>, SignerError>`
- `sign_certificate(value: &mut serde_json::Value, signing_key: &SigningKey, force: bool) -> Result<(), SignerError>`

✅ **Key Features:**
- RFC 8785 JSON canonicalization (deterministic, sorted keys, no whitespace)
- Ed25519 signature generation with base64 encoding
- **PEM-only key support** (PKCS#8 "-----BEGIN PRIVATE KEY-----" format)
- Environment variable support (`SECUREWIPE_SIGN_KEY_PATH`)
- Comprehensive error handling with structured error types

### CLI Integration

✅ **Backup Command:**
```bash
securewipe backup --device /dev/sda --dest /backup --sign --sign-key-path keys/dev_private.pem
```

✅ **Wipe Command:**
```bash
securewipe wipe --device /dev/sda --sign --sign-key-path keys/dev_private.pem
```

✅ **Certificate Signing Command:**
```bash
# Sign existing certificate
securewipe cert sign --file certificate.json --sign-key-path keys/dev_private.pem

# Force overwrite existing signature  
securewipe cert sign --file certificate.json --sign-key-path keys/dev_private.pem --force

# Use environment variable for key path
export SECUREWIPE_SIGN_KEY_PATH=keys/dev_private.pem
securewipe cert sign --file certificate.json
```

✅ **Certificate Verification Command:**
```bash
# Verify a signed certificate
securewipe cert verify --file certificate.json --pubkey keys/dev_public.pem
```

### Signature Format

Signatures are added to certificates with the following JSON structure:
```json
{
  "signature": {
    "alg": "Ed25519",
    "pubkey_id": "sih_root_v1", 
    "sig": "base64_encoded_signature_bytes",
    "canonicalization": "RFC8785_JSON"
  }
}
```

### Key Loading

✅ **Priority Order:**
1. CLI argument `--sign-key-path`
2. Environment variable `SECUREWIPE_SIGN_KEY_PATH`

✅ **Supported Key Formats (PEM-Only):**
- Ed25519 PKCS#8 private keys: `-----BEGIN PRIVATE KEY-----` 
- Ed25519 public keys: `-----BEGIN PUBLIC KEY-----`
- Rejects raw binary .key files with clear error messages

### Security Features

✅ **Protection Against Double-Signing:**
- Prevents accidental re-signing of certificates
- `--force` flag required to overwrite existing signatures

✅ **Canonicalization Guarantees:**
- Deterministic JSON representation
- Resistant to key ordering and whitespace variations
- Compliant with RFC 8785 JSON Canonicalization Scheme

### Testing

✅ **Unit Tests (6 tests passing):**
- JSON canonicalization determinism
- Ed25519 signature round-trip verification
- PEM key loading (valid/invalid formats)
- Already-signed certificate handling
- Golden canonicalization test

✅ **Integration Tests (4 tests passing):**
- End-to-end sign and verify with dev PEM keys
- Tamper detection with dev PEM keys
- Missing signature detection
- Environment variable key loading
- Signature tampering detection
- Cross-key verification failure
- Complex certificate canonicalization

### Error Handling

✅ **Comprehensive Error Types:**
- `KeyFileError`: Private key file issues
- `InvalidKeyFormat`: Unsupported key formats
- `CanonicalizationError`: JSON processing errors
- `SignatureError`: Cryptographic operation failures
- `AlreadySigned`: Double-signing prevention
- `IoError`: File system operations

### Logging

✅ **Structured JSON Logging:**
- Key loading events
- Canonicalization process
- Signature operations
- File operations
- Error conditions

## Usage Examples

### 1. Generate Development Keys (One-Time Setup)
```bash
# Generate Ed25519 keypair in PEM format
mkdir -p keys
openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem
openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem
echo 'keys/' >> .gitignore
```

### 2. Sign Existing Certificate
```bash
# Create test certificate
cat > backup_cert.json << EOF
{
  "cert_id": "backup_001",
  "cert_type": "backup",
  "created_at": "2024-12-20T10:30:00Z",
  "device": {
    "model": "Samsung SSD 980 PRO",
    "serial": "ABC123"
  }
}
EOF

# Sign the certificate (PEM-only)
securewipe cert sign --file backup_cert.json --sign-key-path keys/dev_private.pem
```

### 3. Verify Certificate
```bash
# Verify the signed certificate
securewipe cert verify --file backup_cert.json --pubkey keys/dev_public.pem
```

### 4. Backup with Automatic Signing
```bash
# Backup and sign in one operation
securewipe backup \
  --device /dev/nvme0n1 \
  --dest /mnt/backup \
  --paths Documents Pictures \
  --sign \
  --sign-key-path keys/dev_private.pem
```

### 5. Environment Variable Usage
```bash
# Set key path in environment  
export SECUREWIPE_SIGN_KEY_PATH=keys/dev_private.pem

# Sign without specifying path
securewipe cert sign --file certificate.json

# For verification, portal can use:
export SECUREWIPE_PUBKEY_PATH=keys/dev_public.pem
```

## Dependencies Added

```toml
[dependencies]
ed25519-dalek = { version = "2.0", features = ["rand_core"] }
base64 = "0.21"
```

## Files Modified (PEM-Only Unified Implementation)

1. **`core/src/signer.rs`** - PEM-only signer module
2. **`core/src/cmd.rs`** - PEM-only CLI verification  
3. **`core/tests/signer_integration_tests.rs`** - PEM-only test suite
4. **`SIGNING_IMPLEMENTATION.md`** - Updated to PEM-only instructions

## Verification

All PEM-only implementation requirements have been met:

✅ **PEM-Only Signing**: Accepts only PKCS#8 Ed25519 private key PEM  
✅ **PEM-Only Verification**: Accepts only Ed25519 public key PEM  
✅ **Strict Error Messages**: Clear guidance when non-PEM keys provided  
✅ **RFC 8785 JSON Canonicalization**: Deterministic signing  
✅ **Integration Tests**: Use dev_private.pem and dev_public.pem  
✅ **Environment Variables**: SECUREWIPE_SIGN_KEY_PATH support  
✅ **CLI Verification**: `securewipe cert verify --file <cert.json> --pubkey <pubkey.pem>`  
✅ **Documentation**: Updated to PEM-only workflows  
✅ **.gitignore**: Ensures ./keys/ directory is ignored  

**Key Benefits of PEM-Only Approach:**
- Standard OpenSSL compatibility
- Better tooling integration  
- Clear format expectations
- Reduced complexity
- Industry standard practice

The implementation is production-ready with unified PEM-only key handling.
