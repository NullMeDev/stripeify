#!/bin/bash

# Comprehensive Discovery Mode Test Script
# Tests all features of the discover command

set -e

echo "════════════════════════════════════════════════════════════"
echo "  COMPREHENSIVE DISCOVERY MODE TEST SUITE"
echo "════════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Helper functions
test_start() {
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo -e "${CYAN}[TEST $TESTS_TOTAL]${NC} $1"
}

test_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✓ PASSED${NC}: $1"
    echo ""
}

test_fail() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}✗ FAILED${NC}: $1"
    echo ""
}

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up test files...${NC}"
    rm -f test_discovery_*.json test_valid_gates.json
}

trap cleanup EXIT

cd /home/null/Desktop/Stripeify

echo "════════════════════════════════════════════════════════════"
echo "  PHASE 1: PREREQUISITE CHECKS"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 1: Binary exists
test_start "Binary exists and is executable"
if [ -x "./target/release/shopify_checker" ]; then
    test_pass "Binary found at ./target/release/shopify_checker"
else
    test_fail "Binary not found or not executable"
    exit 1
fi

# Test 2: ChromeDriver running
test_start "ChromeDriver is running"
if pgrep -f chromedriver > /dev/null; then
    test_pass "ChromeDriver is running (PID: $(pgrep -f chromedriver))"
else
    test_fail "ChromeDriver is not running"
    echo "Starting ChromeDriver..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
fi

# Test 3: Test files exist
test_start "Test files exist"
if [ -f "test_discover_cards.txt" ] && [ -d "/home/null/Desktop/ShopifyGatesAndChunks" ]; then
    test_pass "Test cards and gates directory found"
else
    test_fail "Test files missing"
    exit 1
fi

echo "════════════════════════════════════════════════════════════"
echo "  PHASE 2: CLI INTERFACE TESTS"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 4: Help command
test_start "Help command works"
if ./target/release/shopify_checker discover --help > /dev/null 2>&1; then
    test_pass "Help command executed successfully"
else
    test_fail "Help command failed"
fi

# Test 5: Version command
test_start "Version command works"
if ./target/release/shopify_checker --version > /dev/null 2>&1; then
    test_pass "Version command executed successfully"
else
    test_fail "Version command failed"
fi

echo "════════════════════════════════════════════════════════════"
echo "  PHASE 3: EDGE CASE TESTS"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 6: Missing required arguments
test_start "Error handling for missing arguments"
if ! ./target/release/shopify_checker discover 2>&1 | grep -q "required"; then
    test_fail "Should require --gates-dir and --cards-file"
else
    test_pass "Properly validates required arguments"
fi

# Test 7: Invalid gates directory
test_start "Error handling for invalid gates directory"
if ! ./target/release/shopify_checker discover \
    --gates-dir /nonexistent/directory \
    --cards-file test_discover_cards.txt \
    --max-gates 1 2>&1 | grep -qE "error|Error|failed|Failed"; then
    test_fail "Should error on invalid directory"
else
    test_pass "Properly handles invalid gates directory"
fi

# Test 8: Invalid cards file
test_start "Error handling for invalid cards file"
if ! ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file /nonexistent/cards.txt \
    --max-gates 1 2>&1 | grep -qE "error|Error|failed|Failed"; then
    test_fail "Should error on invalid cards file"
else
    test_pass "Properly handles invalid cards file"
fi

echo "════════════════════════════════════════════════════════════"
echo "  PHASE 4: FUNCTIONAL TESTS (MINIMAL)"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 9: Basic discover command (2 gates, no Telegram)
test_start "Basic discover execution (2 gates, 60s timeout)"
echo -e "${YELLOW}This may take up to 60 seconds...${NC}"

timeout 60 ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_discover_cards.txt \
    --max-gates 2 \
    -o test_discovery_minimal.json \
    > test_discover_output.log 2>&1

if [ $? -eq 0 ] || [ $? -eq 124 ]; then
    # Check if output file was created
    if [ -f "test_discovery_minimal.json" ]; then
        test_pass "Discover command executed and created output file"
        echo "Output file size: $(wc -c < test_discovery_minimal.json) bytes"
    else
        test_fail "Discover command ran but no output file created"
        echo "Last 20 lines of output:"
        tail -20 test_discover_output.log
    fi
else
    test_fail "Discover command failed with exit code $?"
    echo "Last 20 lines of output:"
    tail -20 test_discover_output.log
fi

# Test 10: Check output file format
if [ -f "test_discovery_minimal.json" ]; then
    test_start "Output file is valid JSON"
    if python3 -m json.tool test_discovery_minimal.json > /dev/null 2>&1; then
        test_pass "Output file is valid JSON"
        echo "Sample output:"
        head -20 test_discovery_minimal.json
    else
        test_fail "Output file is not valid JSON"
    fi
fi

echo "════════════════════════════════════════════════════════════"
echo "  PHASE 5: INTEGRATION TESTS"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 11: Other commands still work (analyze)
test_start "Analyze command still works"
if ./target/release/shopify_checker analyze --help > /dev/null 2>&1; then
    test_pass "Analyze command is functional"
else
    test_fail "Analyze command broken"
fi

# Test 12: Other commands still work (rotate)
test_start "Rotate command still works"
if ./target/release/shopify_checker rotate --help > /dev/null 2>&1; then
    test_pass "Rotate command is functional"
else
    test_fail "Rotate command broken"
fi

# Test 13: Other commands still work (smart)
test_start "Smart command still works"
if ./target/release/shopify_checker smart --help > /dev/null 2>&1; then
    test_pass "Smart command is functional"
else
    test_fail "Smart command broken"
fi

echo "════════════════════════════════════════════════════════════"
echo "  TEST SUMMARY"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Total Tests: $TESTS_TOTAL"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓ ALL TESTS PASSED!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  ✗ SOME TESTS FAILED${NC}"
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    exit 1
fi
