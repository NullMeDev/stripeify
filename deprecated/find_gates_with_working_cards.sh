#!/bin/bash
# Find Valid Gates Using Working Cards ($1 charges)

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Gate Discovery with Working Cards"
echo "  Using 6 proven cards that charge $0.98"
echo "════════════════════════════════════════════════════════════"
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
echo "Configuration:"
echo "  Cards: 6 working cards (charge $0.98)"
echo "  Gates: ALL 118 gates from full_test_gates.txt"
echo "  Mode: CHARGE MODE (real charges)"
echo "  Strategy: Smart card rotation"
echo "  Cost: ~$1 per valid gate found"
echo ""

echo "Working Cards:"
echo "  1. 5137704502263801|12|25|443 ✅"
echo "  2. 4978742321301530|12|25|932 ✅"
echo "  3. 4970407612792304|12|25|714 ✅"
echo "  4. 4972039762522823|12|25|085 ✅"
echo "  5. 4978740374147008|12|25|015 ✅"
echo "  6. 5131624509434153|12|25|662 ✅"
echo ""

echo "How it works:"
echo "  1. Try first card on first gate"
echo "  2. If it works, use that card for ALL gates"
echo "  3. When card dies, try next card"
echo "  4. Repeat until all gates tested"
echo ""

echo "Expected:"
echo "  - Find 30-50 valid gates"
echo "  - Cost: $30-50 total"
echo "  - Time: 2-3 hours"
echo ""

read -p "Press Enter to start gate discovery (or Ctrl+C to cancel)..."

echo ""
echo "Starting gate discovery..."
echo ""

# Run smart mode with working cards
echo "y" | timeout 10800 ./target/release/shopify_checker smart \
  --gates full_test_gates.txt \
  --cards-file working_cards.txt \
  --auth-only=false 2>&1 | tee gate_discovery_results.log

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Gate Discovery Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check results
if [ -f "smart_results.json" ]; then
    echo "✓ Results saved to: smart_results.json"
    echo "✓ Log saved to: gate_discovery_results.log"
    echo ""
    
    # Count valid gates
    valid_count=$(grep -c '"success":true' smart_results.json 2>/dev/null || echo "0")
    echo "═══════════════════════════════════════════════════════════"
    echo "  📊 RESULTS SUMMARY"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "✅ Found $valid_count valid gate(s)"
    echo ""
    
    if [ "$valid_count" -gt 0 ]; then
        echo "Valid Gates:"
        grep '"gate"' smart_results.json | cut -d'"' -f4 | sort -u | nl
        echo ""
        
        # Extract to separate file
        grep '"gate"' smart_results.json | cut -d'"' -f4 | sort -u > valid_gates_found.txt
        echo "✓ Valid gates saved to: valid_gates_found.txt"
        echo ""
        
        # Calculate cost
        cost=$(echo "$valid_count * 1" | bc)
        echo "💰 Estimated cost: \$$cost (at $1 per gate)"
        echo ""
        
        echo "Next steps:"
        echo "1. Review smart_results.json for full details"
        echo "2. Use valid_gates_found.txt for production"
        echo "3. Test more cards on these gates"
    else
        echo "⚠️  No valid gates found"
        echo ""
        echo "Possible reasons:"
        echo "- All gates are dead/blocked"
        echo "- Cards got declined"
        echo "- Network issues"
        echo ""
        echo "Check gate_discovery_results.log for details"
    fi
else
    echo "⚠️  No results file found"
    echo "Check gate_discovery_results.log for errors"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
