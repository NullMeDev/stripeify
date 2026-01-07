# Quick Run Commands - Telegram Integration

## ⚠️ Important: Always Run from Project Directory

The binary is located at:
```
/home/null/Desktop/Stripeify/target/release/shopify_checker
```

You must be in the project directory to run it with `./target/release/shopify_checker`

---

## 🚀 Correct Commands

### 1. Navigate to Project Directory First:
```bash
cd /home/null/Desktop/Stripeify
```

### 2. Start ChromeDriver:
```bash
# Kill any existing instances
pkill chromedriver

# Start fresh
chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 &
```

### 3. Run Test with 2 Cards:
```bash
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file test_cards.txt \
  --telegram-config telegram_config.json
```

### 4. Run Full Production (42,710 cards):
```bash
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json
```

---

## 🔧 Alternative: Use Full Path

If you want to run from anywhere:

```bash
# From any directory
/home/null/Desktop/Stripeify/target/release/shopify_checker test \
  --gates /home/null/Desktop/Stripeify/production_gates.txt \
  --cards-file /home/null/Desktop/Stripeify/42000Dump.txt \
  --telegram-config /home/null/Desktop/Stripeify/telegram_config.json
```

---

## 📋 Complete Workflow

### One-Time Setup:
```bash
cd /home/null/Desktop/Stripeify
chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 &
```

### Test Run (2 cards):
```bash
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file test_cards.txt \
  --telegram-config telegram_config.json
```

### Production Run (42,710 cards):
```bash
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json
```

---

## ✅ Verify Setup

Before running, verify everything is ready:

```bash
cd /home/null/Desktop/Stripeify

# Check binary exists
ls -lh target/release/shopify_checker

# Check ChromeDriver is running
curl -s http://localhost:9515/status | jq -r '.value.ready'

# Check files exist
ls -lh telegram_config.json production_gates.txt 42000Dump.txt

# Check card count
wc -l 42000Dump.txt
```

Expected output:
```
-rwxrwxr-x ... 11M ... target/release/shopify_checker
true
... telegram_config.json
... production_gates.txt
... 42000Dump.txt
42710 42000Dump.txt
```

---

## 🎯 Quick Copy-Paste Commands

### Test with 2 cards:
```bash
cd /home/null/Desktop/Stripeify && \
pkill chromedriver; chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 & sleep 2 && \
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file test_cards.txt \
  --telegram-config telegram_config.json
```

### Production with all cards:
```bash
cd /home/null/Desktop/Stripeify && \
pkill chromedriver; chromedriver --port=9515 > /tmp/chromedriver.log 2>&1 & sleep 2 && \
./target/release/shopify_checker test \
  --gates production_gates.txt \
  --cards-file 42000Dump.txt \
  --telegram-config telegram_config.json
```

---

## 📊 Monitor Progress

### Watch terminal output:
The program will show real-time progress

### Check Telegram:
Successful charges appear immediately in your group

### View results:
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json | jq
```

---

## 🛑 Stop Running Test

If you need to stop:
```bash
# Press Ctrl+C in the terminal

# Then kill ChromeDriver
pkill chromedriver
```

---

## ❌ Common Mistakes

### ❌ Wrong: Running from home directory
```bash
cd ~
./target/release/shopify_checker test ...
# Error: No such file or directory
```

### ✅ Correct: Running from project directory
```bash
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker test ...
# Works!
```

---

## 📝 Summary

**Always remember:**
1. `cd /home/null/Desktop/Stripeify` FIRST
2. Start ChromeDriver
3. Run the checker
4. Check Telegram for notifications

**The binary location is:**
`/home/null/Desktop/Stripeify/target/release/shopify_checker`
