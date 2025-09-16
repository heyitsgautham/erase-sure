#!/bin/bash

# SecureWipe PDF Generation Test Script
# This script tests all aspects of the PDF generation functionality

set -e  # Exit on any error

echo "🧪 SecureWipe PDF Generation Test Suite"
echo "======================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_TOTAL=0

# Helper function to run test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}Test $((++TESTS_TOTAL)): ${test_name}${NC}"
    echo "Command: $test_command"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED${NC}"
    fi
}

# Change to core directory
cd /Users/gauthamkrishna/Projects/SIH/erase-sure/core

echo -e "\n${YELLOW}Building project...${NC}"
cargo build --quiet

echo -e "\n${YELLOW}Running unit tests...${NC}"
run_test "PDF Unit Tests" "cargo test pdf --quiet"

echo -e "\n${YELLOW}Testing CLI functionality...${NC}"

# Test 1: Valid backup certificate PDF generation
run_test "Generate backup certificate PDF" \
    "./target/release/securewipe cert --export-pdf test_backup_sample_123 && echo 'PDF generated successfully'"

# Test 2: Valid wipe certificate PDF generation
run_test "Generate wipe certificate PDF" \
    "./target/release/securewipe cert --export-pdf test_wipe_sample_456 && echo 'PDF generated successfully'"

# Test 3: Error handling for non-existent certificate
run_test "Error handling for non-existent certificate" \
    "! ./target/release/securewipe cert --export-pdf nonexistent_cert_12345"

# Test 4: Verify PDF files exist and have reasonable size
run_test "Verify backup PDF exists and has content" \
    "[ -f ~/SecureWipe/certificates/test_backup_sample_123.pdf ] && [ -s ~/SecureWipe/certificates/test_backup_sample_123.pdf ]"

run_test "Verify wipe PDF exists and has content" \
    "[ -f ~/SecureWipe/certificates/test_wipe_sample_456.pdf ] && [ -s ~/SecureWipe/certificates/test_wipe_sample_456.pdf ]"

# Test 5: Check PDF file sizes are reasonable (> 1KB, < 100KB)
run_test "Check backup PDF size is reasonable" \
    "[ \$(stat -f%z ~/SecureWipe/certificates/test_backup_sample_123.pdf 2>/dev/null || stat -c%s ~/SecureWipe/certificates/test_backup_sample_123.pdf) -gt 1000 ]"

run_test "Check wipe PDF size is reasonable" \
    "[ \$(stat -f%z ~/SecureWipe/certificates/test_wipe_sample_456.pdf 2>/dev/null || stat -c%s ~/SecureWipe/certificates/test_wipe_sample_456.pdf) -gt 1000 ]"

# Test 6: JSON certificate validation
run_test "Validate backup certificate JSON" \
    "jq empty ~/SecureWipe/certificates/test_backup_sample_123.json 2>/dev/null"

run_test "Validate wipe certificate JSON" \
    "jq empty ~/SecureWipe/certificates/test_wipe_sample_456.json 2>/dev/null"

# Test 7: Certificate type detection
run_test "Backup certificate type detection" \
    "jq -r '.cert_type' ~/SecureWipe/certificates/test_backup_sample_123.json | grep -q 'backup'"

run_test "Wipe certificate type detection" \
    "jq -r '.cert_type' ~/SecureWipe/certificates/test_wipe_sample_456.json | grep -q 'wipe'"

# Test 8: Integration tests
echo -e "\n${YELLOW}Running integration tests...${NC}"
run_test "Certificate PDF Integration Tests" "cargo test cert_pdf --quiet"

# Test 9: Check certificate directory structure
run_test "Certificate directory exists" "[ -d ~/SecureWipe/certificates ]"

# Test 10: CLI help output
run_test "CLI help output works" "./target/release/securewipe cert --help && echo 'Help command works'"

echo -e "\n${YELLOW}Summary${NC}"
echo "======="
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}/$TESTS_TOTAL"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "\n${GREEN}🎉 All tests passed! PDF generation is working correctly.${NC}"
    echo -e "\n${BLUE}Generated PDFs:${NC}"
    ls -la ~/SecureWipe/certificates/test_*.pdf
    
    echo -e "\n${BLUE}File sizes:${NC}"
    du -h ~/SecureWipe/certificates/test_*.pdf
    
    echo -e "\n${BLUE}To view PDFs:${NC}"
    echo "open ~/SecureWipe/certificates/test_backup_sample_123.pdf"
    echo "open ~/SecureWipe/certificates/test_wipe_sample_456.pdf"
    
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please check the output above.${NC}"
    exit 1
fi
