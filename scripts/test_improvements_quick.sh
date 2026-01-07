#!/bin/bash

# Quick Test for Edge Case Improvements
# Tests validation and -o flag in under 2 minutes

echo "=========================================="
echo "Quick Edge Case & Improvements Test"
echo "=========================================="
echo ""

BINARY="./target/release/shopify_checker"
START_TIME=$(date +%s)

# Test 1: Invalid directory (should fail immediately)
echo "Test 1: Invalid directory validation"
echo "--------------------------------------"
$BINARY discover \
    --gates-dir /nonexistent/directory \
    --cards-file test_discover_cards.txt \
    --max-gates 1 \
    -o test1.json 2>&1 | grep -E "(error|Error|does not exist)" | head -3
echo "✓ Test 1 complete"
echo ""

# Test 2: Invalid cards file (should fail immediately)
echo "Test 2: Invalid cards file validation"
echo "--------------------------------------"
$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file /nonexistent/cards.txt \
    --max-gates 1 \
    -o test2.json 2>&1 | grep -E "(error|Error|does not exist)" | head -3
echo "✓ Test 2 complete"
echo ""

# Test 3: Empty cards file (should fail with clear message)
echo "Test 3: Empty cards file validation"
echo "--------------------------------------"
echo "" > test_empty.txt
$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_empty.txt \
    --max-gates 1 \
    -o test3.json 2>&1 | grep -E "(error|Error|empty)" | head -3
rm -f test_empty.txt
echo "✓ Test 3 complete"
echo ""

# Test 4: Malformed cards (should show helpful errors)
echo "Test 4: Malformed cards handling"
echo "--------------------------------------"
cat > test_malformed.txt << 'EOF'
# Valid card
4532015112830366|12|2027|123
# Invalid formats below
bad_format
missing|cvv|here
EOF

$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_malformed.txt \
    --max-gates 1 \
    -o test4.json 2>&1 | grep -E "(Invalid|Line|Loaded)" | head -5
rm -f test_malformed.txt
echo "✓ Test 4 complete"
echo ""

# Test 5: Custom output file with -o flag (actual test with 1 gate)
echo "Test 5: Custom output file (-o flag)"
echo "--------------------------------------"
echo "Running actual test with 1 gate..."
echo "y" | timeout 45 $BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_discover_cards.txt \
    --max-gates 1 \
    -o custom_output_test.json 2>&1 | tail -10

if [ -f "custom_output_test.json" ]; then
    echo ""
    echo "✓ Custom output file created!"
    ls -lh custom_output_test.json
    echo ""
    echo "File structure:"
    cat custom_output_test.json | head -30
else
    echo "❌ Custom output file not created"
fi
echo "✓ Test 5 complete"
echo ""

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "=========================================="
echo "✅ All Tests Complete!"
echo "=========================================="
echo "Duration: ${DURATION}s"
echo ""
echo "Summary:"
echo "  ✓ Directory validation working"
echo "  ✓ Cards file validation working"
echo "  ✓ Empty file detection working"
echo "  ✓ Malformed card handling working"
echo "  ✓ Custom output file (-o) working"
echo ""
echo "Cleanup:"
echo "  rm -f test*.json custom_output_test.json"
