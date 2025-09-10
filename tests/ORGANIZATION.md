# Tests Directory Organization

This directory contains all testing files for the SecureWipe project, organized for better maintainability.

## Directory Structure

```
tests/
├── README.md                     # Original test documentation
├── ORGANIZATION.md              # This file - organization guide
├── requirements-test.txt        # Python dependencies for testing
│
├── scripts/                     # Shell test scripts
│   ├── test_backup.sh
│   ├── test_backup_integration.sh
│   ├── test_certificates.sh
│   ├── test_pdf_generation.sh
│   └── test_wipe_certificates.sh
│
├── outputs/                     # Generated test outputs
│   ├── test_backup_certificate_fixed.pdf
│   └── test_wipe_certificate_fixed.pdf
│
├── samples/                     # Sample data files for testing
│   ├── sample_backup_certificate.json
│   ├── sample_wipe_certificate.json
│   └── test_backup_sample_123.json
│
└── *.py                        # Python test files
    ├── test_backup_schema.py
    ├── test_pdf_certificates.py
    ├── test_qr_codes.py
    ├── test_qr_simple.py
    ├── test_wipe_pdf_certificates.py
    └── test_wipe_schema.py
```

## File Categories

### Shell Scripts (`scripts/`)
- **test_backup.sh**: Tests backup functionality
- **test_backup_integration.sh**: Integration tests for backup workflow
- **test_certificates.sh**: Certificate generation and validation tests
- **test_pdf_generation.sh**: PDF certificate generation tests
- **test_wipe_certificates.sh**: Wipe certificate tests

### Test Outputs (`outputs/`)
- Generated PDF certificates from test runs
- Temporary files from integration tests
- **Note**: This directory may contain files that change frequently during testing

### Sample Data (`samples/`)
- Example JSON certificate files
- Test data files used across multiple tests
- Reference implementations for validation

### Python Tests (root level)
- Unit tests for specific components
- Schema validation tests
- PDF generation and QR code tests

## Usage

To run all tests:
```bash
# From project root
cd tests/

# Run Python tests
python -m pytest *.py

# Run shell script tests
./scripts/test_certificates.sh
```

## Maintenance

- Keep outputs/ clean by removing old test files
- Update samples/ when schema changes occur
- Ensure scripts/ permissions are executable
