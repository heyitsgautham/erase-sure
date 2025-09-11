# Development Keys Setup for SecureWipe Integration Tests

## Overview

Integration tests have been updated to use pre-generated Ed25519 development keypairs instead of generating new keys on every test run. This improves test reliability and CI/CD performance.

## Key Locations

- **Private Key (Raw)**: `keys/dev_private.key` (32 bytes)
- **Private Key (PEM)**: `keys/dev_private.pem` (for reference)
- **Public Key (Raw)**: `keys/dev_public.key` (32 bytes)
- **Public Key (PEM)**: `keys/dev_public.pem` (for verification CLI)

## Setting Up Dev Keys

Run these commands from the repository root:

```bash
# Create keys directory
mkdir -p keys

# Generate Ed25519 keypair in PEM format
openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem
openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem

# Convert PEM keys to raw bytes (32 bytes each) for CLI signing
python3 -c "import base64; pem=open('keys/dev_private.pem').read(); b64=''.join(pem.split('\n')[1:-2]); open('keys/dev_private.key','wb').write(base64.b64decode(b64)[-32:])"
python3 -c "import base64; pem=open('keys/dev_public.pem').read(); b64=''.join(pem.split('\n')[1:-2]); open('keys/dev_public.key','wb').write(base64.b64decode(b64)[-32:])"

# Add to gitignore
echo 'keys/' >> .gitignore
```

## File Formats

### Raw Key Files (.key)
- Used by CLI `cert sign` command
- 32 bytes for both private and public keys
- Binary format

### PEM Key Files (.pem)
- Used by CLI `cert verify` command (public key only)
- Base64-encoded with headers/footers
- Human-readable format

## Integration Test Functions

The following tests have been implemented in `core/tests/signer_integration_tests.rs`:

1. **`test_dev_keys_availability()`**
   - Documents setup instructions if keys are missing
   - Validates CLI functionality if keys exist

2. **`test_sign_and_verify_with_dev_keys()`**
   - Signs a certificate using dev private key
   - Verifies signature using dev public key
   - Tests complete sign→verify workflow

3. **`test_tamper_detection_with_dev_keys()`**
   - Signs certificate, then modifies its content
   - Verifies that tampered certificates are rejected

4. **`test_missing_signature_detection()`**
   - Tests verification of unsigned certificates
   - Ensures proper handling of missing signature fields

## Usage Examples

### Signing a Certificate
```bash
cd core
cargo run -- cert sign --file ../test_cert.json --sign-key-path ../keys/dev_private.key
```

### Verifying a Certificate
```bash
cd core
cargo run -- cert verify --file ../test_cert.json --pubkey ../keys/dev_public.pem
```

## Test Execution

Run integration tests:
```bash
cd core
cargo test --test signer_integration_tests -- --nocapture
```

Run all tests:
```bash
cd core
cargo test
```

## Security Notes

- Dev keys are for **testing purposes only**
- Keys directory is excluded from version control
- Production deployments should use proper key management
- Integration tests gracefully skip if dev keys are not found

## Key Generation Output

After setup, you should see:
```bash
ls -la keys/
# Expected output:
# -rw-r--r--  32 user staff  dev_private.key  # 32-byte raw private key
# -rw-------  119 user staff  dev_private.pem  # PEM private key
# -rw-r--r--  32 user staff  dev_public.key   # 32-byte raw public key
# -rw-r--r--  113 user staff  dev_public.pem   # PEM public key
```

The integration tests ensure reliable and deterministic certificate signing/verification workflows using consistent keypairs across development environments.
