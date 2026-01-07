# Stripeify - Donation Gate Analyzer & Tester

**Version**: 1.0.0  
**Purpose**: Analyze Shopify gates to find donation sites with Stripe integration

## 🎯 What's This?

This is a **separate project** from MadyOriginal that:
1. Analyzes 15,000+ Shopify gates
2. Identifies donation/charity sites
3. Finds sites with Stripe integration
4. Tests card validation on donation gates
5. Ranks gates by reliability

## 📁 Project Structure

```
Stripeify/
├── gate_analyzer.py       ← Analyzes gates, finds donation sites
├── gate_tester.py         ← Tests cards on found gates (to be created)
├── donation_gates.json    ← Results from analysis
├── mady.py               ← Modified for multi-gate support
├── requirements.txt      ← Dependencies
└── README.md            ← This file
```

## 🚀 Quick Start

### Step 1: Analyze Gates

```bash
cd /home/null/Desktop/Stripeify
python3 gate_analyzer.py
```

This will:
- Load 15,000+ Shopify gates from `/home/null/Desktop/ShopifyGates`
- Analyze URLs for donation keywords
- Check sites for Stripe integration
- Find donation forms
- Save results to `donation_gates.json`

### Step 2: Test Gates (Coming Next)

```bash
python3 gate_tester.py
```

This will:
- Load donation gates from analysis
- Test each gate with sample cards
- Measure success rates
- Rank gates by reliability

## 🔍 How It Works

### Gate Analysis Process

1. **URL Keyword Analysis**
   - Scans URLs for donation-related keywords
   - Filters out e-commerce sites
   - Scores potential donation sites

2. **Content Verification**
   - Checks if site is accessible
   - Looks for Stripe integration
   - Finds donation forms
   - Extracts Stripe public keys

3. **Ranking**
   - Ranks by donation keyword count
   - Prioritizes sites with clear donation forms
   - Identifies Stripe-enabled sites

### Donation Keywords

The analyzer looks for:
- donate, donation, charity, foundation
- nonprofit, fundraising, giving
- support, contribute, help, cause
- relief, aid, mission, humanitarian

### Stripe Detection

Finds:
- `pk_live_*` or `pk_test_*` keys
- Stripe.js integration
- Payment form elements

## 📊 Expected Results

From 15,000 gates, you can expect:
- **~500-1000** sites with donation keywords in URL
- **~50-100** verified donation sites with Stripe
- **~10-20** high-quality donation gates for testing

## 🎯 Best Donation Gates

After analysis, the best gates will have:
- ✅ Clear donation form
- ✅ Stripe integration
- ✅ Accessible and fast
- ✅ Simple checkout process
- ✅ Good error messages

## 🔧 Configuration

### Analysis Settings

Edit `gate_analyzer.py` to adjust:
```python
max_check = 100  # How many sites to check in detail
timeout = 10     # Request timeout in seconds
```

### Testing Settings

Edit `gate_tester.py` to adjust:
```python
test_cards_per_gate = 5   # Cards to test per gate
delay_between_tests = 5   # Seconds between tests
```

## 📝 Output Files

### donation_gates.json
```json
[
  {
    "url": "https://example-charity.myshopify.com",
    "title": "Example Charity - Donate",
    "has_stripe": true,
    "has_donation_form": true,
    "stripe_key": "pk_live_xxxxx",
    "donation_keywords_count": 5
  }
]
```

## 🚨 Important Notes

### This is a SEPARATE Project

- ✅ **Stripeify** - Testing donation gates from Shopify list
- ✅ **MadyOriginal** - Your working CC Foundation checker
- ❌ **NO MERGING** - These projects stay separate

### Why Separate?

1. **Different Purpose**
   - MadyOriginal: Production-ready with CC Foundation
   - Stripeify: Experimental gate testing

2. **Different Gates**
   - MadyOriginal: Proven donation sites
   - Stripeify: Testing Shopify-based sites

3. **Risk Isolation**
   - Keep working code safe
   - Test new gates separately
   - Merge only proven gates

## 🎓 Usage Workflow

### Recommended Process

1. **Run Analysis**
   ```bash
   python3 gate_analyzer.py
   ```
   - Finds donation sites
   - Saves to `donation_gates.json`

2. **Review Results**
   - Check `donation_gates.json`
   - Manually verify top sites
   - Note Stripe keys

3. **Test Gates**
   ```bash
   python3 gate_tester.py
   ```
   - Tests with sample cards
   - Measures success rates
   - Ranks by reliability

4. **Select Best Gates**
   - Choose top 5-10 gates
   - Add to MadyOriginal (optional)
   - Or keep in Stripeify for testing

## 🔐 Security Notes

### For Personal Use Only

- This tool is for **authorized testing only**
- Use only with your own cards or test cards
- Respect site terms of service
- Don't abuse donation sites

### Rate Limiting

- Built-in delays between requests
- Respectful to servers
- Avoids triggering anti-fraud systems

## 📈 Next Steps

After running the analyzer:

1. **Review top donation sites found**
2. **Manually verify they're legitimate charities**
3. **Test with a few cards to confirm functionality**
4. **Select best 5-10 gates for regular use**
5. **Optionally add to MadyOriginal as alternatives**

## 🆚 Stripeify vs MadyOriginal

| Feature | MadyOriginal | Stripeify |
|---------|--------------|-----------|
| **Purpose** | Production checker | Gate discovery |
| **Gates** | CC Foundation (proven) | Shopify sites (testing) |
| **Status** | Stable, working | Experimental |
| **Use Case** | Daily card checking | Finding new gates |
| **Risk** | Low (proven gate) | Higher (untested) |

## 🎯 Goal

Find **5-10 reliable donation gates** from the 15,000 Shopify sites that can serve as:
- Backup gates for MadyOriginal
- Alternative testing options
- Redundancy if CC Foundation changes

---

**Status**: 🔧 In Development  
**Last Updated**: 2025-12-21  
**Separate From**: MadyOriginal (no merging)
