#!/bin/bash

# Test Edge Cases for Discovery Mode
# Tests validation, error handling, and -o flag

echo "=========================================="
echo "Edge Case Testing for Discovery Mode"
echo "=========================================="
echo ""

BINARY="./target/release/shopify_checker"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found. Run: cargo build --release"
    exit 1
fi

echo "✓ Binary found"
echo ""

# Test 1: Invalid gates directory
echo "Test 1: Invalid gates directory"
echo "-----------------------------------"
$BINARY discover \
    --gates-dir /nonexistent/directory \
    --cards-file test_discover_cards.txt \
    --max-gates 1 \
    -o test_invalid_dir.json 2>&1 | head -5
echo ""

# Test 2: Invalid cards file
echo "Test 2: Invalid cards file"
echo "-----------------------------------"
$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file /nonexistent/cards.txt \
    --max-gates 1 \
    -o test_invalid_cards.json 2>&1 | head -5
echo ""

# Test 3: Empty cards file
echo "Test 3: Empty cards file"
echo "-----------------------------------"
echo "" > test_empty_cards.txt
$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_empty_cards.txt \
    --max-gates 1 \
    -o test_empty_cards_output.json 2>&1 | head -5
rm -f test_empty_cards.txt
echo ""

# Test 4: Malformed cards file
echo "Test 4: Malformed cards file"
echo "-----------------------------------"
cat > test_malformed_cards.txt << 'EOF'
# This is a comment
invalid_card_format
4532015112830366|12|2027  # Missing CVV
4532015112830366|12|2027|123  # Valid
another|bad|format|here
EOF

$BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_malformed_cards.txt \
    --max-gates 1 \
    -o test_malformed_output.json 2>&1 | head -10
rm -f test_malformed_cards.txt
echo ""

# Test 5: Custom output file with -o flag
echo "Test 5: Custom output file (-o flag)"
echo "-----------------------------------"
echo "Testing with custom output: my_custom_results.json"
echo "y" | timeout 45 $BINARY discover \
    --gates-dir /home/null/Desktop/ShopifyGatesAndChunks \
    --cards-file test_discover_cards.txt \
    --max-gates 1 \
    -o my_custom_results.json 2>&1 | grep -E "(output|saved|Complete)" | head -5

if [ -f "my_custom_results.json" ]; then
    echo "✓ Custom output file created successfully"
    ls -lh my_custom_results.json
    echo ""
    echo "File contents preview:"
    cat my_custom_results.json | head -20
else
    echo "❌ Custom output file not created"
fi
echo ""

# Test 6: Directory with no gate files
echo "Test 6: Directory with no gate files"
echo "-----------------------------------"
mkdir -p test_empty_gates_dir
$BINARY discover \
    --gates-dir test_empty_gates_dir \
    --cards-file test_discover_cards.txt \
    --max-gates 1 \
    -o test_no_gates.json 2>&1 | head -5
rmdir test_empty_gates_dir
echo ""

echo "=========================================="
echo "Edge Case Testing Complete"
echo "=========================================="
echo ""
echo "Summary of created files:"
ls -lh test_*.json my_custom_results.json 2>/dev/null | grep -v "cannot access"
echo ""
echo "Cleanup command:"
echo "rm -f test_*.json my_custom_results.json"
