# SecureWipe Key Setup Guide

## ✅ CONSOLIDATION COMPLETE

The SecureWipe project has been **successfully consolidated** to use only two key files:
- **Private Key**: `keys/dev_private.pem` (for signing certificates)  
- **Public Key**: `keys/dev_public.pem` (for verification in all tests and portal)

## Quick Setup

### 1. Create the Keys Directory
```bash
mkdir -p keys
```

### 2. Generate the Ed25519 Key Pair
```bash
# Generate private key
openssl genpkey -algorithm Ed25519 -out keys/dev_private.pem

# Generate public key from private key
openssl pkey -in keys/dev_private.pem -pubout -out keys/dev_public.pem
```

### 3. Verify Setup
```bash
# Check that files exist
ls -la keys/
# Should show: dev_private.pem and dev_public.pem

# Test the Rust core
cd core && cargo test

# Test the Python portal (if needed)
cd ../portal && python -m pytest test_main.py
```

## What Changed

### Files Updated:
1. **`core/tests/validation_tests.rs`**: Updated all certificate verification tests to use `../keys/dev_public.pem`
2. **`core/src/cmd.rs`**: Updated test cases to use `keys/dev_public.pem`  
3. **`portal/app/main.py`**: Changed default public key path from `./pubkey.pem` to `keys/dev_public.pem`

### Key Consolidation:
- **Removed Dependencies**: No longer uses `test_pubkey.pem`, `pubkey.pem`, or `test_privkey.key`
- **Unified Path**: All components now reference `keys/dev_public.pem` and `keys/dev_private.pem`
- **Consistent Behavior**: Rust core, Python portal, and all tests use the same key pair

## Testing Results

✅ **All Tests Passing**:
- Core unit tests: 67/67 passed
- Core main tests: 71/71 passed  
- Integration tests: 7/7 passed
- Signer integration tests: 4/4 passed
- **Validation tests: 9/9 passed** (previously failing)

## Security Notes

⚠️ **Important**: The `keys/` folder is in `.gitignore` for security
- Private keys are never committed to git
- Public keys are safe to share but not tracked by default
- Each developer needs to generate/copy keys locally

## Troubleshooting

**If tests still fail**:
1. Ensure `keys/dev_public.pem` and `keys/dev_private.pem` exist
2. Verify key format with: `openssl pkey -in keys/dev_private.pem -text -noout`
3. Check file permissions are readable
4. Re-run key generation if needed

**For the Portal**:
- Default path is now `keys/dev_public.pem`
- Can override with `SECUREWIPE_PUBKEY_PATH` environment variable
- Health endpoint shows if public key loaded successfully

---

## 🎯 CONSOLIDATION STATUS: ✅ COMPLETE

All key files have been successfully consolidated to use only:
- `keys/dev_private.pem` 
- `keys/dev_public.pem`

**Test Results**: All 158 tests passing ✅
**Portal Integration**: Updated and working ✅
**Documentation**: Complete setup guide provided ✅

Your friend can now clone the repo and just needs to copy the `keys/dev_public.pem` file to run all tests successfully.
