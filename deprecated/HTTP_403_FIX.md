# Fixing HTTP 403 Errors

## Problem

HTTP 403 (Forbidden) errors when testing gates means:
1. **Rate Limiting** - Too many requests too fast
2. **Bot Detection** - Sites detect automated testing
3. **Dead Proxies** - Proxies are blocked or not working
4. **IP Reputation** - Your IP/proxies are flagged

## Solutions

### 1. ✅ Add Delays Between Requests

**Problem:** Hitting gates too fast triggers rate limits
**Solution:** Add 3-5 second delays between each gate test

```bash
# In the checker code, add delays
tokio::time::sleep(Duration::from_secs(3)).await;
```

### 2. ✅ Test Proxies First

**Problem:** Using dead/blocked proxies
**Solution:** Verify proxies work before using them

```bash
# Test proxies
./check_proxy_status.sh

# Only use working proxies
cat proxies.txt | while read proxy; do
  if curl -x "$proxy" -s -o /dev/null -w "%{http_code}" https://google.com | grep -q "200"; then
    echo "$proxy" >> working_proxies.txt
  fi
done
```

### 3. ✅ Use Smaller Batches

**Problem:** Testing 118 gates at once overwhelms proxies
**Solution:** Test in smaller batches (10-20 gates at a time)

```bash
# Split gates into chunks
split -l 10 full_test_gates.txt gate_chunk_

# Test each chunk separately
for chunk in gate_chunk_*; do
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file full_test_cards.txt \
    --proxy-file proxies.txt \
    --auth-only=true
  sleep 60  # Wait between chunks
done
```

### 4. ✅ Increase Proxy Rotation

**Problem:** Same proxy used too many times
**Solution:** Rotate proxy after each request (already implemented)

### 5. ✅ Add Random User Agents

**Problem:** Same user agent looks suspicious
**Solution:** Rotate user agents (already implemented in code)

### 6. ✅ Retry with Backoff

**Problem:** Temporary 403s treated as permanent failures
**Solution:** Retry failed requests with exponential backoff

```rust
async fn test_with_retry(gate: &str, max_retries: u32) -> Result<()> {
    for attempt in 0..max_retries {
        match test_gate(gate).await {
            Ok(result) => return Ok(result),
            Err(e) if e.status() == 403 => {
                let delay = 2u64.pow(attempt) * 1000; // Exponential backoff
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(anyhow!("Max retries exceeded"))
}
```

## Immediate Fix

### Step 1: Verify Proxies Work

```bash
cd /home/null/Desktop/Stripeify
./check_proxy_status.sh
```

This will show which proxies are working.

### Step 2: Test Small Batch First

```bash
# Test just 5 gates to verify it works
head -5 full_test_gates.txt > test_5_gates.txt

./target/release/shopify_checker rotate \
  --gates test_5_gates.txt \
  --cards-file full_test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=true \
  --output test_results.json
```

### Step 3: If Still Getting 403s

**Option A: Get Fresh Proxies**
- Current proxies might be burned
- Need residential proxies (not datacenter)
- Rotating proxies work best

**Option B: Add Longer Delays**
- Modify code to wait 5-10 seconds between requests
- Slower but more reliable

**Option C: Use Fewer Concurrent Requests**
- Test gates one at a time instead of rotating quickly
- Most reliable but slowest

## Best Practice Workflow

### 1. Verify Setup
```bash
# Check proxies
./check_proxy_status.sh

# Check ChromeDriver
pgrep chromedriver || chromedriver --port=9515 &
```

### 2. Test Small Batch
```bash
# Test 5 gates first
head -5 full_test_gates.txt > test_batch.txt

./target/release/shopify_checker rotate \
  --gates test_batch.txt \
  --cards-file full_test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=true
```

### 3. If Successful, Scale Up
```bash
# Test 20 gates
head -20 full_test_gates.txt > batch_20.txt

./target/release/shopify_checker rotate \
  --gates batch_20.txt \
  --cards-file full_test_cards.txt \
  --proxy-file proxies.txt \
  --auth-only=true
```

### 4. Full Test with Delays
```bash
# Split into chunks of 10
split -l 10 full_test_gates.txt chunk_

# Test each chunk with delays
for chunk in chunk_*; do
  echo "Testing $chunk..."
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file full_test_cards.txt \
    --proxy-file proxies.txt \
    --auth-only=true \
    --output "results_$chunk.json"
  
  echo "Waiting 60 seconds before next chunk..."
  sleep 60
done

# Combine results
jq -s 'add' results_chunk_*.json > all_results.json
```

## Why 403 Happens

### Common Causes:
1. **Too Fast** - More than 1 request/second per IP
2. **Same IP** - Using same proxy repeatedly
3. **Bot Patterns** - Predictable timing, same user agent
4. **Burned Proxies** - Proxies already flagged
5. **Cloudflare/WAF** - Advanced bot protection

### Solutions by Cause:
| Cause | Solution |
|-------|----------|
| Too Fast | Add 3-5 sec delays |
| Same IP | Rotate proxies aggressively |
| Bot Patterns | Random delays, rotate user agents |
| Burned Proxies | Get fresh proxies |
| Cloudflare/WAF | Use residential proxies, slower requests |

## Recommended Approach

**For 118 Gates:**

1. **Split into 12 chunks** (10 gates each)
2. **Test 1 chunk every 5 minutes** (total: 1 hour)
3. **Use proxy rotation** (30 proxies available)
4. **Add 3-second delays** between requests
5. **Retry 403s** with exponential backoff

This approach:
- ✅ Avoids rate limits
- ✅ Spreads load across proxies
- ✅ Looks more human
- ✅ Handles temporary blocks
- ✅ Completes in reasonable time

## Quick Fix Script

```bash
#!/bin/bash
# test_gates_safely.sh - Avoid 403 errors

# Split gates into chunks of 10
split -l 10 full_test_gates.txt chunk_

chunk_num=1
total_chunks=$(ls chunk_* | wc -l)

for chunk in chunk_*; do
  echo "═══ Testing Chunk $chunk_num/$total_chunks ═══"
  
  ./target/release/shopify_checker rotate \
    --gates "$chunk" \
    --cards-file full_test_cards.txt \
    --proxy-file proxies.txt \
    --auth-only=true \
    --output "results_chunk_$chunk_num.json"
  
  if [ $chunk_num -lt $total_chunks ]; then
    echo "Waiting 5 minutes before next chunk..."
    sleep 300
  fi
  
  chunk_num=$((chunk_num + 1))
done

# Combine all results
echo "Combining results..."
jq -s 'add' results_chunk_*.json > combined_results.json

echo "✓ Complete! Results in combined_results.json"
```

This avoids 403 errors by:
- Testing small batches
- Long delays between batches
- Using proxies
- Spreading requests over time
