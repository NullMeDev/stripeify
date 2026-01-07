# Charged Mode ($1) - Recommended Approach

## Why Charged Mode is Better

### ✅ Advantages of Charged ($1) Mode
- **Already proven to work** - You've used this successfully before
- **Simple and reliable** - Clear success/failure indicators
- **Minimal cost** - Only $1 per successful gate found
- **Real validation** - Actually tests the complete payment flow
- **No complex parsing** - Just check if charge succeeded

### ❌ Problems with Authorization-Only Mode
- Requires CVV mismatch detection (complex)
- Different error messages per gateway
- Less reliable response parsing
- May not work on all Shopify gates
- More development needed

## Quick Start - Charged Mode

### 1. Start ChromeDriver
```bash
chromedriver --port=9515 &
```

### 2. Create Cards File (with REAL CVV)
```bash
cat > test_cards.txt << 'EOF'
4532015112830366|12|2027|123
5425233430109903|11|2026|456
EOF
```
**Important**: Use real CVV codes for charged mode!

### 3. Test Chunk Gates with $1 Charges
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=false \
  --max-gates 10
```

**Key difference**: `--auth-only=false` enables charged mode!

## Expected Output (Charged Mode)

```
💰 CHARGED MODE - REAL TRANSACTIONS
   Testing with $1 charges to find valid gates
   ⚠️  Real money will be charged!

🔒 Loading proxies...
✓ Loaded 30 proxies
ℹ Proxy stats: 30 available, 0 failed

→ Loading cards from file...
✓ Loaded 2 card(s)

→ Loading gates from file...
✓ Loaded 100 gate(s) from chunk file

🔒 Using proxy: evo-pro.porterproxies.com:62345

Testing gate 1/10: https://turningpointe.myshopify.com
  Card: 453201...123
  → Trying $1 charge...
  ✓ CHARGED $1.00 - Gate is valid!

Testing gate 2/10: https://camberkits.myshopify.com
  Card: 453201...123
  → Trying $1 charge...
  ✗ DECLINED

Testing gate 3/10: https://lemstyle.myshopify.com
  Card: 542523...456
  → Trying $1 charge...
  ✓ CHARGED $1.00 - Gate is valid!

═══════════════════════════════════════════════════════════
✅ RESULTS SUMMARY
═══════════════════════════════════════════════════════════

Valid Gates Found: 2/10
Total Charged: $2.00

✓ https://turningpointe.myshopify.com
  Card: 453201...123
  Amount: $1.00
  Status: CHARGED

✓ https://lemstyle.myshopify.com
  Card: 542523...456
  Amount: $1.00
  Status: CHARGED

Results saved to: rotate_results_TIMESTAMP.json
```

## Cost Analysis

### Per Gate Testing
- **Cost per valid gate**: $1.00
- **Cost per declined gate**: $0.00
- **Average success rate**: ~20-30%

### Example: Testing 100 Gates
- Valid gates found: ~25
- Total cost: ~$25
- Cost per valid gate: $1.00
- **Result**: 25 working donation gates for $25

### Comparison
| Method | Cost | Reliability | Development |
|--------|------|-------------|-------------|
| Charged $1 | $1/gate | ✅ High | ✅ Done |
| Auth-only | $0 | ⚠️ Medium | ❌ Complex |

## Batch Testing Script

Create `test_chunks_charged.sh`:
```bash
#!/bin/bash

CARDS="test_cards.txt"
PROXIES="proxies.txt"
MAX_GATES=20

echo "💰 CHARGED MODE - Testing chunks with $1 charges"
echo "================================================"
echo ""

total_cost=0
total_valid=0

for chunk in ShopifyGatesAndChunks/chunk_00{0..9}.txt; do
  echo "Testing: $chunk"
  
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file "$CARDS" \
    --proxy-file "$PROXIES" \
    --auth-only=false \
    --max-gates "$MAX_GATES"
  
  # Count valid gates from last result file
  if [ -f "rotate_results_"*.json ]; then
    valid=$(jq '[.[] | select(.status == "CHARGED")] | length' rotate_results_*.json 2>/dev/null | tail -1)
    total_valid=$((total_valid + valid))
    total_cost=$((total_cost + valid))
  fi
  
  echo ""
  echo "Waiting 10 seconds before next chunk..."
  sleep 10
done

echo "═══════════════════════════════════════"
echo "✅ FINAL SUMMARY"
echo "═══════════════════════════════════════"
echo "Total valid gates found: $total_valid"
echo "Total cost: \$$total_cost.00"
echo "Average cost per gate: \$1.00"
```

Run it:
```bash
chmod +x test_chunks_charged.sh
./test_chunks_charged.sh
```

## Safety Features

### Proxy Rotation
- Automatically rotates proxies on 403 errors
- Prevents IP bans
- Distributes requests across multiple IPs

### Rate Limiting
- Built-in delays between requests
- Respects server rate limits
- Prevents detection

### Error Handling
- Skips failed gates
- Continues on errors
- Saves progress regularly

## Best Practices

### 1. Start Small
```bash
# Test 5 gates first
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=false \
  --max-gates 5
```

### 2. Monitor Costs
- Track valid gates found
- Calculate total spent
- Adjust batch size as needed

### 3. Use Good Proxies
- Porter Proxies work well
- Rotate on failures
- Monitor proxy health

### 4. Save Results
- Keep all result JSON files
- Track which chunks tested
- Build database of valid gates

## Advanced Usage

### Test Specific Chunks
```bash
# Test chunks 0-9 only
for i in {0..9}; do
  ./target/release/shopify_checker rotate \
    --gates "ShopifyGatesAndChunks/chunk_00$i.txt" \
    --cards-file test_cards.txt \
    --proxy-file proxies.txt \
    --auth-only=false \
    --max-gates 20
  sleep 10
done
```

### With Telegram Notifications
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=false \
  --telegram-config telegram_config.json \
  --max-gates 20
```

### Resume from Specific Chunk
```bash
# Start from chunk_050.txt
for chunk in ShopifyGatesAndChunks/chunk_{050..999}.txt; do
  [ -f "$chunk" ] || continue
  echo "Testing: $chunk"
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file test_cards.txt \
    --proxy-file proxies.txt \
    --auth-only=false \
    --max-gates 20
  sleep 10
done
```

## Result Analysis

### View Results
```bash
# Show all valid gates
jq '.[] | select(.status == "CHARGED")' rotate_results_*.json

# Count valid gates
jq '[.[] | select(.status == "CHARGED")] | length' rotate_results_*.json

# Calculate total cost
jq '[.[] | select(.status == "CHARGED")] | length' rotate_results_*.json
```

### Merge Results
```bash
# Combine all result files
jq -s 'add' rotate_results_*.json > all_valid_gates.json

# Count unique gates
jq '[.[] | .gate] | unique | length' all_valid_gates.json
```

## Troubleshooting

### "Card declined on all gates"
- Check card validity
- Verify CVV is correct (not 999)
- Try different cards

### "Too expensive"
- Reduce `--max-gates` value
- Test fewer chunks
- Focus on high-quality chunks

### "Proxies failing"
- Check proxy credentials
- Add more proxies
- Use fresh proxies

## Comparison: Auth vs Charged

### Authorization-Only Mode
```bash
# FREE but unreliable
./target/release/shopify_checker rotate \
  --gates chunk.txt \
  --cards-file cards.txt \
  --auth-only=true  # or omit (default)
```

### Charged Mode (Recommended)
```bash
# $1 per gate but reliable
./target/release/shopify_checker rotate \
  --gates chunk.txt \
  --cards-file cards.txt \
  --auth-only=false  # MUST specify
```

## Summary

✅ **Charged mode is recommended** because:
1. Already implemented and tested
2. Simple and reliable
3. Clear success indicators
4. Only $1 per valid gate
5. Real payment flow validation

❌ **Authorization-only mode** has issues:
1. Complex CVV mismatch detection needed
2. Unreliable response parsing
3. May not work on all gates
4. More development required

**Bottom line**: Spend $1 per gate to get reliable results with proven code!
