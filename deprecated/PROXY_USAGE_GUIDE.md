# Proxy Support - Usage Guide

## Overview

Proxy support has been added to bypass 403 errors and rate limiting when checking donation gates. The implementation uses Chrome extensions for authenticated proxy support.

## Features

✅ **Authenticated Proxy Support** - Username/password authentication
✅ **Automatic Rotation** - Rotate proxies on failures
✅ **Failure Tracking** - Remove failed proxies from pool
✅ **HTTP/HTTPS Support** - Works with standard HTTP proxies
✅ **Seamless Integration** - Works with existing authorization-only mode

## Proxy Format

Proxies should be in the format:
```
host:port:username:password
```

### Example proxies.txt:
```
evo-pro.porterproxies.com:62345:PP_5J7SVIL0BJ-country-US-state-Florida:95cc2n4b
evo-pro.porterproxies.com:62345:PP_HEA2J45444-country-US-state-Nebraska:a0uqrrx5
evo-pro.porterproxies.com:62345:PP_FQVQM1J2U2-country-US-state-Florida:7otosm8v
```

## Usage

### Basic Usage (With Proxies)

```bash
# Start ChromeDriver
chromedriver --port=9515 &

# Run with proxies
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt \
  --proxy-file proxies.txt \
  --output results.json
```

### Authorization-Only Mode (Default + Proxies)

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt \
  --proxy-file proxies.txt \
  --auth-only=true
```

### Charge Mode (With Proxies)

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt \
  --proxy-file proxies.txt \
  --auth-only=false
```

### Without Proxies (Original Behavior)

```bash
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt
```

## How It Works

### 1. Proxy Loading
```
🔒 Loading proxies...
✓ Loaded 30 proxies
ℹ Proxy stats: 30 available, 0 failed
```

### 2. Proxy Selection
```
🔒 Using proxy: evo-pro.porterproxies.com:62345
```

### 3. Chrome Extension Creation
- Temporary directory created for extension
- `manifest.json` generated with proxy permissions
- `background.js` created with proxy configuration
- Extension loaded into Chrome

### 4. Proxy Rotation
- Proxies rotate on failures (403, connection errors)
- Failed proxies tracked (max 3 failures)
- Automatic fallback to next available proxy

## Proxy Pool Management

### Rotation Strategy
- **Round-robin**: Cycles through proxies sequentially
- **Failure tracking**: Removes proxies after 3 failures
- **Automatic recovery**: Failed proxies can be retried later

### Failure Handling
```
⚠️ Proxy evo-pro.porterproxies.com:62345 marked as failed (3+ failures)
🔒 Rotating to next proxy...
🔒 Using proxy: evo-pro.porterproxies.com:62345
```

## Testing Chunk Gates

The ShopifyGatesAndChunks directory contains gates split into 200-gate chunks. Test them with proxies:

```bash
# Test first chunk
./target/release/shopify_checker rotate \
  --gates ShopifyGatesAndChunks/chunk_001.txt \
  --cards-file cards.txt \
  --proxy-file proxies.txt
```

## Troubleshooting

### Issue: "Failed to load proxies"
**Solution:** Check proxy file format. Each line should be: `host:port:username:password`

### Issue: "No proxies available"
**Solution:** All proxies have failed. Check:
1. Proxy credentials are correct
2. Proxies are not blocked
3. Proxy service is active

### Issue: Still getting 403 errors
**Solution:**
1. Verify proxy is working: `curl --proxy http://user:pass@host:port https://example.com`
2. Try different proxies
3. Check if site has additional anti-bot measures

### Issue: Chrome extension not loading
**Solution:**
1. Check Chrome/Chromium version compatibility
2. Verify ChromeDriver is running
3. Check file permissions on temp directory

## Performance Considerations

### With Proxies:
- **Slower**: Proxy adds latency (~100-500ms per request)
- **More reliable**: Bypasses rate limiting and 403 errors
- **Scalable**: Can handle more requests with multiple proxies

### Without Proxies:
- **Faster**: Direct connection
- **Limited**: May hit rate limits quickly
- **Risk**: 403 errors on many gates

## Best Practices

1. **Use Multiple Proxies**: Distribute load across 10+ proxies
2. **Monitor Failures**: Watch for proxies being marked as failed
3. **Rotate Regularly**: Don't overuse single proxy
4. **Test First**: Verify proxies work before bulk testing
5. **Keep Credentials Secure**: Don't commit proxies.txt to git

## Example Workflow

### Complete Workflow with Proxies:

```bash
# 1. Start ChromeDriver
chromedriver --port=9515 &

# 2. Test with small batch first
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file test_cards.txt \
  --proxy-file proxies.txt \
  --max-gates 10 \
  --output test_results.json

# 3. Review results
cat test_results.json | jq

# 4. Run full batch if successful
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file 42000Dump.txt \
  --proxy-file proxies.txt \
  --output full_results.json

# 5. Stop ChromeDriver when done
pkill chromedriver
```

## Advanced Usage

### Custom Proxy Rotation

The proxy pool automatically rotates, but you can influence behavior by:

1. **Ordering proxies**: Put best proxies first in proxies.txt
2. **Duplicating proxies**: Add same proxy multiple times for more usage
3. **Regional proxies**: Group proxies by region for geo-specific gates

### Monitoring Proxy Health

```bash
# Watch for proxy failures in real-time
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt \
  --proxy-file proxies.txt 2>&1 | grep -E "proxy|failed"
```

## Security Notes

⚠️ **Important Security Considerations:**

1. **Credentials**: Proxy credentials are stored in memory only
2. **Temp Files**: Extension files are in temp directory (auto-cleaned)
3. **No Logging**: Proxy passwords are not logged
4. **HTTPS**: All proxy communication uses HTTPS when possible

## FAQ

**Q: Can I use SOCKS5 proxies?**
A: Currently only HTTP/HTTPS proxies are supported. SOCKS5 support may be added later.

**Q: How many proxies do I need?**
A: Minimum 5-10 for small batches, 20-30+ for large-scale testing.

**Q: Do proxies work with Telegram notifications?**
A: Yes! Proxies are transparent to Telegram integration.

**Q: Can I use free proxies?**
A: Not recommended. Free proxies are often slow, unreliable, and may be blocked.

**Q: What happens if all proxies fail?**
A: The program will continue without proxies and may encounter 403 errors.

## Support

For issues or questions:
1. Check this guide first
2. Review error messages carefully
3. Test proxies independently
4. Check proxy service status

---

**Proxy support is now fully integrated! Enjoy bypassing 403 errors! 🚀**
