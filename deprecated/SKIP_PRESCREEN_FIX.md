# Skip Pre-screening Fix for 403 Errors

## Problem

The HTTP pre-screening step uses `reqwest` (HTTP client) which gets blocked with 403 errors, even without proxies. This prevents any gates from being tested.

## Root Cause

```rust
async fn http_prescreen_gates(gates: &[Gate]) -> Vec<Gate> {
    // Uses reqwest HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    
    // Gets 403 errors from Shopify
    match client.get(&gate.url).send().await {
        // All gates return 403
    }
}
```

## Solution

Add `--skip-prescreen` flag to bypass HTTP checking and go straight to browser testing.

## Implementation

### Option 1: Use Smart Mode (Already Has This)

The `smart` subcommand doesn't do HTTP pre-screening:

```bash
echo "y" | ./target/release/shopify_checker smart \
  --gates full_test_gates.txt \
  --cards-file full_test_cards.txt \
  --auth-only=true
```

### Option 2: Modify rotate Mode

Add `--skip-prescreen` flag to `rotate` mode.

## Quick Fix (Use Smart Mode)

```bash
cd /home/null/Desktop/Stripeify

# Smart mode bypasses HTTP pre-screening
echo "y" | ./target/release/shopify_checker smart \
  --gates test_10_gates.txt \
  --cards-file full_test_cards.txt \
  --auth-only=true \
  --max-gates 10
```

## Why Smart Mode Works

Smart mode:
1. ✅ Skips HTTP pre-screening
2. ✅ Goes straight to browser testing
3. ✅ Uses browser to bypass 403 blocks
4. ✅ Tests cards intelligently

## Comparison

| Feature | Rotate Mode | Smart Mode |
|---------|-------------|------------|
| HTTP Pre-screen | ✅ Yes (gets 403) | ❌ No |
| Browser Testing | ✅ Yes | ✅ Yes |
| Card Rotation | ❌ No | ✅ Yes |
| Bypasses 403 | ❌ No | ✅ Yes |

## Recommended Command

```bash
# Create test file
head -10 full_test_gates.txt > test_10_gates.txt

# Run smart mode (no pre-screening)
echo "y" | ./target/release/shopify_checker smart \
  --gates test_10_gates.txt \
  --cards-file full_test_cards.txt \
  --auth-only=true
```

This will:
- Skip HTTP pre-screening (avoids 403)
- Use browser directly (bypasses blocks)
- Test 10 gates with all cards
- Use auth-only mode (FREE)
- Find valid gates

## Expected Output

```
🧠 SMART CARD MODE: Intelligent Card Rotation
   ✨ Try multiple cards per gate until one works
   ✨ Once card works, use it for ALL remaining gates

🔐 AUTHORIZATION-ONLY MODE
   ✓ Using wrong CVV (999) - no charges

✓ Loaded 707 cards

Testing gate 1/10: https://example.com
  → Trying card 1... ✓ CVV_MISMATCH
  → Using this card for remaining gates

Testing gate 2/10: https://example2.com
  → Using working card... ✓ CVV_MISMATCH

...

✅ Found 5 valid gates
```

## Summary

**Problem:** HTTP pre-screening gets 403 errors
**Solution:** Use `smart` mode which skips pre-screening
**Command:** `shopify_checker smart` instead of `shopify_checker rotate`
