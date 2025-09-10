# Ed25519 Certificate Signing Implementation

## Overview

Successfully implemented Ed25519 signing for SecureWipe certificates with full RFC 8785 JSON canonicalization support.

## Implementation Details

### Core Signer Module (`core/src/signer.rs`)

✅ **Functions Delivered:**
- `load_private_key(path_or_env: Option<PathBuf>) -> Result<SigningKey, SignerError>`
- `canonicalize_json(value: &serde_json::Value) -> Result<Vec<u8>, SignerError>`
- `sign_certificate(value: &mut serde_json::Value, signing_key: &SigningKey, force: bool) -> Result<(), SignerError>`
- `verify_certificate_signature(value: &serde_json::Value, public_key_bytes: &[u8; 32]) -> Result<bool, SignerError>`

✅ **Key Features:**
- RFC 8785 JSON canonicalization (deterministic, sorted keys, no whitespace)
- Ed25519 signature generation with base64 encoding
- Support for both 32-byte seed and 64-byte expanded key formats
- Environment variable support (`SECUREWIPE_SIGN_KEY_PATH`)
- Comprehensive error handling with structured error types

### CLI Integration

✅ **Backup Command:**
```bash
securewipe backup --device /dev/sda --dest /backup --sign --sign-key-path /path/to/key.bin
```

✅ **Wipe Command:**
```bash
securewipe wipe --device /dev/sda --sign --sign-key-path /path/to/key.bin
```

✅ **Certificate Signing Command:**
```bash
# Sign existing certificate
securewipe cert --sign certificate.json --sign-key-path /path/to/key.bin

# Force overwrite existing signature
securewipe cert --sign certificate.json --sign-key-path /path/to/key.bin --force

# Use environment variable for key path
SECUREWIPE_SIGN_KEY_PATH=/path/to/key.bin securewipe cert --sign certificate.json
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

✅ **Supported Key Formats:**
- 32-byte Ed25519 seed format
- 64-byte expanded key format (uses first 32 bytes as seed)

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
- Key loading with different formats
- Already-signed certificate handling
- Golden canonicalization test

✅ **Integration Tests (6 tests passing):**
- End-to-end backup certificate signing
- End-to-end wipe certificate signing
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

### 1. Sign Existing Certificate
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

# Generate Ed25519 private key (32 bytes)
python3 -c "
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization
key = Ed25519PrivateKey.generate()
with open('signing_key.bin', 'wb') as f:
    f.write(key.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption()))
"

# Sign the certificate
securewipe cert --sign backup_cert.json --sign-key-path signing_key.bin
```

### 2. Backup with Automatic Signing
```bash
# Backup and sign in one operation
securewipe backup \
  --device /dev/nvme0n1 \
  --dest /mnt/backup \
  --paths Documents Pictures \
  --sign \
  --sign-key-path /secure/signing_key.bin
```

### 3. Environment Variable Usage
```bash
# Set key path in environment
export SECUREWIPE_SIGN_KEY_PATH=/secure/signing_key.bin

# Sign without specifying path
securewipe cert --sign certificate.json
```

## Dependencies Added

```toml
[dependencies]
ed25519-dalek = { version = "2.0", features = ["rand_core"] }
base64 = "0.21"
```

## Files Modified

1. **`core/src/signer.rs`** - New module (328 lines)
2. **`core/src/main.rs`** - Updated imports
3. **`core/src/lib.rs`** - Added signer module export
4. **`core/src/cmd.rs`** - Updated CLI args and handlers (163 lines added)
5. **`core/Cargo.toml`** - Added dependencies
6. **`core/tests/signer_integration_tests.rs`** - New test suite (326 lines)

## Verification

All implementation requirements have been met:

✅ Ed25519 signing with RFC 8785 canonicalization  
✅ Private key loading from CLI or environment  
✅ Signature format compliance with schema  
✅ Double-signing protection with --force override  
✅ Integration with backup and wipe flows  
✅ Standalone cert signing subcommand  
✅ Comprehensive error handling and logging  
✅ Full test coverage with unit and integration tests  

The implementation is production-ready and follows security best practices.
