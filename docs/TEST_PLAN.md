# Comprehensive Test Plan - Rust Unified Binary

## 🎯 Testing Objectives

1. Verify all functionality works correctly
2. Ensure performance improvements are realized
3. Validate error handling
4. Confirm output matches Python version
5. Test edge cases and error scenarios

## 📋 Test Categories

### 1. Build & Installation Tests

#### 1.1 Compilation
- [x] Project compiles without errors
- [x] All dependencies resolve correctly
- [x] Binary is created successfully
- [x] Binary size is reasonable (~8MB)

#### 1.2 Help Commands
- [ ] `shopify_checker --help` shows main help
- [ ] `shopify_checker --version` shows version
- [ ] `shopify_checker analyze --help` shows analyzer help
- [ ] `shopify_checker test --help` shows checker help

### 2. Analyzer Tests (`shopify_checker analyze`)

#### 2.1 Basic Functionality
- [ ] Load gates from default directory
- [ ] Load gates from custom directory (`--input`)
- [ ] Handle missing directory gracefully
- [ ] Handle empty directory gracefully
- [ ] Parse gate URLs correctly

#### 2.2 URL Analysis
- [ ] Identify donation keywords in URLs
- [ ] Filter out e-commerce keywords
- [ ] Score URLs correctly
- [ ] Sort by relevance

#### 2.3 Site Content Checking
- [ ] Make HTTP requests successfully
- [ ] Parse HTML content
- [ ] Detect Shopify integration
- [ ] Identify payment gateways
- [ ] Detect donation forms
- [ ] Handle timeouts gracefully
- [ ] Handle connection errors
- [ ] Handle HTTP errors (404, 500, etc.)

#### 2.4 Progress & Output
- [ ] Progress bar displays correctly
- [ ] Progress updates in real-time
- [ ] Results table formats correctly
- [ ] JSON output is valid
- [ ] JSON structure matches expected format
- [ ] Save to default file (donation_gates.json)
- [ ] Save to custom file (`--output`)

#### 2.5 Edge Cases
- [ ] Handle 0 gates found
- [ ] Handle all gates failing
- [ ] Handle partial failures
- [ ] Handle very large gate lists (15,000+)
- [ ] Handle special characters in URLs
- [ ] Handle redirects
- [ ] Handle SSL errors

#### 2.6 Performance
- [ ] Faster than Python version
- [ ] Memory usage is reasonable
- [ ] Respects rate limits (500ms delay)
- [ ] Handles concurrent requests properly

### 3. Checker Tests (`shopify_checker test`)

#### 3.1 Prerequisites
- [ ] Detect missing ChromeDriver
- [ ] Provide helpful error message
- [ ] Verify ChromeDriver is running on port 9515

#### 3.2 Gate Loading
- [ ] Load gates from default file
- [ ] Load gates from custom file (`--gates`)
- [ ] Handle missing file gracefully
- [ ] Handle invalid JSON gracefully
- [ ] Handle empty gate list

#### 3.3 Card Input
- [ ] Accept card in correct format (number|month|year|cvv)
- [ ] Validate card format
- [ ] Reject invalid formats
- [ ] Handle multiple cards
- [ ] Display masked card numbers
- [ ] Handle empty input (no cards)

#### 3.4 Browser Automation
- [ ] Launch headless Chrome successfully
- [ ] Navigate to donation pages
- [ ] Find and fill amount fields
- [ ] Click amount buttons (if no input field)
- [ ] Handle Stripe iframes
- [ ] Fill card details in iframe
- [ ] Fill card details directly (no iframe)
- [ ] Fill email and name fields
- [ ] Click submit buttons
- [ ] Wait for responses

#### 3.5 Exponential Backoff
- [ ] Try $35 first
- [ ] Try $25 if $35 fails
- [ ] Try $14.99 if $25 fails
- [ ] Try $4.99 if $14.99 fails
- [ ] Try $2 if $4.99 fails
- [ ] Try $1 if $2 fails
- [ ] Stop at first success
- [ ] Mark as declined if all fail

#### 3.6 Response Analysis
- [ ] Detect "CHARGED" correctly
- [ ] Detect "CVV_MISMATCH" correctly
- [ ] Detect "INSUFFICIENT_FUNDS" correctly
- [ ] Detect "DECLINED" correctly
- [ ] Handle unexpected responses
- [ ] Handle timeouts
- [ ] Handle browser crashes

#### 3.7 Results & Output
- [ ] Group results by amount
- [ ] Display results table correctly
- [ ] Save to default file (checker_results.json)
- [ ] Save to custom file (`--output`)
- [ ] JSON output is valid
- [ ] JSON structure matches expected format

#### 3.8 Edge Cases
- [ ] Handle 0 successful charges
- [ ] Handle all gates declining
- [ ] Handle browser crashes mid-test
- [ ] Handle network errors
- [ ] Handle malformed donation pages
- [ ] Handle missing form fields
- [ ] Handle JavaScript-heavy pages
- [ ] Handle rate limiting

#### 3.9 Performance
- [ ] Faster than Python version
- [ ] Memory usage is reasonable
- [ ] Browser cleanup works properly
- [ ] No memory leaks

### 4. CLI Tests

#### 4.1 Argument Parsing
- [ ] Parse `analyze` subcommand
- [ ] Parse `test` subcommand
- [ ] Parse `--input` argument
- [ ] Parse `--output` argument
- [ ] Parse `--max` argument
- [ ] Parse `--gates` argument
- [ ] Parse `--max-gates` argument
- [ ] Handle invalid arguments
- [ ] Handle missing required arguments

#### 4.2 Interactive Prompts
- [ ] Prompt for number of sites to check
- [ ] Accept "all" as input
- [ ] Accept numbers as input
- [ ] Handle invalid input
- [ ] Prompt for ChromeDriver confirmation
- [ ] Prompt for proceed confirmation
- [ ] Handle Ctrl+C gracefully

#### 4.3 Error Messages
- [ ] Clear error for missing files
- [ ] Clear error for invalid JSON
- [ ] Clear error for missing ChromeDriver
- [ ] Clear error for network issues
- [ ] Clear error for invalid arguments
- [ ] Helpful suggestions included

### 5. Integration Tests

#### 5.1 Complete Workflow
- [ ] Run analyzer to find gates
- [ ] Verify JSON output
- [ ] Run checker with analyzer output
- [ ] Verify results match expectations
- [ ] Verify performance is better than Python

#### 5.2 Data Flow
- [ ] Analyzer output feeds into checker
- [ ] JSON format is compatible
- [ ] All fields are preserved
- [ ] No data loss between steps

### 6. Comparison Tests (Rust vs Python)

#### 6.1 Functionality
- [ ] Same gates found
- [ ] Same Shopify detection
- [ ] Same payment gateway identification
- [ ] Same donation form detection
- [ ] Same card testing logic
- [ ] Same response analysis

#### 6.2 Performance
- [ ] Rust is faster (measure times)
- [ ] Rust uses less memory (measure usage)
- [ ] Rust has faster startup
- [ ] Rust handles more concurrent operations

#### 6.3 Output
- [ ] JSON format matches
- [ ] Results are equivalent
- [ ] No regressions

### 7. Stress Tests

#### 7.1 Large Datasets
- [ ] Test with 15,000 gates
- [ ] Test with 100+ cards
- [ ] Test with 500+ donation sites
- [ ] Monitor memory usage
- [ ] Monitor CPU usage
- [ ] Verify no crashes

#### 7.2 Error Scenarios
- [ ] Network disconnects mid-run
- [ ] ChromeDriver crashes
- [ ] Disk full (can't save results)
- [ ] Invalid HTML responses
- [ ] Malformed JSON inputs

### 8. Security Tests

#### 8.1 Input Validation
- [ ] Sanitize file paths
- [ ] Validate URLs
- [ ] Validate card formats
- [ ] Prevent path traversal
- [ ] Prevent command injection

#### 8.2 Data Handling
- [ ] Card numbers are masked in output
- [ ] Sensitive data not logged
- [ ] Secure HTTP connections
- [ ] Proper error handling (no data leaks)

## 🧪 Test Execution Plan

### Phase 1: Basic Functionality (30 min)
1. Test help commands
2. Test analyzer with 10 gates
3. Test checker with 1 gate and 1 card
4. Verify JSON outputs

### Phase 2: Core Features (45 min)
1. Test analyzer with 100 gates
2. Test all argument combinations
3. Test error handling
4. Test interactive prompts
5. Verify progress bars and output formatting

### Phase 3: Integration (30 min)
1. Complete workflow: analyze → test
2. Test with multiple cards
3. Test with multiple gates
4. Verify results accuracy

### Phase 4: Performance (30 min)
1. Benchmark analyzer vs Python
2. Benchmark checker vs Python
3. Memory profiling
4. Stress test with large datasets

### Phase 5: Edge Cases (30 min)
1. Test all error scenarios
2. Test malformed inputs
3. Test network issues
4. Test browser crashes

### Phase 6: Comparison (30 min)
1. Run same tests in Python and Rust
2. Compare outputs
3. Compare performance
4. Document differences

## 📊 Success Criteria

### Must Pass
- ✅ All basic functionality tests
- ✅ All core feature tests
- ✅ Integration tests pass
- ✅ No crashes or panics
- ✅ Output matches Python version
- ✅ Performance is better than Python

### Should Pass
- ✅ All edge case tests
- ✅ All error handling tests
- ✅ All stress tests
- ✅ Memory usage is reasonable
- ✅ No memory leaks

### Nice to Have
- ✅ 10x faster than Python
- ✅ 5x less memory than Python
- ✅ Handles 15,000 gates smoothly
- ✅ Zero compilation warnings

## 📝 Test Results Template

```markdown
## Test Results - [Date]

### Environment
- OS: Linux 6.16
- Rust Version: [version]
- Binary Size: [size]

### Analyzer Tests
- Basic Functionality: ✅/❌
- URL Analysis: ✅/❌
- Site Checking: ✅/❌
- Progress & Output: ✅/❌
- Edge Cases: ✅/❌
- Performance: ✅/❌

### Checker Tests
- Prerequisites: ✅/❌
- Gate Loading: ✅/❌
- Card Input: ✅/❌
- Browser Automation: ✅/❌
- Exponential Backoff: ✅/❌
- Response Analysis: ✅/❌
- Results & Output: ✅/❌
- Edge Cases: ✅/❌
- Performance: ✅/❌

### Integration Tests
- Complete Workflow: ✅/❌
- Data Flow: ✅/❌

### Performance Benchmarks
- Analyzer: [X]x faster than Python
- Checker: [X]x faster than Python
- Memory: [X]MB vs [Y]MB (Python)
- Startup: [X]ms vs [Y]ms (Python)

### Issues Found
1. [Issue description]
2. [Issue description]

### Conclusion
[Overall assessment]
```

## 🚀 Next Steps After Testing

1. Fix any bugs found
2. Optimize performance bottlenecks
3. Improve error messages
4. Add missing features
5. Update documentation
6. Create release notes
