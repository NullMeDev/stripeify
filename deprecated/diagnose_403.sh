#!/bin/bash
# Diagnose 403 Errors - Check what's actually happening

echo "════════════════════════════════════════════════════════════"
echo "  403 Error Diagnostic Tool"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check 1: Proxy file exists and format
echo "1. Checking proxy file..."
if [ -f "proxies.txt" ]; then
    echo "✓ proxies.txt exists"
    echo "  First 3 proxies:"
    head -3 proxies.txt | while read line; do
        echo "    $line"
    done
    echo ""
    
    # Check format
    first_proxy=$(head -1 proxies.txt)
    colon_count=$(echo "$first_proxy" | tr -cd ':' | wc -c)
    echo "  Format check: $colon_count colons found"
    if [ "$colon_count" -eq 3 ]; then
        echo "  ✓ Format looks correct (host:port:user:pass)"
    else
        echo "  ⚠️  Format might be wrong. Expected: host:port:user:pass"
    fi
else
    echo "✗ proxies.txt not found!"
fi

echo ""

# Check 2: Test proxy directly
echo "2. Testing first proxy with curl..."
if [ -f "proxies.txt" ]; then
    first_proxy=$(head -1 proxies.txt)
    
    # Parse proxy
    IFS=':' read -r host port user pass <<< "$first_proxy"
    
    echo "  Proxy: $host:$port"
    echo "  Testing connection..."
    
    if timeout 10 curl -x "http://$user:$pass@$host:$port" -s https://api.ipify.org 2>/dev/null; then
        echo "  ✓ Proxy works with curl"
    else
        echo "  ✗ Proxy failed with curl"
        echo "  This proxy might be dead or blocked"
    fi
fi

echo ""

# Check 3: Test without proxy
echo "3. Testing gate access without proxy..."
if [ -f "full_test_gates.txt" ]; then
    test_gate=$(head -1 full_test_gates.txt)
    echo "  Testing: $test_gate"
    
    response=$(timeout 10 curl -s -o /dev/null -w "%{http_code}" "$test_gate" 2>/dev/null)
    echo "  Response code: $response"
    
    if [ "$response" = "200" ] || [ "$response" = "301" ] || [ "$response" = "302" ]; then
        echo "  ✓ Gate accessible without proxy"
    elif [ "$response" = "403" ]; then
        echo "  ⚠️  Getting 403 even without proxy!"
        echo "  This means the gate itself is blocking requests"
    else
        echo "  Response: $response"
    fi
fi

echo ""

# Check 4: ChromeDriver status
echo "4. Checking ChromeDriver..."
if pgrep -f chromedriver > /dev/null; then
    echo "  ✓ ChromeDriver is running"
else
    echo "  ✗ ChromeDriver not running"
    echo "  Start with: chromedriver --port=9515 &"
fi

echo ""

# Check 5: Recent test logs
echo "5. Checking recent test logs..."
if [ -f "log_batch_1.txt" ]; then
    echo "  Found test logs. Checking for 403 errors..."
    grep -i "403\|forbidden\|blocked" log_batch_1.txt | head -5
    echo ""
fi

echo "════════════════════════════════════════════════════════════"
echo "  Recommendations"
echo "════════════════════════════════════════════════════════════"
echo ""

# Analyze and recommend
if [ -f "proxies.txt" ]; then
    first_proxy=$(head -1 proxies.txt)
    IFS=':' read -r host port user pass <<< "$first_proxy"
    
    # Test if proxy works
    if timeout 5 curl -x "http://$user:$pass@$host:$port" -s https://api.ipify.org > /dev/null 2>&1; then
        echo "✓ Your proxies seem to work with curl"
        echo ""
        echo "The issue is likely:"
        echo "1. ChromeDriver doesn't support authenticated proxies well"
        echo "2. Need to use proxy extension or different approach"
        echo ""
        echo "Solutions:"
        echo "A) Use rotating proxy API (recommended)"
        echo "   - See ROTATING_PROXY_SETUP.md"
        echo "   - API Key: 950d5fdb-1960-4718-9cce-ccd10e8654c5"
        echo ""
        echo "B) Run without proxies temporarily to test"
        echo "   - Remove --proxy-file flag"
        echo "   - Use longer delays between requests"
    else
        echo "✗ Your proxies don't work with curl"
        echo ""
        echo "This means:"
        echo "1. Proxies are dead/blocked"
        echo "2. Wrong format or credentials"
        echo ""
        echo "Solutions:"
        echo "A) Get fresh proxies"
        echo "B) Use rotating proxy API"
        echo "   - See ROTATING_PROXY_SETUP.md"
        echo "   - API Key: 950d5fdb-1960-4718-9cce-ccd10e8654c5"
    fi
else
    echo "No proxy file found"
    echo ""
    echo "Recommendation:"
    echo "Set up rotating proxy API"
    echo "- See ROTATING_PROXY_SETUP.md"
    echo "- API Key: 950d5fdb-1960-4718-9cce-ccd10e8654c5"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
