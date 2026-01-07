#!/bin/bash

echo "🧪 Quick Test - Testing with 5 cards only"
echo ""

# Check if ChromeDriver is running
if ! pgrep -f "chromedriver.*9515" > /dev/null; then
    echo "⚠️  ChromeDriver not running. Starting it..."
    chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 &
    sleep 3
    echo "✓ ChromeDriver started"
else
    echo "✓ ChromeDriver already running"
fi

echo ""
echo "Creating test file with 5 cards..."

# Create a small test file with first 5 cards
head -5 42000Dump.txt > test_5_cards.txt

echo "✓ Created test_5_cards.txt"
echo ""
echo "Running checker with 5 cards..."
echo ""

# Run the checker
./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file test_5_cards.txt \
  --telegram-config telegram_config.json

echo ""
echo "✅ Test complete!"
echo ""
echo "Check results:"
echo "  - live_results.json"
echo "  - live_results_working_gates.txt"
