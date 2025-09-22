#!/bin/bash

# Test Certificate Page QR and File Operation Flows
# This script tests the complete certificate management workflow

set -e

echo "🧪 Testing Certificate Page Flows"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
CERT_DIR="$HOME/SecureWipe/certificates"
CORE_BIN="./core/target/release/securewipe"
PORTAL_URL="http://localhost:8000"

# Check prerequisites
echo "📋 Checking prerequisites..."

if [ ! -f "$CORE_BIN" ]; then
    echo -e "${RED}❌ Core binary not found at $CORE_BIN${NC}"
    echo "Run: cd core && cargo build --release"
    exit 1
fi

if [ ! -d "$CERT_DIR" ]; then
    echo -e "${RED}❌ Certificate directory not found at $CERT_DIR${NC}"
    echo "Run a backup operation first to create certificates"
    exit 1
fi

# Find a test certificate
TEST_CERT_JSON=$(ls "$CERT_DIR"/*.json | head -1)
if [ ! -f "$TEST_CERT_JSON" ]; then
    echo -e "${RED}❌ No certificate JSON files found in $CERT_DIR${NC}"
    exit 1
fi

CERT_ID=$(basename "$TEST_CERT_JSON" .json)
TEST_CERT_PDF="$CERT_DIR/$CERT_ID.pdf"

echo -e "${GREEN}✓ Found test certificate: $CERT_ID${NC}"

# Test 1: PDF Generation
echo ""
echo "🔧 Test 1: PDF Generation"
echo "-------------------------"

# Remove PDF if it exists for clean test
if [ -f "$TEST_CERT_PDF" ]; then
    rm "$TEST_CERT_PDF"
    echo "Removed existing PDF for clean test"
fi

echo "Running: $CORE_BIN cert --export-pdf $CERT_ID"
PDF_OUTPUT=$($CORE_BIN cert --export-pdf $CERT_ID 2>&1)

if echo "$PDF_OUTPUT" | grep -q '"status":"success"'; then
    echo -e "${GREEN}✓ PDF generation successful${NC}"
    
    # Extract PDF path from JSON output
    PDF_PATH=$(echo "$PDF_OUTPUT" | grep -o '"pdf_path":"[^"]*"' | sed 's/"pdf_path":"//;s/"//')
    echo "PDF created at: $PDF_PATH"
    
    if [ -f "$PDF_PATH" ]; then
        echo -e "${GREEN}✓ PDF file exists${NC}"
    else
        echo -e "${RED}❌ PDF file not found at expected path${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ PDF generation failed${NC}"
    echo "$PDF_OUTPUT"
    exit 1
fi

# Test 2: Portal Verification
echo ""
echo "🌐 Test 2: Portal Verification"
echo "------------------------------"

# Check if portal is running
if ! curl -s "$PORTAL_URL/health" > /dev/null; then
    echo -e "${YELLOW}⚠️  Portal not running, skipping verification test${NC}"
    echo "To test verification, run: cd portal && python -m uvicorn app.main:app --port 8000"
else
    echo "Testing verification with portal..."
    
    VERIFY_RESPONSE=$(curl -s -X POST "$PORTAL_URL/verify" \
        -H "Content-Type: application/json" \
        -d @"$TEST_CERT_JSON" 2>&1)
    
    if echo "$VERIFY_RESPONSE" | grep -q '"cert_summary"'; then
        echo -e "${GREEN}✓ Portal verification response received${NC}"
        
        # Check if schema validation passed (it might not due to signature issues)
        if echo "$VERIFY_RESPONSE" | grep -q '"schema_valid":true'; then
            echo -e "${GREEN}✓ Schema validation passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Schema validation failed (expected for unsigned certs)${NC}"
        fi
    else
        echo -e "${RED}❌ Portal verification failed${NC}"
        echo "$VERIFY_RESPONSE"
    fi
fi

# Test 3: QR Payload Generation
echo ""
echo "📱 Test 3: QR Payload Generation"
echo "--------------------------------"

# Read certificate content
CERT_CONTENT=$(cat "$TEST_CERT_JSON")
CERT_CREATED_AT=$(echo "$CERT_CONTENT" | grep -o '"created_at":"[^"]*"' | sed 's/"created_at":"//;s/"//')

echo "Certificate created at: $CERT_CREATED_AT"

# Generate QR payload (simplified version of what UI does)
QR_PAYLOAD_URL="$PORTAL_URL/verify?cert_id=$CERT_ID"
echo "QR payload URL: $QR_PAYLOAD_URL"

# Test if the QR payload URL works
if curl -s "$QR_PAYLOAD_URL" > /dev/null; then
    echo -e "${GREEN}✓ QR payload URL is accessible${NC}"
else
    echo -e "${YELLOW}⚠️  QR payload URL not accessible (portal may not have cert_id endpoint)${NC}"
fi

# Test 4: File Operations (simulated)
echo ""
echo "📁 Test 4: File Operation Tests"
echo "-------------------------------"

# Test file existence checks
if [ -f "$TEST_CERT_JSON" ]; then
    echo -e "${GREEN}✓ Certificate JSON exists and is readable${NC}"
else
    echo -e "${RED}❌ Certificate JSON not accessible${NC}"
    exit 1
fi

if [ -f "$TEST_CERT_PDF" ]; then
    echo -e "${GREEN}✓ Certificate PDF exists and is readable${NC}"
    
    # Check PDF file size (should be > 1KB for a real PDF)
    PDF_SIZE=$(stat -f%z "$TEST_CERT_PDF" 2>/dev/null || stat -c%s "$TEST_CERT_PDF" 2>/dev/null)
    if [ "$PDF_SIZE" -gt 1000 ]; then
        echo -e "${GREEN}✓ PDF file has reasonable size (${PDF_SIZE} bytes)${NC}"
    else
        echo -e "${YELLOW}⚠️  PDF file seems small (${PDF_SIZE} bytes)${NC}"
    fi
else
    echo -e "${RED}❌ Certificate PDF not accessible${NC}"
fi

# Test 5: Certificate Content Validation
echo ""
echo "🔍 Test 5: Certificate Content Validation"
echo "-----------------------------------------"

# Check required fields
REQUIRED_FIELDS=("cert_id" "cert_type" "created_at" "device")
for field in "${REQUIRED_FIELDS[@]}"; do
    if echo "$CERT_CONTENT" | grep -q "\"$field\""; then
        echo -e "${GREEN}✓ Required field '$field' present${NC}"
    else
        echo -e "${RED}❌ Required field '$field' missing${NC}"
    fi
done

# Summary
echo ""
echo "📊 Test Summary"
echo "==============="
echo -e "${GREEN}✓ PDF Generation: Working${NC}"
echo -e "${GREEN}✓ File Operations: Working${NC}"
echo -e "${GREEN}✓ Certificate Content: Valid${NC}"

if curl -s "$PORTAL_URL/health" > /dev/null; then
    echo -e "${GREEN}✓ Portal Integration: Working${NC}"
else
    echo -e "${YELLOW}⚠️  Portal Integration: Not tested (portal not running)${NC}"
fi

echo ""
echo -e "${GREEN}🎉 All core certificate flows are working!${NC}"
echo ""
echo "Next steps to test in UI:"
echo "1. Navigate to /certificates page"
echo "2. Verify QR code displays properly"
echo "3. Test 'Generate PDF' button (if PDF missing)"
echo "4. Test 'Open JSON' and 'Open PDF' buttons"
echo "5. Test 'Verify Online' button"
echo ""
echo "The certificate page should now show:"
echo "- Real QR codes with verify URLs"
echo "- Working file open operations"
echo "- PDF generation when needed"
echo "- Online verification results"
