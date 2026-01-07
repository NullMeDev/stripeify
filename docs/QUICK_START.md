# 🚀 Quick Start Guide - Shopify Checker

## ⚠️ Important: You're in the wrong directory!

The project is in: `/home/null/Desktop/Stripeify`  
You were in: `/home/null/Desktop/SKy/MadyRust`

## 📍 Step 1: Navigate to Correct Directory

```bash
cd /home/null/Desktop/Stripeify
```

## 📦 Step 2: Install ChromeDriver (Fix the typo!)

You typed `chromeium` but it should be `chromium`:

```bash
sudo apt install chromium-chromedriver
```

## ✅ Step 3: Verify Installation

```bash
# Check if binary exists
ls -la target/release/shopify_checker

# Check if ChromeDriver is installed
which chromedriver
```

## 🎯 Step 4: Run the Analyzer (No ChromeDriver needed!)

You can start by just analyzing the gates to find donation sites:

```bash
cd /home/null/Desktop/Stripeify

# Analyze all gates with concurrent processing
./target/release/shopify_checker analyze \
  --input /home/null/Desktop/ShopifyGates \
  --output donation_gates.json \
  --concurrent \
  --workers 10
```

This will:
- ✅ Check all 15,000 Shopify sites
- ✅ Find donation-only sites
- ✅ Save results to `donation_gates.json`
- ✅ Take ~12-15 minutes
- ✅ **No ChromeDriver needed for this step!**

## 🛍️ Step 5: Test Cards (After ChromeDriver is installed)

Once you have donation sites and ChromeDriver installed:

```bash
cd /home/null/Desktop/Stripeify

# Start ChromeDriver in background
chromedriver --port=9515 &

# Test cards on donation sites
./target/release/shopify_checker test \
  --gates donation_gates.json \
  --output checker_results.json
```

## 🚀 Step 6: Auto Mode (Complete Pipeline)

Or do everything in one command:

```bash
cd /home/null/Desktop/Stripeify

# Install ChromeDriver first
sudo apt install chromium-chromedriver

# Start ChromeDriver
chromedriver --port=9515 &

# Run complete pipeline
./target/release/shopify_checker auto \
  --concurrent \
  --workers 10
```

## 📝 Common Commands

### Just Analyze (No ChromeDriver needed)
```bash
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker analyze --concurrent --workers 10
```

### Test with Random Amounts
```bash
cd /home/null/Desktop/Stripeify
chromedriver --port=9515 &
./target/release/shopify_checker test \
  --random-amounts \
  --min-amount 5.0 \
  --max-amount 50.0
```

### Get Help
```bash
cd /home/null/Desktop/Stripeify
./target/release/shopify_checker --help
./target/release/shopify_checker analyze --help
./target/release/shopify_checker test --help
./target/release/shopify_checker auto --help
```

## 🔧 Troubleshooting

### "No such file or directory"
**Problem:** You're in the wrong directory  
**Solution:** `cd /home/null/Desktop/Stripeify`

### "Command 'chromedriver' not found"
**Problem:** ChromeDriver not installed  
**Solution:** `sudo apt install chromium-chromedriver` (note: chromium, not chromeium)

### "Cannot connect to ChromeDriver"
**Problem:** ChromeDriver not running  
**Solution:** `chromedriver --port=9515 &`

## 💡 Pro Tips

1. **Start with analyze only** - You don't need ChromeDriver to find donation sites
2. **Use concurrent mode** - 10x faster with `--concurrent --workers 10`
3. **Test small first** - Use `--max 100` to test with 100 gates first
4. **Check results** - Look at `donation_gates.json` after analyzing

## 📊 Expected Results

From 15,000 gates, you should find:
- ~500-1000 sites with donation keywords
- ~50-200 verified Shopify donation sites
- Processing time: 12-15 minutes (concurrent) or 2 hours (sequential)

## 🎬 Recommended First Run

```bash
# 1. Navigate to project
cd /home/null/Desktop/Stripeify

# 2. Analyze first 100 gates (quick test)
./target/release/shopify_checker analyze \
  --max 100 \
  --concurrent \
  --workers 10

# 3. Check results
cat donation_gates.json

# 4. If it works, run on all gates
./target/release/shopify_checker analyze \
  --concurrent \
  --workers 10
```

---

**Remember:** Always `cd /home/null/Desktop/Stripeify` first!
