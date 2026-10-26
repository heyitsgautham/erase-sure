# SecureWipe Verification Portal

A FastAPI-based service for validating JSON certificates (backup or wipe) against their schemas.

## Features

- **Schema Validation**: Validates certificates against `/certs/schemas/backup_schema.json` and `/certs/schemas/wipe_schema.json`
- **Certificate Type Detection**: Automatically detects backup vs wipe certificates via `cert_type` field
- **RESTful API**: Simple POST endpoint for validation with JSON response
- **Web Interface**: HTML page with usage instructions and examples
- **Comprehensive Testing**: Unit tests for valid and invalid certificate payloads

## Quick Start

### 1. Install Dependencies

```bash
cd portal
pip install -r requirements.txt
```

### 2. Run the Server

```bash
# Using the run script
python run.py --reload

# Or directly with uvicorn
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

### 3. Access the Service

- **Web Interface**: http://localhost:8000/
- **API Documentation**: http://localhost:8000/docs (FastAPI auto-generated)
- **Health Check**: http://localhost:8000/health

## API Endpoints

### POST /verify

Validates a JSON certificate against the appropriate schema.

**Request:**
- Content-Type: `application/json`
- Body: Raw JSON certificate data

**Response:**
```json
{
  "schema_valid": true,
  "errors": [],
  "cert_summary": {
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "cert_type": "backup",
    "device_model": "Samsung SSD 980 PRO 1TB",
    "destination": "usb (External Drive)"
  }
}
```

**Example Usage:**
```bash
# Validate a backup certificate
curl -X POST "http://localhost:8000/verify" \
     -H "Content-Type: application/json" \
     -d @examples/valid_backup_cert.json

# Validate a wipe certificate
curl -X POST "http://localhost:8000/verify" \
     -H "Content-Type: application/json" \
     -d @examples/valid_wipe_cert.json
```

### GET /

Returns an HTML page with usage instructions and examples.

### GET /verify/{cert_id}

Placeholder endpoint for future certificate lookup functionality.

### GET /health

Health check endpoint returning service status and schema loading status.

## Testing

Run the suite from `portal/`.
`conftest.py` creates a throwaway key pair, so no key needs to exist first.

```bash
# Run all tests with the coverage floor from pyproject.toml
python -m pytest --cov

# Run one area
python -m pytest test_signature.py -v
python -m pytest test_routes.py::TestCertificateVerification -v
```

## Certificate Types

### Backup Certificates
- **Type**: `"backup"`
- **Key Fields**: `device`, `files_summary`, `destination`, `crypto`
- **Summary**: Shows destination type and label

### Wipe Certificates
- **Type**: `"wipe"`
- **Key Fields**: `device`, `policy`, `commands`, `verify`
- **Summary**: Shows NIST level and method

## Schema Validation

The service validates certificates against JSON schemas in `/certs/schemas/`:
- `backup_schema.json`: For backup certificates
- `wipe_schema.json`: For wipe certificates

Validation includes:
- Required field presence
- Data type validation
- Enum value validation (e.g., NIST levels: CLEAR, PURGE)
- String format validation (e.g., SHA-256 hashes, date-time)
- Signature algorithm validation (Ed25519 only)
- Public key ID validation (must be "sih_root_v1")

## Error Handling

The service provides detailed error messages for:
- Missing required fields
- Invalid data types
- Invalid enum values
- Malformed JSON
- Schema validation failures

## Development

### Project Structure
```
portal/
├── app/
│   ├── main.py          # FastAPI app and routes
│   ├── keys.py          # Trusted public key and certificate schemas, loaded at import
│   ├── signature.py     # RFC 8785 canonical JSON, Ed25519 check, certificate hash
│   ├── verification.py  # validate_certificate and the response models
│   └── pages.py         # The help page served at /
├── examples/            # Sample certificate files
├── sample_certs.py      # Certificates the tests start from
├── test_signature.py    # Canonicalization and signature tests
├── test_verification.py # validate_certificate tests
├── test_routes.py       # HTTP route tests through the FastAPI test client
├── test_cli_interop.py  # A certificate signed by the Rust CLI verifies here
├── test_keygen.py       # Key generation script tests
├── run.py               # Server startup script
├── requirements.txt     # Direct dependencies
└── requirements-lock.txt # Hashed lockfile CI installs from
```

### Adding New Features

1. **New Validation Rules**: Modify `validate_certificate()` in `app/verification.py`
2. **New Certificate Types**: Add schema files and update type detection logic
3. **New Endpoints**: Add routes to `app/main.py`

### Security Considerations

- Service is stateless (no database in MVP)
- Only validates public data and signatures
- Never handles private keys
- CORS enabled for development (configure for production)

## Future Enhancements

- **Database Integration**: Store and lookup certificates by ID
- **Ed25519 Signature Verification**: Full cryptographic validation
- **Certificate Chain Validation**: Verify backup-to-wipe linkage
- **Batch Validation**: Process multiple certificates
- **API Authentication**: Add security for production use
