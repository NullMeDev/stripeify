use colored::*;
use std::time::{Duration, Instant};

/// Statistics tracker for live display
#[derive(Debug, Clone)]
pub struct Stats {
    pub total_cards: usize,
    pub current_card_index: usize,
    pub current_batch: usize,
    pub total_batches: usize,
    pub approved: usize,
    pub declined: usize,
    pub cvv_mismatch: usize,
    pub insufficient_funds: usize,
    pub errors: usize,
    pub start_time: Instant,
    pub current_card: String,
    pub current_result: String,
    pub current_gate: String,
}

impl Stats {
    pub fn new(total_cards: usize) -> Self {
        let total_batches = (total_cards + 3) / 4; // 4 cards per batch
        Self {
            total_cards,
            current_card_index: 0,
            current_batch: 0,
            total_batches,
            approved: 0,
            declined: 0,
            cvv_mismatch: 0,
            insufficient_funds: 0,
            errors: 0,
            start_time: Instant::now(),
            current_card: String::new(),
            current_result: String::new(),
            current_gate: String::new(),
        }
    }
    
    pub fn update_current_gate(&mut self, gate: &str) {
        self.current_gate = gate.to_string();
    }

    pub fn update_current_card(&mut self, card: &str) {
        self.current_card = card.to_string();
        self.current_card_index += 1;
        self.current_batch = (self.current_card_index + 3) / 4;
    }

    pub fn record_result(&mut self, status: &str) {
        match status {
            "CHARGED" => {
                self.approved += 1;
                self.current_result = "✅ Approved".to_string();
            }
            "CVV_MISMATCH" => {
                self.cvv_mismatch += 1;
                self.current_result = "⚠️  CVV Mismatch".to_string();
            }
            "INSUFFICIENT_FUNDS" => {
                self.insufficient_funds += 1;
                self.current_result = "💰 Insufficient Funds".to_string();
            }
            "DECLINED" => {
                self.declined += 1;
                self.current_result = "❌ Declined".to_string();
            }
            _ => {
                self.errors += 1;
                self.current_result = "⚠️  Error".to_string();
            }
        }
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
        self.current_result = "⚠️  Error".to_string();
    }

    pub fn success_rate(&self) -> f64 {
        let total_tested = self.approved + self.declined + self.cvv_mismatch + self.insufficient_funds;
        if total_tested == 0 {
            0.0
        } else {
            (self.approved as f64 / total_tested as f64) * 100.0
        }
    }

    pub fn cards_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            0.0
        } else {
            self.current_card_index as f64 / elapsed
        }
    }

    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Display live stats - Static box with Midnight Purple, updates in place
    pub fn display(&self) {
        // Move cursor to top-left without clearing (keeps box static)
        print!("\x1B[1;1H");

        // Midnight Purple color
        let purple = "\x1B[38;2;102;51;153m";
        let reset = "\x1B[0m";

        // Helper function to safely pad strings
        let safe_pad = |text: &str, target_len: usize| -> String {
            let text_len = text.len();
            if text_len >= target_len {
                text.chars().take(target_len).collect()
            } else {
                let padding = target_len - text_len;
                format!("{}{}", text, " ".repeat(padding))
            }
        };

        // Top border
        println!("{}╔════════════════════════════════════════════════════╗{}", purple, reset);
        
        // Title
        println!("{}║  LIVE STATS                                        ║{}", purple, reset);
        
        // Separator
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        
        // Current gate (truncated)
        let gate_display = if self.current_gate.len() > 35 {
            format!("{}...", &self.current_gate[..35])
        } else {
            self.current_gate.clone()
        };
        let gate_text = format!("Gate:   {}", gate_display);
        println!("{}║  {}║{}", purple, safe_pad(&gate_text, 50), reset);
        
        // Current card (truncated for privacy)
        let card_display = if self.current_card.len() > 6 {
            format!("{}...{}", &self.current_card[..6], &self.current_card[self.current_card.len().saturating_sub(3)..])
        } else {
            self.current_card.clone()
        };
        let card_text = format!("Card:   {}", card_display);
        println!("{}║  {}║{}", purple, safe_pad(&card_text, 50), reset);
        
        // Result - Show clearly if Charged or Declined
        let result_display = if self.current_result.contains("Approved") || self.current_result.contains("CHARGED") {
            "✅ CHARGED"
        } else if self.current_result.contains("Declined") || self.current_result.contains("DECLINED") {
            "❌ DECLINED"
        } else if self.current_result.contains("CVV") {
            "⚠️  CVV MISMATCH"
        } else if self.current_result.contains("Insufficient") {
            "💰 INSUFFICIENT FUNDS"
        } else if self.current_result.is_empty() {
            "⏳ Testing..."
        } else {
            &self.current_result
        };
        let result_text = format!("Result: {}", result_display);
        println!("{}║  {}║{}", purple, safe_pad(&result_text, 50), reset);
        
        // Separator
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        
        // Progress
        let progress_text = format!("Progress: {}/{} cards (Batch {}/{})", 
            self.current_card_index, self.total_cards, 
            self.current_batch, self.total_batches);
        println!("{}║  {}║{}", purple, safe_pad(&progress_text, 50), reset);
        
        // Separator
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        
        // Stats line 1
        let stats1_text = format!("Approved: {}    Declined: {}", self.approved, self.declined);
        println!("{}║  {}║{}", purple, safe_pad(&stats1_text, 50), reset);
        
        // Stats line 2
        let stats2_text = format!("CVV: {}    Insuf: {}    Errors: {}", 
            self.cvv_mismatch, self.insufficient_funds, self.errors);
        println!("{}║  {}║{}", purple, safe_pad(&stats2_text, 50), reset);
        
        // Separator
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        
        // Performance stats
        let elapsed_secs = self.elapsed_time().as_secs_f64();
        let perf_text = format!("Success: {:>5.1}%  Speed: {:>5.2} c/s  Time: {:>7.1}s", 
            self.success_rate(), self.cards_per_second(), elapsed_secs);
        println!("{}║  {}║{}", purple, safe_pad(&perf_text, 50), reset);
        
        // Bottom border
        println!("{}╚════════════════════════════════════════════════════╝{}", purple, reset);
        
        // Flush to ensure immediate display
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }
    
    /// Initialize the display (draw box once at start)
    pub fn init_display() {
        // Clear screen once at the start
        print!("\x1B[2J\x1B[1;1H");
        
        let purple = "\x1B[38;2;102;51;153m";
        let reset = "\x1B[0m";
        
        // Draw initial empty box
        println!("{}╔════════════════════════════════════════════════════╗{}", purple, reset);
        println!("{}║  LIVE STATS                                        ║{}", purple, reset);
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        println!("{}║  Gate:   Scanning...                               ║{}", purple, reset);
        println!("{}║  Card:   Initializing...                           ║{}", purple, reset);
        println!("{}║  Result: ⏳ Starting...                            ║{}", purple, reset);
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        println!("{}║  Progress: 0/0 cards (Batch 0/0)                   ║{}", purple, reset);
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        println!("{}║  Approved: 0    Declined: 0                        ║{}", purple, reset);
        println!("{}║  CVV: 0    Insuf: 0    Errors: 0                   ║{}", purple, reset);
        println!("{}╠════════════════════════════════════════════════════╣{}", purple, reset);
        println!("{}║  Success:   0.0%  Speed:  0.00 c/s  Time:     0.0s ║{}", purple, reset);
        println!("{}╚════════════════════════════════════════════════════╝{}", purple, reset);
        
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }
}
