use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use shopify_checker::analyzer;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "shopify_checker")]
#[command(about = "Unified Shopify donation site analyzer and card checker", long_about = None)]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze Shopify gates to find donation sites
    Analyze {
        /// Directory containing gate text files
        #[arg(short, long, default_value = "/home/null/Desktop/ShopifyGates")]
        input: String,
        
        /// Output JSON file for donation gates
        #[arg(short, long, default_value = "donation_gates.json")]
        output: String,
        
        /// Maximum number of sites to check in detail (default: all)
        #[arg(short, long)]
        max: Option<usize>,
        
        /// Enable concurrent processing (faster but more resource intensive)
        #[arg(short, long)]
        concurrent: bool,
        
        /// Number of concurrent workers (default: 10)
        #[arg(short, long, default_value = "10")]
        workers: usize,
    },
    
    /// Test cards on donation sites using browser automation
    Test {
        /// Input gates JSON file
        #[arg(short, long, default_value = "donation_gates.json")]
        gates: String,
        
        /// Output results JSON file
        #[arg(short, long, default_value = "checker_results.json")]
        output: String,
        
        /// Maximum number of gates to test (default: all)
        #[arg(short, long)]
        max_gates: Option<usize>,
        
        /// Use random amounts instead of exponential backoff
        #[arg(short, long)]
        random_amounts: bool,
        
        /// Minimum random amount (default: 1.0)
        #[arg(long, default_value = "1.0")]
        min_amount: f64,
        
        /// Maximum random amount (default: 50.0)
        #[arg(long, default_value = "50.0")]
        max_amount: f64,
        
        /// Load cards from file instead of stdin (format: number|month|year|cvv, one per line)
        #[arg(long)]
        cards_file: Option<String>,
        
        /// Telegram configuration file (enables Telegram notifications)
        #[arg(long)]
        telegram_config: Option<String>,
    },
    
    /// Test cards with LIVE STATS display (improved version with better error handling)
    TestLive {
        /// Input gates JSON file
        #[arg(short, long, default_value = "production_gates.json")]
        gates: String,
        
        /// Output results JSON file
        #[arg(short, long, default_value = "live_results.json")]
        output: String,
        
        /// Maximum number of gates to test (default: all)
        #[arg(short, long)]
        max_gates: Option<usize>,
        
        /// Cards file (REQUIRED - format: number|month|year|cvv, one per line)
        #[arg(short, long, required = true)]
        cards_file: String,
        
        /// Telegram configuration file (enables Telegram notifications)
        #[arg(long)]
        telegram_config: Option<String>,
    },
    
    /// ROTATIONAL GATE MODE: Find working gate, use it for all cards until it fails, then rotate
    Rotate {
        /// Input gates JSON file
        #[arg(short, long, default_value = "production_gates.json")]
        gates: String,
        
        /// Output results JSON file
        #[arg(short, long, default_value = "rotate_results.json")]
        output: String,
        
        /// Maximum number of gates to test (default: all)
        #[arg(short, long)]
        max_gates: Option<usize>,
        
        /// Cards file (REQUIRED - format: number|month|year|cvv, one per line)
        #[arg(short, long, required = true)]
        cards_file: String,
        
        /// Telegram configuration file (enables Telegram notifications)
        #[arg(long)]
        telegram_config: Option<String>,
        
        /// Authorization-only mode: Use wrong CVV to check cards WITHOUT charging (default: true)
        #[arg(long, default_value = "true")]
        auth_only: bool,
    },
    
    /// Auto mode: Analyze gates then test cards automatically
    Auto {
        /// Directory containing gate text files
        #[arg(short, long, default_value = "/home/null/Desktop/ShopifyGates")]
        input: String,
        
        /// Maximum number of sites to analyze (default: all)
        #[arg(short, long)]
        max_analyze: Option<usize>,
        
        /// Maximum number of gates to test (default: all found)
        #[arg(long)]
        max_test: Option<usize>,
        
        /// Enable concurrent processing for analysis
        #[arg(short, long)]
        concurrent: bool,
        
        /// Number of concurrent workers (default: 10)
        #[arg(short, long, default_value = "10")]
        workers: usize,
        
        /// Use random amounts instead of exponential backoff
        #[arg(short, long)]
        random_amounts: bool,
        
        /// Minimum random amount (default: 1.0)
        #[arg(long, default_value = "1.0")]
        min_amount: f64,
        
        /// Maximum random amount (default: 50.0)
        #[arg(long, default_value = "50.0")]
        max_amount: f64,
    },
}

fn print_banner() {
    println!("{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║     Shopify Checker - Unified Rust Implementation        ║".cyan());
    println!("{}", "║     Analyze gates & test cards with browser automation   ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    print_banner();
    
    match cli.command {
        Commands::Analyze { input, output, max, concurrent, workers } => {
            run_analyzer(&input, &output, max, concurrent, workers)?;
        }
        Commands::Test { gates, output, max_gates, random_amounts, min_amount, max_amount, cards_file, telegram_config } => {
            run_checker(&gates, &output, max_gates, random_amounts, min_amount, max_amount, cards_file.as_deref(), telegram_config.as_deref())?;
        }
        Commands::TestLive { gates, output, max_gates, cards_file, telegram_config } => {
            run_checker_live(&gates, &output, max_gates, &cards_file, telegram_config.as_deref())?;
        }
        Commands::Rotate { gates, output, max_gates, cards_file, telegram_config, auth_only } => {
            run_checker_rotate(&gates, &output, max_gates, &cards_file, telegram_config.as_deref(), auth_only)?;
        }
        Commands::Auto {
            input, 
            max_analyze, 
            max_test, 
            concurrent, 
            workers, 
            random_amounts, 
            min_amount, 
            max_amount 
        } => {
            run_auto_mode(
                &input, 
                max_analyze, 
                max_test, 
                concurrent, 
                workers, 
                random_amounts, 
                min_amount, 
                max_amount
            )?;
        }
    }
    
    Ok(())
}

fn run_analyzer(input_dir: &str, output_file: &str, max_check: Option<usize>, concurrent: bool, workers: usize) -> Result<()> {
    println!("\n{}", "🔍 ANALYZE MODE: Finding Donation Sites".bold().green());
    println!();
    
    // Load gates
    println!("{}", "Loading Shopify gates...".yellow());
    let gates = analyzer::load_shopify_gates(input_dir)?;
    println!("{} Loaded {} Shopify gates\n", "✓".green(), gates.len());
    
    // Ask user how many to check
    let max_check = if let Some(max) = max_check {
        max
    } else {
        println!("{}", format!("Total gates loaded: {}", gates.len()).yellow());
        println!("{}", "Recommendation: Check all gates for complete database".cyan());
        
        print!("How many sites to check in detail? (Enter 'all' or number, default: all): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        
        if input.is_empty() || input == "all" {
            gates.len()
        } else {
            input.parse::<usize>().unwrap_or(gates.len())
        }
    };
    
    println!("{} Will check {} gates\n", "✓".green(), max_check);
    
    // Analyze gates
    let verified_sites = analyzer::analyze_gates(gates, max_check)?;
    
    // Display results
    analyzer::display_results(&verified_sites);
    
    // Save results
    if !verified_sites.is_empty() {
        analyzer::save_results(&verified_sites, output_file)?;
        
        println!("\n{}", "═".repeat(60).cyan());
        println!("{}", "✅ Analysis Complete!".bold().green());
        println!("{}", "═".repeat(60).cyan());
        println!("\n{}", "Next steps:".bold().yellow());
        println!("  1. Review the donation sites in {}", output_file.cyan());
        println!("  2. Run {} to test cards", "shopify_checker test".cyan());
        println!();
    }
    
    Ok(())
}

fn run_checker(
    gates_file: &str, 
    output_file: &str, 
    max_gates: Option<usize>, 
    random_amounts: bool, 
    min_amount: f64, 
    max_amount: f64,
    cards_file: Option<&str>,
    telegram_config: Option<&str>,
) -> Result<()> {
    println!("\n{}", "🛍️  TEST MODE: Checking Cards on Donation Sites".bold().green());
    println!();
    
    if random_amounts {
        println!("{}", format!("Using random amounts between ${:.2} and ${:.2}", min_amount, max_amount).yellow());
    } else {
        println!("{}", "Using exponential backoff: $35 → $25 → $14.99 → $4.99 → $2 → $1".yellow());
    }
    
    if let Some(config_path) = telegram_config {
        println!("{}", format!("✓ Telegram notifications enabled (config: {})", config_path).green());
    }
    
    if let Some(cards_path) = cards_file {
        println!("{}", format!("✓ Loading cards from file: {}", cards_path).green());
    }
    
    println!();
    
    // Check if ChromeDriver is needed
    println!("{}", "⚠️  Important: ChromeDriver must be running on port 9515".yellow());
    println!("{}", "   Start it with: chromedriver --port=9515 &".dimmed());
    println!();
    
    print!("Is ChromeDriver running? (y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        println!("\n{}", "Please start ChromeDriver first:".yellow());
        println!("  {}", "chromedriver --port=9515 &".cyan());
        println!("\nThen run this command again.");
        return Ok(());
    }
    
    // Run the async checker
    tokio::runtime::Runtime::new()?.block_on(async {
        checker::run_checker(gates_file, output_file, max_gates, cards_file, telegram_config).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Testing Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!();
    
    Ok(())
}

fn run_auto_mode(
    input_dir: &str,
    max_analyze: Option<usize>,
    max_test: Option<usize>,
    concurrent: bool,
    workers: usize,
    random_amounts: bool,
    min_amount: f64,
    max_amount: f64,
) -> Result<()> {
    println!("\n{}", "🚀 AUTO MODE: Complete Pipeline".bold().green());
    println!("{}", "   Step 1: Analyze gates to find donation sites".cyan());
    println!("{}", "   Step 2: Test cards on found donation sites".cyan());
    println!();
    
    if concurrent {
        println!("{}", format!("Using {} concurrent workers for analysis", workers).yellow());
    }
    if random_amounts {
        println!("{}", format!("Using random amounts between ${:.2} and ${:.2}", min_amount, max_amount).yellow());
    }
    println!();
    
    // Step 1: Analyze
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "STEP 1: ANALYZING GATES".bold().yellow());
    println!("{}", "═".repeat(60).cyan());
    
    let temp_gates_file = "auto_donation_gates.json";
    run_analyzer(input_dir, temp_gates_file, max_analyze, concurrent, workers)?;
    
    // Check if we found any donation sites
    let gates_content = std::fs::read_to_string(temp_gates_file)?;
    let gates: Vec<serde_json::Value> = serde_json::from_str(&gates_content)?;
    
    if gates.is_empty() {
        println!("\n{}", "❌ No donation sites found. Cannot proceed to testing.".red());
        return Ok(());
    }
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "STEP 2: TESTING CARDS".bold().yellow());
    println!("{}", "═".repeat(60).cyan());
    println!();
    
    // Step 2: Test
    let output_file = "auto_checker_results.json";
    run_checker(temp_gates_file, output_file, max_test, random_amounts, min_amount, max_amount, None, None)?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ AUTO MODE COMPLETE!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Files created:".bold().yellow());
    println!("  {} - Donation sites found", temp_gates_file.cyan());
    println!("  {} - Card testing results", output_file.cyan());
    println!();
    
    Ok(())
}

fn run_checker_live(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
) -> Result<()> {
    use shopify_checker::checker_v2;
    
    println!("\n{}", "🎮 LIVE MODE: Real-time Stats Display".bold().green());
    println!("{}", "   ✨ Midnight Purple & Lime Green UI".cyan());
    println!("{}", "   ✨ Better element detection & retry logic".cyan());
    println!("{}", "   ✨ Smart retry: Stop on first success per card".cyan());
    println!();
    
    if let Some(config_path) = telegram_config {
        println!("{}", format!("✓ Telegram notifications enabled (config: {})", config_path).green());
    }
    
    println!("{}", format!("✓ Loading cards from: {}", cards_file).green());
    println!();
    
    // Check if ChromeDriver is running
    println!("{}", "⚠️  Important: ChromeDriver must be running on port 9515".yellow());
    println!("{}", "   Start it with: chromedriver --port=9515 &".dimmed());
    println!();
    
    print!("Is ChromeDriver running? (y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        println!("\n{}", "Please start ChromeDriver first:".yellow());
        println!("  {}", "chromedriver --port=9515 &".cyan());
        println!("\nThen run this command again.");
        return Ok(());
    }
    
    // Run the improved async checker with live stats
    tokio::runtime::Runtime::new()?.block_on(async {
        checker_v2::run_checker_v2(gates_file, output_file, max_gates, cards_file, telegram_config).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Live Testing Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!();
    
    Ok(())
}

fn run_checker_rotate(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
) -> Result<()> {
    use shopify_checker::checker_v3;
    
    println!("\n{}", "🔄 ROTATIONAL GATE MODE: Smart Gate Rotation".bold().green());
    println!("{}", "   ✨ Find working gate first".cyan());
    println!("{}", "   ✨ Use one gate for ALL cards".cyan());
    println!("{}", "   ✨ Rotate only when gate fails (3 consecutive failures)".cyan());
    println!("{}", "   ✨ Much more efficient than testing each card on all gates".cyan());
    println!();
    
    if let Some(config_path) = telegram_config {
        println!("{}", format!("✓ Telegram notifications enabled (config: {})", config_path).green());
    }
    
    println!("{}", format!("✓ Loading cards from: {}", cards_file).green());
    println!();
    
    // Check if ChromeDriver is running
    println!("{}", "⚠️  Important: ChromeDriver must be running on port 9515".yellow());
    println!("{}", "   Start it with: chromedriver --port=9515 &".dimmed());
    println!();
    
    print!("Is ChromeDriver running? (y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        println!("\n{}", "Please start ChromeDriver first:".yellow());
        println!("  {}", "chromedriver --port=9515 &".cyan());
        println!("\nThen run this command again.");
        return Ok(());
    }
    
    // Run the rotational gate checker
    tokio::runtime::Runtime::new()?.block_on(async {
        checker_v3::run_checker_v3(gates_file, output_file, max_gates, cards_file, telegram_config, auth_only).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Rotational Testing Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!();
    
    Ok(())
}

