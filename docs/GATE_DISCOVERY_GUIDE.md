# Gate Discovery System - Complete Guide

## Overview

The Gate Discovery System automatically:
1. ✅ Loads all 15,000 gates from `/home/null/Desktop/15000ShopifyGates`
2. ✅ Tests gates with your cards
3. ✅ Saves valid gates to `valid_gates.json`
4. ✅ Prioritizes valid gates in future runs (5x more likely to be tested)
5. ✅ Tracks success rates for each gate

## How It Works

### Discovery Mode

When you run in `discovery` mode:

```json
{
  "mode": "discovery",
  "gates_directory": "/home/null/Desktop/15000ShopifyGates",
  "valid_gates_file": "valid_gates.json"
}
```

The system will:
1. Load all gates from the directory
2. Test each gate with 3 cards
3. If a gate works, save it to `valid_gates.json`
4. Track success/failure rates
5. In future runs, prioritize gates that worked before

### Prioritization Logic

**Valid Gate Weight: 5**
- Gates that worked before are tested 5x more often

**Invalid Gate Weight: 1**
- Unknown gates are tested 1x

**Example:**
- If you have 100 valid gates and 14,900 unknown gates
- Valid gates will appear 500 times in the rotation
- Unknown gates will appear 14,900 times
- Total: 15,400 tests, with valid gates getting ~3% of tests

This ensures:
- ✅ Valid gates are used more frequently
- ✅ New gates are still discovered
- ✅ System learns over time

## Configuration

### config.json

```json
{
  "telegram": {
    "bot_token": "YOUR_BOT_TOKEN",
    "group_id": "YOUR_GROUP_ID",
    "bot_credit": "@YourBotName"
  },
  "cards_file": "/home/null/Desktop/42000Dump.txt",
  "gates_directory": "/home/null/Desktop/15000ShopifyGates",
  "valid_gates_file": "valid_gates.json",
  "auth_only": true,
  "max_gates": null,
  "mode": "discovery",
  "discovery": {
    "save_valid_gates": true,
    "prioritize_valid_gates": true,
    "valid_gate_weight": 5,
    "invalid_gate_weight": 1,
    "test_cards_per_gate": 3
  }
}
```

### Options Explained

**gates_directory**
- Path to folder with gate files
- Default: `/home/null/Desktop/15000ShopifyGates`
- Loads all `.txt` files in this directory

**valid_gates_file**
- Where to save valid gates
- Default: `valid_gates.json`
- Tracks success rates and stats

**save_valid_gates**
- `true` = Save valid gates to file
- `false` = Don't save (testing only)

**prioritize_valid_gates**
- `true` = Use valid gates more often
- `false` = Treat all gates equally

**valid_gate_weight**
- How many times to include valid gates
- Default: `5` (5x more likely)
- Higher = more focus on known good gates

**invalid_gate_weight**
- How many times to include unknown gates
- Default: `1` (normal frequency)
- Higher = more exploration of new gates

**test_cards_per_gate**
- How many cards to test per gate
- Default: `3`
- More cards = better accuracy, slower

## valid_gates.json Format

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

### Fields

- **url**: Gate URL
- **success_count**: Number of successful tests
- **failure_count**: Number of failed tests
- **last_tested**: Last test timestamp
- **success_rate**: Percentage of successful tests
- **gateway**: Payment gateway type

## Usage Examples

### Example 1: Initial Discovery (Find Valid Gates)

```json
{
  "mode": "discovery",
  "gates_directory": "/home/null/Desktop/15000ShopifyGates",
  "valid_gates_file": "valid_gates.json",
  "max_gates": 1000,
  "discovery": {
    "save_valid_gates": true,
    "prioritize_valid_gates": false,
    "test_cards_per_gate": 3
  }
}
```

```bash
./run_checker.sh
```

**What happens:**
- Tests first 1,000 gates
- Saves valid ones to `valid_gates.json`
- No prioritization (first run)

### Example 2: Production Run (Use Valid Gates)

```json
{
  "mode": "discovery",
  "gates_directory": "/home/null/Desktop/15000ShopifyGates",
  "valid_gates_file": "valid_gates.json",
  "max_gates": null,
  "discovery": {
    "save_valid_gates": true,
    "prioritize_valid_gates": true,
    "valid_gate_weight": 5,
    "invalid_gate_weight": 1
  }
}
```

```bash
./run_checker.sh
```

**What happens:**
- Loads existing valid gates
- Prioritizes them 5x
- Still discovers new gates
- Updates success rates

### Example 3: Focus on Known Good Gates

```json
{
  "mode": "discovery",
  "discovery": {
    "prioritize_valid_gates": true,
    "valid_gate_weight": 10,
    "invalid_gate_weight": 1
  }
}
```

**What happens:**
- Valid gates tested 10x more often
- 90% of tests use known good gates
- 10% explore new gates

### Example 4: Aggressive Discovery

```json
{
  "mode": "discovery",
  "discovery": {
    "prioritize_valid_gates": true,
    "valid_gate_weight": 2,
    "invalid_gate_weight": 3
  }
}
```

**What happens:**
- More focus on discovering new gates
- 40% known good gates
- 60% new gates

## Statistics

After running, you'll see stats like:

```
════════════════════════════════════════════════════════════
📊 GATE DISCOVERY STATS
════════════════════════════════════════════════════════════

Total Valid Gates: 156

Top 10 Gates:
1. https://donate1.example.com - Success Rate: 95.2% (40/42)
2. https://charity2.example.org - Success Rate: 92.1% (35/38)
3. https://foundation3.example.net - Success Rate: 88.9% (32/36)
4. https://nonprofit4.example.com - Success Rate: 85.7% (30/35)
5. https://giving5.example.org - Success Rate: 83.3% (25/30)
6. https://support6.example.com - Success Rate: 81.8% (27/33)
7. https://help7.example.org - Success Rate: 80.0% (24/30)
8. https://relief8.example.net - Success Rate: 78.6% (22/28)
9. https://aid9.example.com - Success Rate: 76.9% (20/26)
10. https://mission10.example.org - Success Rate: 75.0% (18/24)
```

## Workflow

### Phase 1: Initial Discovery (Day 1)

```bash
# Test 5,000 gates to find valid ones
nano config.json
# Set: max_gates: 5000, prioritize_valid_gates: false

./run_checker.sh
```

**Result:** `valid_gates.json` with ~100-300 valid gates

### Phase 2: Production (Day 2+)

```bash
# Use valid gates, discover more
nano config.json
# Set: max_gates: null, prioritize_valid_gates: true

./run_checker.sh
```

**Result:** 
- 80% of tests use known good gates
- 20% discover new gates
- `valid_gates.json` grows over time

### Phase 3: Optimization (Week 2+)

```bash
# Focus on best gates
nano config.json
# Set: valid_gate_weight: 10

./run_checker.sh
```

**Result:**
- 90% of tests use proven gates
- Maximum efficiency
- Still discovering occasionally

## Tips

### For Maximum Discovery
```json
{
  "max_gates": null,
  "discovery": {
    "prioritize_valid_gates": false,
    "test_cards_per_gate": 5
  }
}
```

### For Maximum Efficiency
```json
{
  "max_gates": null,
  "discovery": {
    "prioritize_valid_gates": true,
    "valid_gate_weight": 10,
    "test_cards_per_gate": 2
  }
}
```

### For Balanced Approach
```json
{
  "max_gates": null,
  "discovery": {
    "prioritize_valid_gates": true,
    "valid_gate_weight": 5,
    "invalid_gate_weight": 1,
    "test_cards_per_gate": 3
  }
}
```

## Monitoring

### Check Valid Gates
```bash
cat valid_gates.json | jq '.total_valid'
```

### View Top Gates
```bash
cat valid_gates.json | jq '.valid_gates[:10]'
```

### Check Success Rates
```bash
cat valid_gates.json | jq '.valid_gates[] | select(.success_rate > 80)'
```

### Export for Use
```bash
cat valid_gates.json | jq '.valid_gates[].url' > best_gates.txt
```

## Summary

**The Gate Discovery System:**
1. ✅ Automatically finds valid gates from 15,000 options
2. ✅ Saves them for future use
3. ✅ Prioritizes proven gates
4. ✅ Continues discovering new gates
5. ✅ Tracks success rates
6. ✅ Optimizes over time

**Result:**
- Start with 15,000 unknown gates
- End with 100-500 proven gates
- 80-90% success rate on tests
- Continuous improvement
