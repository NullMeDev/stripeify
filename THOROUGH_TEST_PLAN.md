# Thorough Testing Plan - Stripeify

**Date:** December 23, 2024  
**Objective:** Complete thorough testing of all components

---

## Test Environment

- **Gates Available:** 75 gate files in `/home/null/Desktop/ShopifyChunks/`
- **Cards Available:** 6 working cards in `working_cards.txt`
- **Binary:** `target/release/shopify_checker` (14.5 MB)
- **Config:** `config.json` (discovery mode, auth-only)

---

## Test Suite

### 1. ✅ ALREADY COMPLETED

- [x] Build & Compilation
- [x] CLI Interface (help, version)
- [x] Discovery Mode (2 gates, basic test)
- [x] Browser Automation (ChromeDriver)
- [x] Authorization-Only Mode
- [x] Valid Gates Database Creation

---

### 2. 🔄 DISCOVERY MODE - EXTENDED TESTING

#### Test 2.1: Medium Scale Discovery (10 gates)
**Purpose:** Test with more gates to verify scaling
```bash
./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 10 \
    --auth-only=true
```

**Expected:**
- Process 10 gates successfully
- Update valid_gates.json
- Live stats display working
- No memory leaks

#### Test 2.2: Discovery with Telegram
**Purpose:** Verify Telegram notifications
```bash
./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 5 \
    --telegram
```

**Expected:**
- Telegram messages sent to group -1003538559040
- Bot credit @MissNullMe appears
- Proper formatting

---

### 3. 🔄 ROTATE MODE TESTING

#### Test 3.1: Basic Rotation
**Purpose:** Test rotational gate strategy
```bash
./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file working_cards.txt \
    --output rotate_results.json
```

**Expected:**
- Find working gate
- Use for all cards
- Rotate when gate fails
- Output file created

#### Test 3.2: Rotation with Telegram
```bash
./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file working_cards.txt \
    --telegram
```

---

### 4. 🔄 SMART MODE TESTING

#### Test 4.1: Basic Smart Mode
**Purpose:** Test smart card strategy
```bash
./target/release/shopify_checker smart \
    --gates valid_gates.json \
    --cards-file working_cards.txt \
    --output smart_results.json
```

**Expected:**
- Try multiple cards per gate
- Find working card
- Use for remaining gates

---

### 5. 🔄 ANALYZE MODE TESTING

#### Test 5.1: Gate Analysis
**Purpose:** Test gate analyzer
```bash
./target/release/shopify_checker analyze \
    --input /home/null/Desktop/ShopifyChunks/ \
    --output analyzed_gates.json
```

**Expected:**
- Analyze all gate files
- Identify donation sites
- Create structured output

---

### 6. 🔄 PROXY TESTING

#### Test 6.1: Proxy Rotation
**Purpose:** Test proxy support
```bash
# First check if proxies.txt exists and has content
cat proxies.txt

# Then test with proxies
./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 5 \
    --proxies proxies.txt
```

**Expected:**
- Load proxies from file
- Rotate through proxies
- Handle proxy failures

---

### 7. 🔄 EDGE CASE TESTING

#### Test 7.1: Invalid Gates Directory
```bash
./target/release/shopify_checker discover \
    --gates-dir /nonexistent/path \
    --cards-file working_cards.txt \
    --max-gates 5
```

**Expected:**
- Graceful error message
- No crash
- Clear user feedback

#### Test 7.2: Invalid Cards File
```bash
./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file /nonexistent/cards.txt \
    --max-gates 5
```

**Expected:**
- Graceful error message
- No crash

#### Test 7.3: Empty Gate Files
```bash
# Create empty test file
mkdir -p test_empty_gates
touch test_empty_gates/empty.txt

./target/release/shopify_checker discover \
    --gates-dir test_empty_gates/ \
    --cards-file working_cards.txt
```

**Expected:**
- Skip empty files
- Continue processing

#### Test 7.4: Malformed Card Data
```bash
# Create test file with bad data
echo "invalid|card|data" > test_bad_cards.txt
echo "1234567890123456|13|2025|999" >> test_bad_cards.txt

./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file test_bad_cards.txt \
    --max-gates 2
```

**Expected:**
- Skip invalid cards
- Process valid cards
- Log warnings

---

### 8. 🔄 PERFORMANCE TESTING

#### Test 8.1: Speed Benchmark
```bash
time ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 20
```

**Measure:**
- Total runtime
- Cards per second
- Memory usage

#### Test 8.2: Memory Leak Test
```bash
# Run with valgrind if available
valgrind --leak-check=full ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 10
```

---

### 9. 🔄 INTEGRATION TESTING

#### Test 9.1: Full Workflow
```bash
# 1. Analyze gates
./target/release/shopify_checker analyze \
    --input /home/null/Desktop/ShopifyChunks/ \
    --output analyzed.json

# 2. Discover valid gates
./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --max-gates 10

# 3. Use discovered gates in rotate mode
./target/release/shopify_checker rotate \
    --gates valid_gates.json \
    --cards-file working_cards.txt \
    --telegram
```

---

### 10. 🔄 LONG-RUNNING STABILITY

#### Test 10.1: Extended Run
```bash
# Run for 1 hour with all 75 gates
timeout 3600 ./target/release/shopify_checker discover \
    --gates-dir /home/null/Desktop/ShopifyChunks/ \
    --cards-file working_cards.txt \
    --telegram
```

**Monitor:**
- Memory usage over time
- CPU usage
- Error rate
- Success rate

---

## Test Execution Order

1. ✅ Discovery Mode - Extended (Test 2.1)
2. ✅ Discovery with Telegram (Test 2.2)
3. ✅ Rotate Mode Basic (Test 3.1)
4. ✅ Smart Mode Basic (Test 4.1)
5. ✅ Analyze Mode (Test 5.1)
6. ✅ Proxy Testing (Test 6.1)
7. ✅ Edge Cases (Tests 7.1-7.4)
8. ✅ Performance (Tests 8.1-8.2)
9. ✅ Integration (Test 9.1)
10. ✅ Long-Running (Test 10.1) - Optional

---

## Success Criteria

### Must Pass:
- [ ] All 4 command modes work
- [ ] Telegram integration functional
- [ ] Edge cases handled gracefully
- [ ] No crashes or panics
- [ ] Valid output files created
- [ ] Authorization-only mode confirmed

### Should Pass:
- [ ] Proxy rotation working
- [ ] Performance acceptable (>0.5 cards/sec)
- [ ] Memory stable over time
- [ ] All documentation accurate

---

## Test Results Log

Results will be documented in: `THOROUGH_TEST_RESULTS.md`

---

## Notes

- ChromeDriver must be running on port 9515
- Telegram bot token must be valid
- Some tests may take 30+ minutes
- Monitor system resources during long tests
