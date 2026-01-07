use crate::common::{Gate, DONATION_KEYWORDS, ECOMMERCE_KEYWORDS};
use anyhow::{Context, Result};
use colored::*;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use scraper::{Html, Selector};
use std::fs;
use std::thread;
use std::time::Duration;

/// Load Shopify gates from text files in a directory
pub fn load_shopify_gates(directory: &str) -> Result<Vec<String>> {
    let pattern = format!("{}/*.txt", directory);
    let mut gates = Vec::new();
    
    for entry in glob(&pattern).context("Failed to read glob pattern")? {
        match entry {
            Ok(path) => {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {:?}", path))?;
                
                for line in content.lines() {
                    let url = line.trim();
                    if !url.is_empty() {
                        gates.push(url.to_string());
                    }
                }
            }
            Err(e) => eprintln!("Error reading file: {}", e),
        }
    }
    
    Ok(gates)
}

/// Analyze URL for donation keywords
pub fn analyze_url_keywords(url: &str) -> (bool, usize) {
    let url_lower = url.to_lowercase();
    
    // Count donation keywords
    let donation_score = DONATION_KEYWORDS
        .iter()
        .filter(|&&keyword| url_lower.contains(keyword))
        .count();
    
    // Count e-commerce keywords (but exclude "shop" if it's part of "myshopify.com")
    let ecommerce_score = ECOMMERCE_KEYWORDS
        .iter()
        .filter(|&&keyword| {
            // Special case: don't count "shop" if it's only in "myshopify.com"
            if keyword == "shop" && url_lower.contains("myshopify.com") {
                // Check if "shop" appears outside of "myshopify.com"
                let without_myshopify = url_lower.replace("myshopify.com", "");
                without_myshopify.contains("shop")
            } else {
                url_lower.contains(keyword)
            }
        })
        .count();
    
    // Is donation site if has donation keywords and no e-commerce keywords
    let is_donation = donation_score > 0 && ecommerce_score == 0;
    
    (is_donation, donation_score)
}

/// Check site content for Shopify and donation indicators
pub fn check_site_content(url: &str, timeout_secs: u64) -> Result<Gate> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }
    
    let html = response.text()?;
    let document = Html::parse_document(&html);
    let html_lower = html.to_lowercase();
    
    // Extract title
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());
    
    // Check for Shopify indicators
    let has_shopify = html_lower.contains("shopify")
        || url.to_lowercase().contains("myshopify.com")
        || html.contains("cdn.shopify.com")
        || html.contains("shopify-features")
        || html.contains("shopify.theme")
        || document.select(&Selector::parse("meta[name='shopify-checkout-api-token']").unwrap()).next().is_some()
        || document.select(&Selector::parse("script[src*='cdn.shopify.com']").unwrap()).next().is_some();
    
    // Check for Shopify Payments
    let has_shopify_payments = html_lower.contains("shopify payments")
        || html.contains("shopify.payment")
        || html.contains("checkout.shopify.com")
        || html_lower.contains("shopifypay")
        || html_lower.contains("shop_pay")
        || document.select(&Selector::parse("form[action*='checkout.shopify.com']").unwrap()).next().is_some();
    
    // Determine payment gateway
    let payment_gateway = if html_lower.contains("shopify payments") || html_lower.contains("shop pay") {
        Some("Shopify Payments".to_string())
    } else if html_lower.contains("stripe") && has_shopify {
        Some("Shopify + Stripe".to_string())
    } else if has_shopify {
        Some("Shopify (Unknown Gateway)".to_string())
    } else {
        None
    };
    
    // Check for donation form indicators
    let donation_form_selectors = vec![
        "form[id*='donat' i]",
        "form[class*='donat' i]",
        "input[name*='donat' i]",
        "input[name*='amount' i]",
        "input[name*='contribution' i]",
        "button:contains('donate')",
        "button:contains('give')",
        "button:contains('contribute')",
        "a[href*='donate' i]",
        "a[href*='donation' i]",
        "div[class*='donat' i]",
    ];
    
    let has_donation_form = donation_form_selectors.iter().any(|selector| {
        if let Ok(sel) = Selector::parse(selector) {
            document.select(&sel).next().is_some()
        } else {
            false
        }
    }) || (html_lower.contains("donation") && html_lower.contains("amount"));
    
    // Count donation keywords in content
    let donation_keywords_count = DONATION_KEYWORDS
        .iter()
        .filter(|&&keyword| html_lower.contains(keyword))
        .count();
    
    Ok(Gate {
        url: url.to_string(),
        gateway: payment_gateway.clone().unwrap_or_default(),
        donation_form: has_donation_form,
        title,
        has_shopify,
        has_shopify_payments,
        payment_gateway,
        donation_keywords_count,
    })
}

/// Analyze gates to find donation sites
pub fn analyze_gates(gates: Vec<String>, max_check: usize) -> Result<Vec<Gate>> {
    println!("\n{}", "🔍 Analyzing Shopify Gates for Donation Sites".bold().cyan());
    println!();
    
    // Step 1: Quick URL analysis
    println!("{}", "Step 1: Analyzing URLs for donation keywords...".yellow());
    
    let mut potential_sites: Vec<(String, usize)> = gates
        .iter()
        .filter_map(|url| {
            let (is_donation, score) = analyze_url_keywords(url);
            if is_donation {
                Some((url.clone(), score))
            } else {
                None
            }
        })
        .collect();
    
    println!("{} Found {} potential donation sites from URL analysis\n", 
        "✓".green(), potential_sites.len());
    
    // Sort by keyword score (highest first)
    potential_sites.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Step 2: Check top candidates
    let check_count = max_check.min(potential_sites.len());
    println!("{} Checking top {} sites for Shopify integration...\n", 
        "Step 2:".yellow(), check_count);
    
    let pb = ProgressBar::new(check_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let mut verified_sites = Vec::new();
    
    for (_i, (url, _score)) in potential_sites.iter().take(check_count).enumerate() {
        pb.set_message(format!("Checking: {}...", &url[..url.len().min(40)]));
        
        match check_site_content(url, 10) {
            Ok(gate) => {
                if gate.has_shopify {
                    verified_sites.push(gate);
                }
            }
            Err(_) => {
                // Skip sites that error out
            }
        }
        
        pb.inc(1);
        
        // Be respectful to servers
        thread::sleep(Duration::from_millis(500));
    }
    
    pb.finish_with_message("Done!");
    println!();
    
    Ok(verified_sites)
}

/// Display analysis results
pub fn display_results(sites: &[Gate]) {
    if sites.is_empty() {
        println!("\n{}", "❌ No donation sites with Shopify found".red());
        return;
    }
    
    println!("\n{} Found {} Donation Sites with Shopify Integration\n", 
        "✅".green(), sites.len());
    
    // Display table header
    println!("{}", "═".repeat(120).cyan());
    println!("{:<5} {:<40} {:<8} {:<25} {:<6} {:<8}", 
        "Rank".bold(), 
        "URL".bold(), 
        "Shopify".bold(), 
        "Gateway".bold(), 
        "Form".bold(),
        "Keywords".bold()
    );
    println!("{}", "═".repeat(120).cyan());
    
    // Display each site
    for (i, site) in sites.iter().enumerate() {
        let url_display = if site.url.len() > 40 {
            format!("{}...", &site.url[..37])
        } else {
            site.url.clone()
        };
        
        let gateway_display = site.payment_gateway
            .as_ref()
            .map(|g| if g.len() > 25 { format!("{}...", &g[..22]) } else { g.clone() })
            .unwrap_or_else(|| "N/A".to_string());
        
        println!("{:<5} {:<40} {:<8} {:<25} {:<6} {:<8}", 
            (i + 1).to_string().cyan(),
            url_display.blue(),
            if site.has_shopify { "✓".green() } else { "✗".red() },
            gateway_display.magenta(),
            if site.donation_form { "✓".green() } else { "✗".red() },
            site.donation_keywords_count.to_string().yellow()
        );
    }
    
    println!("{}", "═".repeat(120).cyan());
    
    // Show payment gateway summary
    println!("\n{}", "💳 Payment Gateways Detected:".bold().cyan());
    for (i, site) in sites.iter().enumerate() {
        if let Some(gateway) = &site.payment_gateway {
            println!("  {}. {}", (i + 1).to_string().cyan(), site.url);
            println!("     Gateway: {}", gateway.yellow());
            if site.has_shopify_payments {
                println!("     Shopify Payments: {}", "✓ Enabled".green());
            }
        }
    }
}

/// Save results to JSON file
pub fn save_results(sites: &[Gate], output_file: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(sites)?;
    fs::write(output_file, json)?;
    println!("\n{} Results saved to {}", "✓".green(), output_file);
    Ok(())
}
