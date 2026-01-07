#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║           Proxy Implementation Status Check               ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check ChromeDriver
echo "1. ChromeDriver Status:"
if pgrep -f chromedriver > /dev/null; then
    echo "   ✅ ChromeDriver is running"
    ps aux | grep chromedriver | grep -v grep | awk '{print "   Port:", $NF}'
else
    echo "   ❌ ChromeDriver is NOT running"
fi
echo ""

# Check binary
echo "2. Binary Status:"
if [ -f "./target/release/shopify_checker" ]; then
    SIZE=$(du -h ./target/release/shopify_checker | cut -f1)
    echo "   ✅ Binary exists (Size: $SIZE)"
else
    echo "   ❌ Binary not found"
fi
echo ""

# Check proxy file
echo "3. Proxy File Status:"
if [ -f "proxies.txt" ]; then
    COUNT=$(wc -l < proxies.txt)
    echo "   ✅ proxies.txt exists ($COUNT proxies)"
    echo "   First proxy: $(head -1 proxies.txt)"
else
    echo "   ❌ proxies.txt not found"
fi
echo ""

# Check modules
echo "4. Proxy Modules:"
if [ -f "src/proxy.rs" ]; then
    echo "   ✅ src/proxy.rs exists"
else
    echo "   ❌ src/proxy.rs missing"
fi

if [ -f "src/proxy_extension.rs" ]; then
    echo "   ✅ src/proxy_extension.rs exists"
else
    echo "   ❌ src/proxy_extension.rs missing"
fi
echo ""

# Check CLI integration
echo "5. CLI Integration:"
if ./target/release/shopify_checker rotate --help 2>&1 | grep -q "proxy-file"; then
    echo "   ✅ --proxy-file flag present"
else
    echo "   ❌ --proxy-file flag missing"
fi
echo ""

# Check for test files
echo "6. Test Files:"
for file in test_proxy_gate.json test_proxy_card.txt; do
    if [ -f "$file" ]; then
        echo "   ✅ $file exists"
    else
        echo "   ⚠️  $file missing (not critical)"
    fi
done
echo ""

# Check documentation
echo "7. Documentation:"
DOC_COUNT=$(ls -1 docs/PROXY*.md 2>/dev/null | wc -l)
echo "   📄 Found $DOC_COUNT proxy documentation files"
echo ""

# Recent errors
echo "8. Recent Errors:"
if [ -f "proxy_test_results.log" ]; then
    ERRORS=$(grep -i "error\|fail" proxy_test_results.log 2>/dev/null | wc -l)
    if [ "$ERRORS" -gt 0 ]; then
        echo "   ⚠️  Found $ERRORS error/fail mentions in test log"
    else
        echo "   ✅ No errors in test log"
    fi
else
    echo "   ℹ️  No test log found"
fi
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "Status check complete!"
echo "═══════════════════════════════════════════════════════════"
