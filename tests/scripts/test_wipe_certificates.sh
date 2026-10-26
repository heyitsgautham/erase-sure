#!/bin/bash

# SecureWipe Wipe Certificate Testing Suite
# Tests both JSON schema validation and PDF generation for wipe certificates

set -e

echo "SecureWipe Wipe Certificate Testing Suite"
echo "========================================"

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_FILE="$PROJECT_ROOT/certs/schemas/wipe_schema.json"

echo "📁 Project root: $PROJECT_ROOT"
echo "📄 Schema file: $SCHEMA_FILE"
echo "🐍 Python version: $(python3 --version)"
echo

# Check if schema file exists
if [[ ! -f "$SCHEMA_FILE" ]]; then
    echo "❌ Schema file not found: $SCHEMA_FILE"
    exit 1
fi

echo "Running JSON Schema Validation Tests..."
echo "--------------------------------------"

# Run JSON schema validation tests
if python3 "$PROJECT_ROOT/tests/test_wipe_schema.py"; then
    echo
    echo "✅ JSON Schema validation: PASSED"
else
    echo
    echo "❌ JSON Schema validation: FAILED"
    echo "Please fix the schema issues before proceeding."
    exit 1
fi

echo
echo "Next steps:"
echo "1. Install PDF dependencies: pip install -r tests/requirements-test.txt"
echo "2. Run PDF generation test: python core/python/wipe_pdf.py"
echo "3. Verify certificate in portal: POST to /verify endpoint"

echo
echo "Checking if PDF dependencies are available..."
echo "--------------------------------------------"

# Check for PDF dependencies
MISSING_DEPS=()

if ! python3 -c "import reportlab" 2>/dev/null; then
    MISSING_DEPS+=("reportlab")
else
    echo "✓ Available: reportlab"
fi

if ! python3 -c "import qrcode" 2>/dev/null; then
    MISSING_DEPS+=("qrcode")
else
    echo "✓ Available: qrcode"
fi

if ! python3 -c "import PIL" 2>/dev/null; then
    MISSING_DEPS+=("pillow")
else
    echo "✓ Available: pillow"
fi

if [[ ${#MISSING_DEPS[@]} -gt 0 ]]; then
    echo
    echo "⚠️  Missing: ${MISSING_DEPS[*]}"
    echo
    echo "📋 To test PDF generation, install dependencies:"
    echo "   pip3 install -r tests/requirements-test.txt"
else
    echo
    echo "🚀 All PDF dependencies available! Running PDF generation test..."
    echo "----------------------------------------------------------------"
    
    if python3 "$PROJECT_ROOT/core/python/wipe_pdf.py"; then
        echo
        echo "✅ PDF Generation: PASSED"
        echo "📄 Check the generated PDF: /tmp/test_wipe_certificate.pdf"
    else
        echo
        echo "❌ PDF Generation: FAILED"
        exit 1
    fi
fi

echo
echo "✅ Testing complete!"
