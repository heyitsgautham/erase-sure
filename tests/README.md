# Testing SecureWipe Certificate Format

This directory contains tests for validating the new audit-ready certificate format against the JSON Schema specifications.

## Quick Start

From the project root directory, run:

```bash
# Make test script executable
chmod +x test_certificates.sh

# Run all certificate tests
./test_certificates.sh
```

## Test Components

### 1. JSON Schema Validation (`test_backup_schema.py`)

Tests that certificate JSON conforms to the strict schema defined in `certs/schemas/backup_schema.json`.

**What it tests:**
- Schema loading and validation
- Required field presence
- Field type constraints (enums, patterns, formats)
- Sample certificate generation
- Positive and negative validation cases

**Requirements:**
- `jsonschema` library (auto-installed by test runner)

### 2. PDF Certificate Generation (`test_pdf_certificates.py`)

Tests PDF certificate generation with the new audit-ready format.

**What it tests:**
- JSON-to-PDF conversion
- Professional styling with tables and QR codes
- All certificate sections (device, crypto, verification, etc.)
- QR code generation for verification links

**Requirements:**
- `reportlab` - PDF generation
- `qrcode[pil]` - QR code generation
- `Pillow` - Image processing

Install with:
```bash
pip install -r tests/requirements-test.txt
```

## Certificate Structure

The new audit-ready backup certificate includes:

### Required Fields
- `cert_type`: "backup" 
- `cert_id`: Unique identifier (letters, digits, underscore, hyphen)
- `certificate_version`: Semantic version (e.g., "v1.0.0")
- `created_at`: RFC3339 timestamp with timezone
- `issuer`: Organization and tool information
- `device`: Hardware details (model, serial, bus, capacity)
- `files_summary`: Backup statistics
- `destination`: Backup target information
- `crypto`: AES-256-CTR encryption details
- `verification`: Post-backup verification results
- `policy`: NIST compliance policy
- `result`: "PASS" or "FAIL"
- `environment`: Runtime environment details
- `exceptions`: Any issues or exceptions
- `signature`: Ed25519 digital signature
- `metadata`: Certificate hashes and QR payload

### Security Requirements
- **Encryption**: Mandatory AES-256-CTR
- **Signatures**: Ed25519 only, pubkey_id must be "sih_root_v1"
- **Hashes**: SHA-256 for all integrity checking
- **Verification**: Minimum 5 random file samples required

### Compliance Features
- NIST SP 800-88 Rev.1 alignment
- Audit trail with operator and environment tracking
- Tamper-evident with cryptographic signatures
- Machine-readable JSON + human-readable PDF
- QR codes for easy verification

## Sample Output

When tests pass, you'll get:
1. `tests/sample_backup_certificate.json` - Valid certificate for inspection
2. `/tmp/test_backup_certificate.pdf` - Generated PDF certificate
3. Validation confirmation against the JSON schema

## Integration with Core

The core Rust application should:
1. Generate certificates matching this exact schema
2. Validate generated certificates before signing
3. Use the PDF generation logic for user-facing certificates
4. Include QR codes linking to the verification portal

## Next Steps

After testing passes:
1. Implement certificate generation in `/core` (Rust)
2. Wire PDF generation into Tauri UI 
3. Set up portal verification endpoint
4. Test end-to-end backup → wipe → verify flow