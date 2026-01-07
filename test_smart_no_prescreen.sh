#!/bin/bash
# Test with Smart Mode - Bypasses HTTP Pre-screening (Avoids 403)

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Smart Mode Testing - NO HTTP PRE-SCREENING"
echo "  Bypasses 403 errors by using browser directly"
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

# Create test file with first 5 gates
echo "Creating test file with 5 gates..."
head -5 full_test_gates.txt > test_5_gates_smart.txt
echo "✓ Created test_5_gates_smart.txt"

echo ""
echo "Configuration:"
echo "  Mode: SMART (no HTTP pre-screening)"
echo "  Gates: 5 (from full_test_gates.txt)"
echo "  Cards: All from full_test_cards.txt"
echo "  Auth-only: YES (FREE - no charges)"
echo "  Proxies: NONE"
echo ""

echo "Why Smart Mode?"
echo "  ✓ Skips HTTP pre-screening (avoids 403)"
echo "  ✓ Uses browser directly"
echo "  ✓ Intelligent card rotation"
echo "  ✓ Finds working card first, then uses it for all gates"
echo ""

echo "This will:"
echo "  1. Try multiple cards on first gate until one works"
echo "  2. Use that working card for all remaining gates"
echo "  3. When card dies, find next working card"
echo "  4. Take ~10-15 minutes for 5 gates"
echo ""

read -p "Press Enter to start testing..."

echo ""
echo "Starting smart mode test..."
echo ""

# Run smart mode (no pre-screening)
echo "y" | timeout 900 ./target/release/shopify_checker smart \
  --gates test_5_gates_smart.txt \
  --cards-file full_test_cards.txt \
  --auth-only=true \
  --max-gates 5 2>&1 | tee test_smart_results.log

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Test Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check results
if [ -f "smart_results.json" ]; then
    echo "✓ Results saved to: smart_results.json"
    echo "✓ Log saved to: test_smart_results.log"
    echo ""
    
    # Count valid gates
    valid_count=$(grep -c '"success":true' smart_results.json 2>/dev/null || echo "0")
    echo "Found $valid_count valid gate(s)"
    echo ""
    
    if [ "$valid_count" -gt 0 ]; then
        echo "✅ SUCCESS! Found working gates using smart mode"
        echo ""
        echo "Valid gates:"
        grep '"gate"' smart_results.json | cut -d'"' -f4 | sort -u
        echo ""
        echo "Next steps:"
        echo "1. Review smart_results.json for details"
        echo "2. Test more gates: head -20 full_test_gates.txt > test_20_gates_smart.txt"
        echo "3. Run: ./test_smart_no_prescreen.sh (edit to use test_20_gates_smart.txt)"
    else
        echo "⚠️  No valid gates found in this batch"
        echo ""
        echo "Try:"
        echo "1. Test different gates: tail -5 full_test_gates.txt > test_5_gates_smart.txt"
        echo "2. Run this script again"
    fi
else
    echo "⚠️  No results file found"
    echo "Check test_smart_results.log for errors"
    echo ""
    echo "Common issues:"
    echo "- ChromeDriver not running"
    echo "- Gates are actually dead"
    echo "- Network issues"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
