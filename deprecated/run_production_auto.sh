#!/bin/bash

# Automatic Production Gate Finder
# Smart card rotation: finds working card, then uses it until dead
# NO PROMPTS - Set and forget

set -e

echo "═══════════════════════════════════════════════════════════"
echo "🚀 AUTOMATIC PRODUCTION GATE FINDER"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Find cards file
CARDS_FILE=""
if [ -f "cards.txt" ]; then
    CARDS_FILE="cards.txt"
elif [ -f "../cards.txt" ]; then
    CARDS_FILE="../cards.txt"
elif [ -f "test_auth_cards.txt" ]; then
    CARDS_FILE="test_auth_cards.txt"
else
    echo "❌ ERROR: No cards file found!"
    echo "Please create cards.txt with your cards (format: number|month|year|cvv)"
    exit 1
fi

CARD_COUNT=$(wc -l < "$CARDS_FILE")
echo "✓ Cards: $CARD_COUNT"

# Check proxies
if [ ! -f "proxies.txt" ]; then
    echo "❌ ERROR: proxies.txt not found!"
    exit 1
fi

PROXY_COUNT=$(wc -l < "proxies.txt")
echo "✓ Proxies: $PROXY_COUNT"

# Count chunks
CHUNK_COUNT=$(ls -1 ShopifyGatesAndChunks/*.txt 2>/dev/null | wc -l)
echo "✓ Chunks: $CHUNK_COUNT"
echo ""

# Start ChromeDriver if not running
if ! pgrep -f chromedriver > /dev/null; then
    echo "→ Starting ChromeDriver..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
    echo "✓ ChromeDriver started"
else
    echo "✓ ChromeDriver running"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "🔥 STARTING AUTOMATIC RUN - NO PROMPTS"
echo "═══════════════════════════════════════════════════════════"
echo "Strategy: Find working card → Use until dead → Next card"
echo "Mode: Charged (\$1 per valid gate)"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Create results directory
mkdir -p production_results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="production_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Merge all chunks into one file
echo "→ Merging all chunks into master gate list..."
cat ShopifyGatesAndChunks/*.txt > "$RESULTS_DIR/all_gates.txt"
TOTAL_GATES=$(wc -l < "$RESULTS_DIR/all_gates.txt")
echo "✓ Total gates to process: $TOTAL_GATES"
echo ""

# Run the SMART checker with ALL gates and ALL cards
# Smart strategy: Try cards on each gate until one works, then use that card for all remaining gates
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Processing all $TOTAL_GATES gates with SMART card rotation..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "y" | ./target/release/shopify_checker smart \
    --gates "$RESULTS_DIR/all_gates.txt" \
    --cards-file "$CARDS_FILE" \
    --proxy-file proxies.txt \
    --auth-only=false \
    --output "$RESULTS_DIR/valid_gates.json" \
    2>&1 | tee "$RESULTS_DIR/run.log"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ RUN COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Count results
if [ -f "$RESULTS_DIR/valid_gates.json" ]; then
    VALID_COUNT=$(jq '[.[] | select(.status == "CHARGED")] | length' "$RESULTS_DIR/valid_gates.json" 2>/dev/null || echo "0")
    UNIQUE_GATES=$(jq '[.[] | .gate] | unique | length' "$RESULTS_DIR/valid_gates.json" 2>/dev/null || echo "0")
    
    echo "📊 FINAL STATISTICS:"
    echo "  • Total gates processed: $TOTAL_GATES"
    echo "  • Valid gates found: $UNIQUE_GATES"
    echo "  • Total charges: $VALID_COUNT"
    echo "  • Total cost: \$$VALID_COUNT.00"
    echo "  • Success rate: $(awk "BEGIN {printf \"%.2f\", ($UNIQUE_GATES/$TOTAL_GATES)*100}")%"
    echo ""
    echo "📁 Results: $RESULTS_DIR/valid_gates.json"
    echo "📄 Log: $RESULTS_DIR/run.log"
else
    echo "⚠️  No results file generated"
    echo "Check log: $RESULTS_DIR/run.log"
fi

echo ""
echo "🎉 Production run complete!"
