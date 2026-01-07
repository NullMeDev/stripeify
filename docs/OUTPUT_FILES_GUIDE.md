# 📁 Output Files Guide - Where Results Are Saved

## 🎯 Quick Answer

**Charged cards and test results are saved to JSON files in the project directory.**

---

## 📊 Output Files by Mode

### 1. ANALYZE Mode Output

**File:** `donation_gates.json` (or custom name with `--output`)  
**Location:** `/home/null/Desktop/Stripeify/donation_gates.json`

**Contains:**
- All donation sites found from the 15,000 gates
- URL, payment gateway, Shopify status for each site

**Example:**
```json
[
  {
    "url": "https://charity-example.myshopify.com",
    "gateway": "Shopify Payments",
    "donation_form": true,
    "has_shopify": true,
    "has_shopify_payments": true,
    "payment_gateway": "Shopify Payments",
    "donation_keywords_count": 3
  }
]
```

**How to view:**
```bash
cd /home/null/Desktop/Stripeify
cat donation_gates.json
# Or with formatting:
cat donation_gates.json | python3 -m json.tool
```

---

### 2. TEST Mode Output

**Files Created:**
1. `checker_results.json` - All test results
2. `checker_results_working_gates.json` - **Working gates only** (JSON)
3. `checker_results_working_gates.txt` - **Working gates only** (text, one per line)

**Location:** `/home/null/Desktop/Stripeify/`

**checker_results.json Contains:**
- **ALL charged cards** with their details
- Gate URL where card was charged
- Amount charged
- Card (masked for security)
- Status (CHARGED, CVV_MISMATCH, INSUFFICIENT_FUNDS, etc.)

**checker_results_working_gates.json Contains:**
- **Only the gate URLs that successfully charged**
- Deduplicated (no duplicates)
- Ready to use for future testing

**checker_results_working_gates.txt Contains:**
- Same as JSON but in simple text format
- One gate URL per line
- Easy to copy/paste or use in scripts

**Example:**
```json
[
  {
    "gate": "https://charity1.myshopify.com",
    "card": "453201...123",
    "amount": 35.0,
    "status": "CHARGED",
    "success": true
  },
  {
    "gate": "https://donate2.myshopify.com",
    "card": "453201...123",
    "amount": 25.0,
    "status": "CVV_MISMATCH",
    "success": true
  },
  {
    "gate": "https://foundation3.myshopify.com",
    "card": "542523...456",
    "amount": 14.99,
    "status": "CHARGED",
    "success": true
  }
]
```

**How to view:**
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json
# Or with formatting:
cat checker_results.json | python3 -m json.tool
```

**How to filter:**
```bash
# Show only $35 charges
cat checker_results.json | python3 -m json.tool | grep -A 5 '"amount": 35'

# Count charges by amount
cat checker_results.json | python3 -m json.tool | grep '"amount"' | sort | uniq -c
```

---

### 3. AUTO Mode Output

**Files Created:**
1. `auto_donation_gates.json` - Donation sites found
2. `auto_checker_results.json` - All test results
3. `auto_checker_results_working_gates.json` - **Working gates only** (JSON)
4. `auto_checker_results_working_gates.txt` - **Working gates only** (text)

**Location:** `/home/null/Desktop/Stripeify/`

**auto_checker_results.json** contains the same format as TEST mode above.
**auto_checker_results_working_gates.json** and **.txt** contain only the gates that successfully charged.

**How to view:**
```bash
cd /home/null/Desktop/Stripeify
cat auto_checker_results.json | python3 -m json.tool
```

---

## 🔍 Finding Charged Cards

### Method 1: View the JSON File Directly

```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json
```

### Method 2: Pretty Print with Python

```bash
cd /home/null/Desktop/Stripeify
python3 -m json.tool checker_results.json
```

### Method 3: Filter by Status

```bash
# Show only CHARGED cards
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
charged = [r for r in data if r['status'] == 'CHARGED']
print(json.dumps(charged, indent=2))
"
```

### Method 4: Group by Amount

```bash
# Show charges grouped by amount
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -c "
import json, sys
from collections import defaultdict
data = json.load(sys.stdin)
by_amount = defaultdict(list)
for r in data:
    by_amount[r['amount']].append(r)
for amount in sorted(by_amount.keys(), reverse=True):
    print(f'\n\${amount} Gates ({len(by_amount[amount])} found):')
    for r in by_amount[amount]:
        print(f'  ✓ {r[\"gate\"]}')
        print(f'    Card: {r[\"card\"]}')
        print(f'    Status: {r[\"status\"]}\n')
"
```

---

## 📋 Output File Locations Summary

| Mode | Output File | Location | Contains |
|------|-------------|----------|----------|
| **analyze** | `donation_gates.json` | `/home/null/Desktop/Stripeify/` | Donation sites found |
| **test** | `checker_results.json` | `/home/null/Desktop/Stripeify/` | All test results |
| **test** | `checker_results_working_gates.json` | `/home/null/Desktop/Stripeify/` | **Working gates only** |
| **test** | `checker_results_working_gates.txt` | `/home/null/Desktop/Stripeify/` | **Working gates (text)** |
| **auto** | `auto_donation_gates.json` | `/home/null/Desktop/Stripeify/` | Donation sites found |
| **auto** | `auto_checker_results.json` | `/home/null/Desktop/Stripeify/` | All test results |
| **auto** | `auto_checker_results_working_gates.json` | `/home/null/Desktop/Stripeify/` | **Working gates only** |
| **auto** | `auto_checker_results_working_gates.txt` | `/home/null/Desktop/Stripeify/` | **Working gates (text)** |

---

## 💡 Custom Output Locations

You can specify custom output files:

### Analyze Mode
```bash
./target/release/shopify_checker analyze \
  --output /path/to/my_donation_sites.json
```

### Test Mode
```bash
./target/release/shopify_checker test \
  --output /path/to/my_results.json
```

---

## 🎯 Quick Commands

### View all charged cards:
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -m json.tool
```

### Count total charges:
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -c "import json, sys; print(len(json.load(sys.stdin)))"
```

### Find $35 gates:
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -c "
import json, sys
data = json.load(sys.stdin)
gates_35 = [r for r in data if r['amount'] == 35.0]
print(f'Found {len(gates_35)} \$35 gates:')
for r in gates_35:
    print(f'  {r[\"gate\"]} - Card: {r[\"card\"]} - Status: {r[\"status\"]}')
"
```

---

## 📊 Terminal Output

In addition to JSON files, the program also displays results in the terminal:

```
═══════════════════════════════════════════════════════════
✅ FINAL RESULTS
═══════════════════════════════════════════════════════════

💰 $35.00 Gates (5 found):
  ✓ https://charity1.myshopify.com
    Card: 453201...123
    Status: CHARGED

  ✓ https://donate2.myshopify.com
    Card: 453201...123
    Status: CHARGED

💰 $25.00 Gates (8 found):
  ✓ https://foundation3.myshopify.com
    Card: 542523...456
    Status: CVV_MISMATCH
  ...

✓ Results saved to checker_results.json
```

---

## 🔐 Security Note

The JSON files contain:
- ✅ Masked card numbers (e.g., `453201...123`)
- ✅ Gate URLs
- ✅ Amounts charged
- ✅ Status (CHARGED, DECLINED, etc.)

**NOT included:**
- ❌ Full card numbers
- ❌ CVV codes
- ❌ Expiry dates

The full card details are only in memory during testing and never saved to disk.

---

## 📁 File Structure After Running

```
/home/null/Desktop/Stripeify/
├── target/
│   └── release/
│       └── shopify_checker          # Binary
├── donation_gates.json              # Donation sites (from analyze)
├── checker_results.json             # CHARGED CARDS (from test)
├── auto_donation_gates.json         # Donation sites (from auto)
├── auto_checker_results.json        # CHARGED CARDS (from auto)
└── [documentation files]
```

---

## ✅ Summary

**Charged cards are saved to:**
- `checker_results.json` (when using `test` mode)
- `auto_checker_results.json` (when using `auto` mode)

**Location:** `/home/null/Desktop/Stripeify/`

**To view:**
```bash
cd /home/null/Desktop/Stripeify
cat checker_results.json | python3 -m json.tool
