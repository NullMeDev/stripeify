use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use thirtyfour::prelude::*;

// Exponential backoff amounts
const BACKOFF_AMOUNTS: [f64; 6] = [35.00, 25.00, 14.99, 4.99, 2.00, 1.00];

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Gate {
    url: String,
    #[serde(default)]
    gateway: String,
    #[serde(default)]
    donation_form: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckResult {
    gate: String,
    card: String,
    amount: f64,
    status: String,
    success: bool,
}

#[derive(Debug, Clone)]
struct CardData {
    number: String,
    month: String,
    year: String,
    cvv: String,
}

impl CardData {
    fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 4 {
            anyhow::bail!("Invalid card format. Use: number|month|year|cvv");
        }
        
        Ok(CardData {
            number: parts[0].to_string(),
            month: parts[1].to_string(),
            year: parts[2].to_string(),
            cvv: parts[3].to_string(),
        })
    }
    
    fn masked(&self) -> String {
        format!("{}...{}", &self.number[..6], &self.number[self.number.len()-3..])
    }
}

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
        if let Ok(element) = driver.find(By::Css(selector)).await {
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
        if let Ok(iframe) = driver.find(By::Css(selector)).await {
            driver.switch_to_frame(&iframe).await?;
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
                
                driver.switch_to_default_content().await?;
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
            if let Ok(element) = driver.find(By::Css(selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                element.send_keys(&card.number).await?;
                break;
            }
        }
        
        // Month
        for selector in &["input[name*='month']", "select[name*='month']"] {
            if let Ok(element) = driver.find(By::Css(selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                element.send_keys(&card.month).await?;
                break;
            }
        }
        
        // Year
        for selector in &["input[name*='year']", "select[name*='year']"] {
            if let Ok(element) = driver.find(By::Css(selector)).await {
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                element.send_keys(&card.year).await?;
                break;
            }
        }
        
        // CVV
        for selector in &["input[name*='cvv']", "input[name*='cvc']", "input[id*='cvv']"] {
            if let Ok(element) = driver.find(By::Css(selector)).await {
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
        if let Ok(button) = driver.find(By::Css(selector)).await {
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

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "🛍️  Shopify Donation Checker (Rust + Browser Automation)".bold().cyan());
    println!("{}\n", "No API keys needed - fills forms like a real user!".dimmed());
    
    // Load donation gates
    let gates_json = fs::read_to_string("donation_gates.json")
        .context("Failed to read donation_gates.json. Run gate_analyzer.py first!")?;
    let gates: Vec<Gate> = serde_json::from_str(&gates_json)?;
    
    println!("{} Loaded {} donation gates\n", "✓".green(), gates.len());
    
    // Get cards from user
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
    
    // Ask how many gates to test
    print!("\nHow many gates to test? (default: all): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let max_gates = input.trim().parse::<usize>().unwrap_or(gates.len()).min(gates.len());
    
    let gates_to_test = &gates[..max_gates];
    
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
    caps.add_chrome_arg("--headless")?;
    caps.add_chrome_arg("--no-sandbox")?;
    caps.add_chrome_arg("--disable-dev-shm-usage")?;
    caps.add_chrome_arg("--disable-gpu")?;
    caps.add_chrome_arg("--window-size=1920,1080")?;
    caps.add_chrome_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    
    println!("{} Browser ready\n", "✓".green());
    
    // Check cards
    let mut all_results = Vec::new();
    
    for card in &cards {
        println!("\n{}", format!("═══ Testing card: {} ═══", card.masked()).bold().cyan());
        
        for gate in gates_to_test {
            if let Ok(Some(result)) = check_card_on_gate(&driver, card, gate).await {
                all_results.push(result);
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    
    // Close browser
    driver.quit().await?;
    
    // Display results
    println!("\n{}", "═".repeat(60));
    println!("{}", "✅ FINAL RESULTS".bold().green());
    println!("{}\n", "═".repeat(60));
    
    if all_results.is_empty() {
        println!("{}", "❌ No successful charges found".red());
        println!("{}", "All cards declined on all gates".yellow());
    } else {
        // Group by amount
        use std::collections::HashMap;
        let mut by_amount: HashMap<String, Vec<&CheckResult>> = HashMap::new();
        for result in &all_results {
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
        
        // Save results
        let json = serde_json::to_string_pretty(&all_results)?;
        fs::write("checker_results.json", json)?;
        println!("{} Results saved to checker_results.json", "✓".green());
    }
    
    Ok(())
}
