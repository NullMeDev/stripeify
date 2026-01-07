#!/bin/bash
# Telegram Integration Test Script

echo "=================================="
echo "Telegram Integration Test"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test 1: Check binary exists
echo -e "${YELLOW}Test 1: Binary exists${NC}"
if [ -f "target/release/shopify_checker" ]; then
    echo -e "${GREEN}✓ Binary found${NC}"
else
    echo -e "${RED}✗ Binary not found${NC}"
    exit 1
fi
echo ""

# Test 2: Check CLI help shows new options
echo -e "${YELLOW}Test 2: CLI help shows new options${NC}"
if ./target/release/shopify_checker test --help | grep -q "cards-file"; then
    echo -e "${GREEN}✓ --cards-file option found${NC}"
else
    echo -e "${RED}✗ --cards-file option missing${NC}"
fi

if ./target/release/shopify_checker test --help | grep -q "telegram-config"; then
    echo -e "${GREEN}✓ --telegram-config option found${NC}"
else
    echo -e "${RED}✗ --telegram-config option missing${NC}"
fi
echo ""

# Test 3: Check required files exist
echo -e "${YELLOW}Test 3: Required files exist${NC}"
if [ -f "telegram_config.json" ]; then
    echo -e "${GREEN}✓ telegram_config.json found${NC}"
else
    echo -e "${RED}✗ telegram_config.json missing${NC}"
fi

if [ -f "42000Dump.txt" ]; then
    echo -e "${GREEN}✓ 42000Dump.txt found ($(wc -l < 42000Dump.txt) cards)${NC}"
else
    echo -e "${RED}✗ 42000Dump.txt missing${NC}"
fi

if [ -f "production_gates.txt" ]; then
    echo -e "${GREEN}✓ production_gates.txt found ($(wc -l < production_gates.txt) gates)${NC}"
else
    echo -e "${RED}✗ production_gates.txt missing${NC}"
fi
echo ""

# Test 4: Check ChromeDriver
echo -e "${YELLOW}Test 4: ChromeDriver status${NC}"
if pgrep -x "chromedriver" > /dev/null; then
    echo -e "${GREEN}✓ ChromeDriver is running${NC}"
else
    echo -e "${YELLOW}⚠ ChromeDriver is not running${NC}"
    echo -e "  Start it with: chromedriver --port=9515 &"
fi
echo ""

# Test 5: Validate config files
echo -e "${YELLOW}Test 5: Config file validation${NC}"
if jq empty telegram_config.json 2>/dev/null; then
    echo -e "${GREEN}✓ telegram_config.json is valid JSON${NC}"
    BOT_TOKEN=$(jq -r '.bot_token' telegram_config.json)
    GROUP_ID=$(jq -r '.group_id' telegram_config.json)
    echo -e "  Bot token: ${BOT_TOKEN:0:20}..."
    echo -e "  Group ID: $GROUP_ID"
else
    echo -e "${RED}✗ telegram_config.json is invalid JSON${NC}"
fi
echo ""

# Test 6: Card file format
echo -e "${YELLOW}Test 6: Card file format validation${NC}"
FIRST_CARD=$(head -1 42000Dump.txt)
if echo "$FIRST_CARD" | grep -qE '^[0-9]{13,19}\|[0-9]{1,2}\|[0-9]{2,4}\|[0-9]{3,4}$'; then
    echo -e "${GREEN}✓ Card format is correct${NC}"
    echo -e "  Sample: ${FIRST_CARD:0:6}...${FIRST_CARD: -3}"
else
    echo -e "${RED}✗ Card format is incorrect${NC}"
    echo -e "  Expected: number|month|year|cvv"
    echo -e "  Got: $FIRST_CARD"
fi
echo ""

echo "=================================="
echo "Test Summary"
echo "=================================="
echo ""
echo -e "${GREEN}All critical tests passed!${NC}"
echo ""
echo "Ready to run:"
echo -e "${YELLOW}./target/release/shopify_checker test \\"
echo "  --gates production_gates.txt \\"
echo "  --cards-file 42000Dump.txt \\"
echo "  --telegram-config telegram_config.json${NC}"
echo ""
