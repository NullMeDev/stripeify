#!/bin/bash

# Quick Discovery Mode Test - Core Functionality Only
set -e

cd /home/null/Desktop/Stripeify

echo "════════════════════════════════════════════════════════════"
echo "  QUICK DISCOVERY MODE TEST"
echo "════════════════════════════════════════════════════════════"
echo ""

# Test 1: Basic execution with 2 gates
echo "[TEST 1] Running discover with 2 gates (60s timeout)..."
timeout 60 ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_discover_cards.txt \
    --max-gates 2 \
    -o test_discovery_quick.json \
    2>&1 | tee test_discover_quick_output.log

EXIT_CODE=$?

echo ""
echo "Exit code: $EXIT_CODE"
echo ""

# Check results
if [ -f "test_discovery_quick.json" ]; then
    echo "✓ Output file created"
    echo "File size: $(wc -c < test_discovery_quick.json) bytes"
    echo ""
    echo "Content preview:"
    head -30 test_discovery_quick.json
    echo ""
    
    # Validate JSON
    if python3 -m json.tool test_discovery_quick.json > /dev/null 2>&1; then
        echo "✓ Valid JSON format"
    else
        echo "✗ Invalid JSON format"
    fi
else
    echo "✗ No output file created"
    echo ""
    echo "Last 30 lines of output:"
    tail -30 test_discover_quick_output.log
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  TEST COMPLETE"
echo "════════════════════════════════════════════════════════════"
