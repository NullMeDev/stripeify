#!/bin/bash
# Simple Card Checker Runner
# Uses config.json for all settings

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Shopify Card Checker with Telegram Notifications"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if config exists
if [ ! -f "config.json" ]; then
    echo "❌ Error: config.json not found"
    echo ""
    echo "Create config.json with:"
    echo "{"
    echo "  \"telegram\": {"
    echo "    \"bot_token\": \"YOUR_BOT_TOKEN\","
    echo "    \"group_id\": \"YOUR_GROUP_ID\","
    echo "    \"bot_credit\": \"@YourBotName\""
    echo "  },"
    echo "  \"cards_file\": \"full_test_cards.txt\","
    echo "  \"gates_file\": \"donation_gates.json\","
    echo "  \"auth_only\": true,"
    echo "  \"max_gates\": null,"
    echo "  \"mode\": \"smart\""
    echo "}"
    exit 1
fi

# Read config
CARDS_FILE=$(jq -r '.cards_file' config.json)
GATES_FILE=$(jq -r '.gates_file // "null"' config.json)
GATES_DIR=$(jq -r '.gates_directory // "null"' config.json)
AUTH_ONLY=$(jq -r '.auth_only' config.json)
MAX_GATES=$(jq -r '.max_gates' config.json)
MODE=$(jq -r '.mode' config.json)
PROXIES_FILE=$(jq -r '.proxies_file // "null"' config.json)

echo "Configuration:"
echo "  Cards: $CARDS_FILE"
if [ "$MODE" = "discovery" ]; then
    echo "  Gates: $GATES_DIR"
else
    echo "  Gates: $GATES_FILE"
fi
echo "  Mode: $MODE"
echo "  Auth Only: $AUTH_ONLY"
if [ "$MAX_GATES" != "null" ]; then
    echo "  Max Gates: $MAX_GATES"
else
    echo "  Max Gates: ALL"
fi
echo ""

# Check if files exist
if [ ! -f "$CARDS_FILE" ]; then
    echo "❌ Error: Cards file not found: $CARDS_FILE"
    exit 1
fi

# Check gates based on mode
if [ "$MODE" = "discovery" ]; then
    if [ "$GATES_DIR" = "null" ] || [ ! -d "$GATES_DIR" ]; then
        echo "❌ Error: Gates directory not found: $GATES_DIR"
        exit 1
    fi
    GATE_COUNT=$(find "$GATES_DIR" -name "*.txt" | wc -l)
    echo "Files:"
    echo "  ✓ $(wc -l < "$CARDS_FILE") cards loaded"
    echo "  ✓ $GATE_COUNT gate files found in directory"
else
    if [ "$GATES_FILE" = "null" ] || [ ! -f "$GATES_FILE" ]; then
        echo "❌ Error: Gates file not found: $GATES_FILE"
        exit 1
    fi
    GATE_COUNT=$(jq 'length' "$GATES_FILE" 2>/dev/null || echo "0")
    echo "Files:"
    echo "  ✓ $(wc -l < "$CARDS_FILE") cards loaded"
    echo "  ✓ $GATE_COUNT gates loaded"
fi
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
echo "Starting card checker..."
echo "Press Ctrl+C to stop"
echo ""

# Build command based on mode
if [ "$MODE" = "discovery" ]; then
    CMD="./target/release/shopify_checker discover"
    CMD="$CMD --gates-dir $GATES_DIR"
    CMD="$CMD --cards-file $CARDS_FILE"
    CMD="$CMD --telegram-config config.json"
    CMD="$CMD --auth-only=$AUTH_ONLY"
    
    if [ "$PROXIES_FILE" != "null" ]; then
        CMD="$CMD --proxy-file $PROXIES_FILE"
    fi
    
    if [ "$MAX_GATES" != "null" ]; then
        CMD="$CMD --max-gates $MAX_GATES"
    fi
else
    CMD="./target/release/shopify_checker $MODE"
    CMD="$CMD --gates $GATES_FILE"
    CMD="$CMD --cards-file $CARDS_FILE"
    CMD="$CMD --telegram-config config.json"
    CMD="$CMD --auth-only=$AUTH_ONLY"
    
    if [ "$PROXIES_FILE" != "null" ]; then
        CMD="$CMD --proxy-file $PROXIES_FILE"
    fi
    
    if [ "$MAX_GATES" != "null" ]; then
        CMD="$CMD --max-gates $MAX_GATES"
    fi
fi

# Run checker
echo "y" | $CMD

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Checker Complete!"
echo "════════════════════════════════════════════════════════════"
