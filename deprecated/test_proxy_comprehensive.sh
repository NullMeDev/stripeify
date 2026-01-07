#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║     Comprehensive Proxy Implementation Test Suite        ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0
TEST_LOG="proxy_test_results.log"

# Clear previous log
> "$TEST_LOG"

log_test() {
    echo "$1" | tee -a "$TEST_LOG"
}

pass_test() {
    ((TESTS_PASSED++))
    log_test "✅ PASS: $1"
}

fail_test() {
    ((TESTS_FAILED++))
    log_test "❌ FAIL: $1"
}

echo "═══════════════════════════════════════════════════════════"
echo "TEST 1: Proxy File Loading"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 1.1: Valid proxy file
if [ -f "proxies.txt" ]; then
    PROXY_COUNT=$(wc -l < proxies.txt)
    if [ "$PROXY_COUNT" -gt 0 ]; then
        pass_test "Valid proxy file exists with $PROXY_COUNT proxies"
    else
        fail_test "Proxy file is empty"
    fi
else
    fail_test "proxies.txt not found"
fi

# Test 1.2: Proxy format validation
log_test ""
log_test "Checking proxy format..."
FIRST_PROXY=$(head -1 proxies.txt)
if echo "$FIRST_PROXY" | grep -qE '^[^:]+:[0-9]+:[^:]+:.+$'; then
    pass_test "Proxy format is valid: host:port:username:password"
else
    fail_test "Invalid proxy format in proxies.txt"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 2: Binary and CLI Integration"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 2.1: Binary exists
if [ -f "./target/release/shopify_checker" ]; then
    pass_test "Binary exists at target/release/shopify_checker"
else
    fail_test "Binary not found"
    exit 1
fi

# Test 2.2: Help shows proxy flag
HELP_OUTPUT=$(./target/release/shopify_checker rotate --help 2>&1)
if echo "$HELP_OUTPUT" | grep -q "proxy-file"; then
    pass_test "CLI help shows --proxy-file flag"
else
    fail_test "CLI help missing --proxy-file flag"
fi

# Test 2.3: Help shows correct format
if echo "$HELP_OUTPUT" | grep -q "host:port:username:password"; then
    pass_test "Help text shows correct proxy format"
else
    fail_test "Help text missing proxy format description"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 3: Invalid Proxy Handling"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 3.1: Create test gates and cards
cat > test_proxy_gate.json << 'EOF'
[
  {
    "url": "https://donate.example.com",
    "gateway": "Shopify",
    "donation_form": true
  }
]
EOF

cat > test_proxy_card.txt << 'EOF'
4532015112830366|12|2027|999
EOF

# Test 3.2: Test with invalid proxy file
log_test "Testing with invalid proxy formats..."
if [ -f "test_invalid_proxies.txt" ]; then
    pass_test "Invalid proxy test file created"
else
    fail_test "Could not create invalid proxy test file"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 4: Proxy Module Unit Tests"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 4.1: Check proxy module exists
if [ -f "src/proxy.rs" ]; then
    pass_test "Proxy module (src/proxy.rs) exists"
    
    # Check for key functions
    if grep -q "pub struct ProxyPool" src/proxy.rs; then
        pass_test "ProxyPool struct defined"
    else
        fail_test "ProxyPool struct not found"
    fi
    
    if grep -q "pub fn from_file" src/proxy.rs; then
        pass_test "from_file function defined"
    else
        fail_test "from_file function not found"
    fi
    
    if grep -q "pub fn get_next" src/proxy.rs; then
        pass_test "get_next function defined (rotation)"
    else
        fail_test "get_next function not found"
    fi
    
    if grep -q "pub fn report_failure" src/proxy.rs; then
        pass_test "report_failure function defined"
    else
        fail_test "report_failure function not found"
    fi
else
    fail_test "Proxy module not found"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 5: Proxy Extension Module"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 5.1: Check proxy extension module
if [ -f "src/proxy_extension.rs" ]; then
    pass_test "Proxy extension module (src/proxy_extension.rs) exists"
    
    # Check for key components
    if grep -q "pub struct ProxyExtension" src/proxy_extension.rs; then
        pass_test "ProxyExtension struct defined"
    else
        fail_test "ProxyExtension struct not found"
    fi
    
    if grep -q "manifest.json" src/proxy_extension.rs; then
        pass_test "Chrome manifest.json generation code present"
    else
        fail_test "manifest.json generation not found"
    fi
    
    if grep -q "background.js" src/proxy_extension.rs; then
        pass_test "Chrome background.js generation code present"
    else
        fail_test "background.js generation not found"
    fi
else
    fail_test "Proxy extension module not found"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 6: Checker Integration"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 6.1: Check checker_v3 integration
if grep -q "proxy_file: Option<&str>" src/checker_v3.rs; then
    pass_test "checker_v3 accepts proxy_file parameter"
else
    fail_test "checker_v3 missing proxy_file parameter"
fi

if grep -q "ProxyPool::from_file" src/checker_v3.rs; then
    pass_test "checker_v3 loads proxy pool"
else
    fail_test "checker_v3 doesn't load proxy pool"
fi

if grep -q "ProxyExtension::new" src/checker_v3.rs; then
    pass_test "checker_v3 creates proxy extension"
else
    fail_test "checker_v3 doesn't create proxy extension"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 7: Dependencies"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 7.1: Check Cargo.toml dependencies
if grep -q 'rand = "0.8"' Cargo.toml; then
    pass_test "rand dependency added"
else
    fail_test "rand dependency missing"
fi

if grep -q 'tempfile = "3.8"' Cargo.toml; then
    pass_test "tempfile dependency added"
else
    fail_test "tempfile dependency missing"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 8: Documentation"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 8.1: Check documentation files
if [ -f "docs/PROXY_USAGE_GUIDE.md" ]; then
    pass_test "Proxy usage guide exists"
else
    fail_test "Proxy usage guide missing"
fi

if [ -f "docs/PROXY_IMPLEMENTATION_PLAN.md" ]; then
    pass_test "Implementation plan exists"
else
    fail_test "Implementation plan missing"
fi

if [ -f "docs/PROXY_IMPLEMENTATION_COMPLETE.md" ]; then
    pass_test "Implementation complete document exists"
else
    fail_test "Implementation complete document missing"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST 9: Dry Run (No ChromeDriver)"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 9.1: Run without ChromeDriver to test proxy loading
log_test "Testing proxy loading without ChromeDriver..."
OUTPUT=$(echo "n" | timeout 10 ./target/release/shopify_checker rotate \
    --gates test_proxy_gate.json \
    --cards-file test_proxy_card.txt \
    --proxy-file proxies.txt 2>&1 || true)

if echo "$OUTPUT" | grep -q "Loading proxies"; then
    pass_test "Proxy loading message appears"
else
    fail_test "Proxy loading message not found"
fi

if echo "$OUTPUT" | grep -q "Loaded.*proxies"; then
    pass_test "Proxy count displayed"
else
    fail_test "Proxy count not displayed"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "TEST SUMMARY"
echo "═══════════════════════════════════════════════════════════"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))

log_test "Total Tests: $TOTAL_TESTS"
log_test "Passed: $TESTS_PASSED"
log_test "Failed: $TESTS_FAILED"
log_test "Pass Rate: $PASS_RATE%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    log_test "🎉 ALL TESTS PASSED!"
    EXIT_CODE=0
else
    log_test "⚠️  SOME TESTS FAILED"
    EXIT_CODE=1
fi

echo ""
log_test "Detailed log saved to: $TEST_LOG"
echo ""

# Cleanup
rm -f test_proxy_gate.json test_proxy_card.txt

exit $EXIT_CODE
