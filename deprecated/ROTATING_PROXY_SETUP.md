# Rotating Proxy API Integration

## Your Rotating Proxy API Key
```
950d5fdb-1960-4718-9cce-ccd10e8654c5
```

## What is a Rotating Proxy API?

Instead of using a list of static proxies, a rotating proxy API:
- ✅ Automatically rotates IPs on each request
- ✅ No need to manage proxy list
- ✅ Better quality (residential IPs)
- ✅ Handles dead proxies automatically
- ✅ Much more reliable

## How to Use

### Format
```
http://USERNAME:API_KEY@gateway.example.com:PORT
```

### Common Rotating Proxy Providers

**1. Webshare.io**
```
http://username:950d5fdb-1960-4718-9cce-ccd10e8654c5@proxy.webshare.io:80
```

**2. Smartproxy**
```
http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@gate.smartproxy.com:7000
```

**3. Bright Data (Luminati)**
```
http://customer:950d5fdb-1960-4718-9cce-ccd10e8654c5@brd.superproxy.io:22225
```

**4. Oxylabs**
```
http://customer:950d5fdb-1960-4718-9cce-ccd10e8654c5@pr.oxylabs.io:7777
```

**5. IPRoyal**
```
http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@geo.iproyal.com:12321
```

## Setup Instructions

### Step 1: Identify Your Provider

Check your proxy provider's documentation for the correct format. Common patterns:

- **Webshare:** `proxy.webshare.io:80`
- **Smartproxy:** `gate.smartproxy.com:7000`
- **Bright Data:** `brd.superproxy.io:22225`
- **Oxylabs:** `pr.oxylabs.io:7777`
- **IPRoyal:** `geo.iproyal.com:12321`

### Step 2: Create Rotating Proxy File

```bash
cd /home/null/Desktop/Stripeify

# Replace with your actual provider's gateway
echo "http://username:950d5fdb-1960-4718-9cce-ccd10e8654c5@proxy.webshare.io:80" > rotating_proxy.txt
```

**Note:** Replace:
- `username` with your actual username (if required)
- `proxy.webshare.io:80` with your provider's gateway

### Step 3: Test the Proxy

```bash
# Test if proxy works
curl -x "http://username:950d5fdb-1960-4718-9cce-ccd10e8654c5@proxy.webshare.io:80" https://api.ipify.org

# Should return a different IP than your real IP
```

### Step 4: Use with Checker

```bash
# Use rotating proxy
./target/release/shopify_checker rotate \
  --gates full_test_gates.txt \
  --cards-file full_test_cards.txt \
  --proxy-file rotating_proxy.txt \
  --auth-only=true
```

## Advantages Over Static Proxies

| Feature | Static Proxies | Rotating Proxy API |
|---------|---------------|-------------------|
| IP Rotation | Manual | Automatic |
| Dead Proxies | Must remove manually | Handled automatically |
| Quality | Mixed | High (residential) |
| 403 Errors | Common | Rare |
| Management | High maintenance | Zero maintenance |
| Speed | Variable | Consistent |

## Configuration Examples

### Example 1: Webshare.io
```bash
# rotating_proxy.txt
http://myusername:950d5fdb-1960-4718-9cce-ccd10e8654c5@proxy.webshare.io:80
```

### Example 2: Smartproxy
```bash
# rotating_proxy.txt
http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@gate.smartproxy.com:7000
```

### Example 3: Bright Data
```bash
# rotating_proxy.txt
http://lum-customer-USERNAME:950d5fdb-1960-4718-9cce-ccd10e8654c5@brd.superproxy.io:22225
```

## Testing Your Setup

### Test 1: Check IP Rotation
```bash
# Run this 3 times - should get different IPs
for i in {1..3}; do
  curl -x "http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@gateway:port" https://api.ipify.org
  echo ""
done
```

### Test 2: Test with Checker
```bash
# Create test files
echo "https://example.com" > test_gate.txt
echo "4532015112830366|12|2027|999" > test_card.txt

# Test with rotating proxy
./target/release/shopify_checker rotate \
  --gates test_gate.txt \
  --cards-file test_card.txt \
  --proxy-file rotating_proxy.txt \
  --max-gates 1
```

## Troubleshooting

### Error: "Proxy connection failed"
**Solution:** Check your proxy format
```bash
# Correct format:
http://username:API_KEY@gateway.com:port

# Common mistakes:
https://...  # Should be http://
missing username
missing port
wrong gateway address
```

### Error: "Authentication failed"
**Solution:** Verify API key and username
```bash
# Test authentication
curl -x "http://user:API_KEY@gateway:port" https://api.ipify.org
```

### Error: "Still getting 403 errors"
**Solution:** Add delays between requests
```bash
# Use the safe testing script
./test_gates_safely.sh
```

## Best Practices

### 1. Use Rotating Proxy for All Tests
```bash
# Always specify rotating proxy
--proxy-file rotating_proxy.txt
```

### 2. Still Use Batch Testing
Even with rotating proxies, batch testing is recommended:
```bash
./test_gates_safely.sh
# Edit to use rotating_proxy.txt instead of proxies.txt
```

### 3. Monitor Usage
Most rotating proxy services have usage limits:
- Check your dashboard regularly
- Monitor bandwidth usage
- Watch for rate limit warnings

### 4. Backup Static Proxies
Keep your static proxies as backup:
```bash
# Primary: Rotating proxy
--proxy-file rotating_proxy.txt

# Backup: Static proxies
--proxy-file proxies.txt
```

## Quick Setup Script

```bash
#!/bin/bash
# setup_rotating_proxy.sh

echo "Rotating Proxy Setup"
echo "===================="
echo ""
echo "Your API Key: 950d5fdb-1960-4718-9cce-ccd10e8654c5"
echo ""
echo "Enter your proxy provider:"
echo "1) Webshare.io"
echo "2) Smartproxy"
echo "3) Bright Data"
echo "4) Oxylabs"
echo "5) IPRoyal"
echo "6) Custom"
read -p "Choice (1-6): " choice

case $choice in
  1)
    read -p "Enter username: " username
    echo "http://$username:950d5fdb-1960-4718-9cce-ccd10e8654c5@proxy.webshare.io:80" > rotating_proxy.txt
    ;;
  2)
    echo "http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@gate.smartproxy.com:7000" > rotating_proxy.txt
    ;;
  3)
    read -p "Enter customer ID: " customer
    echo "http://lum-customer-$customer:950d5fdb-1960-4718-9cce-ccd10e8654c5@brd.superproxy.io:22225" > rotating_proxy.txt
    ;;
  4)
    echo "http://customer:950d5fdb-1960-4718-9cce-ccd10e8654c5@pr.oxylabs.io:7777" > rotating_proxy.txt
    ;;
  5)
    echo "http://user:950d5fdb-1960-4718-9cce-ccd10e8654c5@geo.iproyal.com:12321" > rotating_proxy.txt
    ;;
  6)
    read -p "Enter full proxy URL: " custom
    echo "$custom" > rotating_proxy.txt
    ;;
esac

echo ""
echo "✓ Created rotating_proxy.txt"
echo ""
echo "Testing proxy..."
if curl -x "$(cat rotating_proxy.txt)" -s https://api.ipify.org; then
  echo ""
  echo "✓ Proxy works!"
else
  echo ""
  echo "✗ Proxy test failed. Check your configuration."
fi
```

## Usage with All Scripts

### Update find_valid_gates_and_cards.sh
```bash
# Change line:
--proxy-file proxies.txt

# To:
--proxy-file rotating_proxy.txt
```

### Update test_gates_safely.sh
```bash
# Change:
PROXY_FILE="proxies.txt"

# To:
PROXY_FILE="rotating_proxy.txt"
```

### Update run_production_auto.sh
```bash
# Change:
--proxy-file proxies.txt

# To:
--proxy-file rotating_proxy.txt
```

## Summary

**Your API Key:** `950d5fdb-1960-4718-9cce-ccd10e8654c5`

**Setup:**
1. Identify your proxy provider
2. Create `rotating_proxy.txt` with correct format
3. Test with `curl`
4. Use with `--proxy-file rotating_proxy.txt`

**Benefits:**
- ✅ Automatic IP rotation
- ✅ No 403 errors
- ✅ Better quality
- ✅ Zero maintenance

This will dramatically improve your success rate!
