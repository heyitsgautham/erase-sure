#!/usr/bin/env bash
set -e

echo "SecureWipe Certificate Testing Suite"
echo "===================================="

# Check if we're in the right directory
if [[ ! -f "certs/schemas/backup_schema.json" ]]; then
    echo "❌ Error: Must run from project root directory"
    echo "Expected to find: certs/schemas/backup_schema.json"
    exit 1
fi

echo "📁 Project root: $(pwd)"
echo "📄 Schema file: certs/schemas/backup_schema.json"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: python3 not found"
    exit 1
fi

echo "🐍 Python version: $(python3 --version)"

# Check if jsonschema is available
if ! python3 -c "import jsonschema" 2>/dev/null; then
    echo "⚠️  Installing jsonschema..."
    pip3 install jsonschema
fi

echo ""
echo "Running JSON Schema Validation Tests..."
echo "--------------------------------------"
python3 tests/test_backup_schema.py

echo ""
echo "Checking if PDF dependencies are available..."
echo "--------------------------------------------"

# Check for PDF dependencies
pdf_deps_available=true

for dep in reportlab qrcode; do
    if ! python3 -c "import $dep" 2>/dev/null; then
        echo "⚠️  Missing: $dep"
        pdf_deps_available=false
    else
        echo "✓ Available: $dep"
    fi
done

if [[ "$pdf_deps_available" == "true" ]]; then
    echo ""
    echo "Running PDF Generation Tests..."
    echo "------------------------------"
    python3 core/python/backup_pdf.py
else
    echo ""
    echo "📋 To test PDF generation, install dependencies:"
    echo "   pip3 install -r tests/requirements-test.txt"
fi

echo ""
echo "✅ Testing complete!"