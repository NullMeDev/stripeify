# Proxy Support Implementation Plan

## Overview
Add proxy support to bypass 403 errors when checking gates. Proxies will rotate to distribute requests and avoid rate limiting.

## Proxy Format
```
host:port:username:password
```

Example:
```
evo-pro.porterproxies.com:62345:PP_5J7SVIL0BJ-country-US-state-Florida:95cc2n4b
```

## Implementation Steps

### 1. ✅ Proxy Module Created
- [x] `src/proxy.rs` - Proxy pool manager with rotation
- [x] Added to `src/lib.rs`
- [x] Added `rand` dependency to `Cargo.toml`

### 2. ✅ CLI Flag Added
- [x] Added `--proxy-file` flag to `rotate` command in `src/main.rs`
- [x] Updated function signature to pass proxy_file parameter

### 3. ⏳ Update checker_v3.rs
Need to modify `run_checker_v3` function to:
- Accept `proxy_file: Option<&str>` parameter
- Load proxy pool if proxy file provided
- Configure ChromeDriver to use proxies
- Rotate proxies on failures

### 4. ⏳ ChromeDriver Proxy Configuration
Two approaches:
1. **HTTP Proxy** - Set via ChromeOptions
2. **SOCKS Proxy** - Set via ChromeOptions
3. **Extension** - Create Chrome extension for auth proxies

For authenticated proxies (username:password), we need to use Chrome extension approach.

### 5. ⏳ Proxy Rotation Strategy
- Start with first proxy
- On 403 error or connection failure, rotate to next proxy
- Track proxy failures (max 3 failures per proxy)
- Remove failed proxies from pool
- If all proxies fail, exit with error

### 6. ⏳ Testing
- Test with single proxy
- Test with multiple proxies
- Test proxy rotation on failures
- Test with invalid proxies

## Code Changes Required

### src/checker_v3.rs

```rust
pub async fn run_checker_v3(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
    proxy_file: Option<&str>,  // NEW PARAMETER
) -> Result<()> {
    // Load proxy pool if provided
    let proxy_pool = if let Some(proxy_path) = proxy_file {
        Some(ProxyPool::from_file(proxy_path)?)
    } else {
        None
    };
    
    // ... rest of function
}
```

### ChromeDriver Setup with Proxy

```rust
async fn setup_driver_with_proxy(proxy: Option<&Proxy>) -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--headless")?;
    caps.add_chrome_arg("--no-sandbox")?;
    caps.add_chrome_arg("--disable-dev-shm-usage")?;
    
    if let Some(proxy) = proxy {
        // For authenticated proxies, we need to create a Chrome extension
        let proxy_extension = create_proxy_extension(proxy)?;
        caps.add_chrome_arg(&format!("--load-extension={}", proxy_extension))?;
        
        // Or use proxy server (for non-authenticated)
        // caps.add_chrome_arg(&format!("--proxy-server={}:{}", proxy.host, proxy.port))?;
    }
    
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    Ok(driver)
}
```

### Proxy Extension for Chrome

```rust
fn create_proxy_extension(proxy: &Proxy) -> Result<String> {
    // Create temporary directory for extension
    let ext_dir = tempfile::tempdir()?;
    let ext_path = ext_dir.path();
    
    // Create manifest.json
    let manifest = json!({
        "version": "1.0.0",
        "manifest_version": 2,
        "name": "Proxy Auth",
        "permissions": [
            "proxy",
            "tabs",
            "unlimitedStorage",
            "storage",
            "<all_urls>",
            "webRequest",
            "webRequestBlocking"
        ],
        "background": {
            "scripts": ["background.js"]
        },
        "minimum_chrome_version": "22.0.0"
    });
    
    fs::write(
        ext_path.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?
    )?;
    
    // Create background.js
    let background_js = format!(r#"
        var config = {{
            mode: "fixed_servers",
            rules: {{
                singleProxy: {{
                    scheme: "http",
                    host: "{}",
                    port: parseInt({})
                }},
                bypassList: ["localhost"]
            }}
        }};
        
        chrome.proxy.settings.set({{value: config, scope: "regular"}}, function() {{}});
        
        function callbackFn(details) {{
            return {{
                authCredentials: {{
                    username: "{}",
                    password: "{}"
                }}
            }};
        }}
        
        chrome.webRequest.onAuthRequired.addListener(
            callbackFn,
            {{urls: ["<all_urls>"]}},
            ['blocking']
        );
    "#, proxy.host, proxy.port, proxy.username, proxy.password);
    
    fs::write(ext_path.join("background.js"), background_js)?;
    
    Ok(ext_path.to_string_lossy().to_string())
}
```

## Usage

```bash
# With proxies
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt \
  --proxy-file proxies.txt

# Without proxies (current behavior)
./target/release/shopify_checker rotate \
  --gates production_gates.json \
  --cards-file cards.txt
```

## Benefits

1. **Bypass 403 Errors** - Rotate IPs to avoid rate limiting
2. **Distributed Requests** - Spread load across multiple IPs
3. **Resilience** - Continue working even if some proxies fail
4. **Scalability** - Add more proxies as needed

## Next Steps

1. Update `checker_v3.rs` function signature
2. Implement proxy extension creation
3. Integrate proxy pool with driver setup
4. Add proxy rotation logic
5. Test with real proxies
6. Document usage

## Files to Modify

- [x] `src/proxy.rs` - Created
- [x] `src/lib.rs` - Updated
- [x] `Cargo.toml` - Updated
- [x] `src/main.rs` - Updated
- [ ] `src/checker_v3.rs` - Needs update
- [ ] Create proxy extension helper functions
- [ ] Add error handling for proxy failures
- [ ] Add logging for proxy usage

## Testing Checklist

- [ ] Test with valid proxy
- [ ] Test with invalid proxy
- [ ] Test proxy rotation
- [ ] Test with no proxies (current behavior)
- [ ] Test with multiple proxies
- [ ] Test proxy failure handling
- [ ] Test with chunk gates from ShopifyGatesAndChunks/
