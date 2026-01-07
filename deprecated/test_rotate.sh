#!/bin/bash

# Test script for rotational gate mode
# Uses first 5 cards from test file

echo "🔄 Testing Rotational Gate Mode"
echo "================================"
echo ""

# Create test cards file with 5 cards
head -5 test_cards.txt > test_rotate_cards.txt

echo "✓ Created test file with 5 cards"
echo ""

# Show what we're testing
echo "Cards to test:"
cat test_rotate_cards.txt
echo ""

echo "Gates to use:"
echo "  - production_gates.json (15 gates)"
echo ""

echo "Starting rotational gate checker..."
echo "This will:"
echo "  1. Find a working gate first"
echo "  2. Use that gate for all 5 cards"
echo "  3. Rotate if gate fails 3 times"
echo ""

# Run the checker
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file test_rotate_cards.txt \
  --output test_rotate_results.json

echo ""
echo "✓ Test complete!"
echo ""
echo "Results saved to: test_rotate_results.json"
echo ""

# Show results if file exists
if [ -f test_rotate_results.json ]; then
    echo "Results summary:"
    cat test_rotate_results.json | jq -r '.[] | "\(.status): \(.card) on \(.gate)"' 2>/dev/null || cat test_rotate_results.json
fi

# Cleanup
rm -f test_rotate_cards.txt
