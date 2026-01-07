#!/bin/bash
# Setup Porter Proxies Rotating API

echo "════════════════════════════════════════════════════════════"
echo "  Porter Proxies Setup"
echo "════════════════════════════════════════════════════════════"
echo ""

# Your credentials from proxies.txt
HOST="evo-pro.porterproxies.com"
PORT="62345"
USER="PP_5J7SVIL0BJ-country-US-state-Florida"
PASS="95cc2n4b"

# Also your API key
API_KEY="950d5fdb-1960-4718-9cce-ccd10e8654c5"

echo "Detected Porter Proxies configuration:"
echo "  Host: $HOST"
echo "  Port: $PORT"
echo "  User: $USER"
echo ""

# Create rotating proxy file
echo "http://$USER:$PASS@$HOST:$PORT" > rotating_proxy.txt

echo "✓ Created rotating_proxy.txt"
echo ""

# Test it
echo "Testing proxy..."
if curl -x "http://$USER:$PASS@$HOST:$PORT" -s https://api.ipify.org; then
    echo ""
    echo "✓ Proxy works!"
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo "  Next Steps"
    echo "════════════════════════════════════════════════════════════"
    echo ""
    echo "The issue is ChromeDriver doesn't support authenticated proxies."
    echo ""
    echo "SOLUTION: Test WITHOUT proxies but with longer delays"
    echo ""
    echo "Run this command:"
    echo "  head -5 full_test_gates.txt > test_5_gates.txt"
    echo "  "
    echo "  echo 'y' | ./target/release/shopify_checker rotate \\"
    echo "    --gates test_5_gates.txt \\"
    echo "    --cards-file full_test_cards.txt \\"
    echo "    --auth-only=true \\"
    echo "    --max-gates 5"
    echo ""
    echo "This will:"
    echo "- Test 5 gates only"
    echo "- Use auth-only mode (FREE)"
    echo "- NO proxies (to avoid 403)"
    echo "- Find valid gates"
    echo ""
else
    echo ""
    echo "✗ Proxy test failed"
fi
