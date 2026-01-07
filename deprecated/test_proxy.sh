#!/bin/bash

echo "🧪 Testing Proxy Implementation"
echo "================================"
echo ""

# Check if binary exists
if [ ! -f "./target/release/shopify_checker" ]; then
    echo "❌ Binary not found. Building..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "❌ Build failed!"
        exit 1
    fi
fi

echo "✓ Binary found"
echo ""

# Check if proxies.txt exists
if [ ! -f "proxies.txt" ]; then
    echo "❌ proxies.txt not found!"
    exit 1
fi

echo "✓ proxies.txt found"
echo ""

# Count proxies
PROXY_COUNT=$(wc -l < proxies.txt)
echo "📊 Proxies available: $PROXY_COUNT"
echo ""

# Check if test gates exist
if [ ! -f "production_gates.json" ]; then
    echo "⚠️  production_gates.json not found, using test gates..."
    
    # Create minimal test gates
    cat > test_proxy_gates.json << 'EOF'
[
  {
    "url": "https://donate.example.com",
    "gateway": "Shopify",
    "donation_form": true
  }
]
EOF
    GATES_FILE="test_proxy_gates.json"
else
    GATES_FILE="production_gates.json"
fi

echo "✓ Using gates file: $GATES_FILE"
echo ""

# Check if test cards exist
if [ ! -f "test_auth_cards.txt" ]; then
    echo "⚠️  test_auth_cards.txt not found, creating test card..."
    echo "4532015112830366|12|2027|999" > test_proxy_cards.txt
    CARDS_FILE="test_proxy_cards.txt"
else
    CARDS_FILE="test_auth_cards.txt"
fi

echo "✓ Using cards file: $CARDS_FILE"
echo ""

# Check if ChromeDriver is running
if ! pgrep -x "chromedriver" > /dev/null; then
    echo "⚠️  ChromeDriver not running. Starting..."
    chromedriver --port=9515 &
    CHROMEDRIVER_PID=$!
    sleep 2
    echo "✓ ChromeDriver started (PID: $CHROMEDRIVER_PID)"
else
    echo "✓ ChromeDriver already running"
    CHROMEDRIVER_PID=""
fi

echo ""
echo "🚀 Running proxy test..."
echo "========================"
echo ""

# Run with proxies (test 1 gate only)
timeout 60 ./target/release/shopify_checker rotate \
    --gates "$GATES_FILE" \
    --cards-file "$CARDS_FILE" \
    --proxy-file proxies.txt \
    --max-gates 1 \
    --auth-only=true \
    --output test_proxy_results.json 2>&1 | head -100

EXIT_CODE=$?

echo ""
echo "========================"
echo "📊 Test Results"
echo "========================"
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Test completed successfully!"
elif [ $EXIT_CODE -eq 124 ]; then
    echo "⏱️  Test timed out (60 seconds)"
else
    echo "❌ Test failed with exit code: $EXIT_CODE"
fi

echo ""

# Check if results file was created
if [ -f "test_proxy_results.json" ]; then
    echo "✓ Results file created"
    echo ""
    echo "Results preview:"
    cat test_proxy_results.json | jq '.' 2>/dev/null || cat test_proxy_results.json
else
    echo "⚠️  No results file created"
fi

echo ""

# Cleanup
if [ -n "$CHROMEDRIVER_PID" ]; then
    echo "🧹 Stopping ChromeDriver..."
    kill $CHROMEDRIVER_PID 2>/dev/null
fi

# Clean up test files
if [ -f "test_proxy_gates.json" ]; then
    rm test_proxy_gates.json
fi

if [ -f "test_proxy_cards.txt" ]; then
    rm test_proxy_cards.txt
fi

echo ""
echo "✅ Test complete!"
