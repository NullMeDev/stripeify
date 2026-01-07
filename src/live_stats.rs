use colored::*;
use std::io::{self, Write};
use std::time::{Duration, Instant};

pub struct LiveStats {
    total_cards: usize,
    current_card_index: usize,
    current_card: String,
    current_result: String,
    
    // Counters
    approved: usize,
    declined: usize,
    cvv_mismatch: usize,
    insufficient_funds: usize,
    errors: usize,
    
    // Batch info
    cards_per_batch: usize,
    current_batch: usize,
    total_batches: usize,
    
    // Performance
    start_time: Instant,
    cards_processed: usize,
    
    // Approved cards list
    approved_cards: Vec<String>,
}

impl LiveStats {
    pub fn new(total_cards: usize, cards_per_batch: usize) -> Self {
        let total_batches = (total_cards + cards_per_batch - 1) / cards_per_batch;
        
        Self {
            total_cards,
            current_card_index: 0,
            current_card: String::new(),
            current_result: "❌".to_string(),
            approved: 0,
            declined: 0,
            cvv_mismatch: 0,
            insufficient_funds: 0,
            errors: 0,
            cards_per_batch,
            current_batch: 1,
            total_batches,
            start_time: Instant::now(),
            cards_processed: 0,
            approved_cards: Vec::new(),
        }
    }
    
    pub fn update_card(&mut self, card: &str, index: usize) {
        self.current_card = Self::mask_card(card);
        self.current_card_index = index + 1;
        self.current_batch = (index / self.cards_per_batch) + 1;
        self.current_result = "⏳".to_string();
    }
    
    pub fn record_result(&mut self, status: &str) {
        self.cards_processed += 1;
        
        match status {
            "CHARGED" | "CVV_MISMATCH" | "INSUFFICIENT_FUNDS" => {
                self.approved += 1;
                self.current_result = "✅".to_string();
                
                // Add to approved cards list with status
                let card_with_status = format!("{} [{}]", self.current_card, status);
                self.approved_cards.push(card_with_status);
                
                if status == "CVV_MISMATCH" {
                    self.cvv_mismatch += 1;
                } else if status == "INSUFFICIENT_FUNDS" {
                    self.insufficient_funds += 1;
                }
            }
            "DECLINED" => {
                self.declined += 1;
                self.current_result = "❌".to_string();
            }
            _ => {
                self.errors += 1;
                self.current_result = "⚠️".to_string();
            }
        }
    }
    
    pub fn display(&self) {
        let success_rate = if self.cards_processed > 0 {
            (self.approved as f64 / self.cards_processed as f64) * 100.0
        } else {
            0.0
        };
        
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            self.cards_processed as f64 / elapsed
        } else {
            0.0
        };
        
        // Move cursor to home position (top-left) without clearing screen
        print!("\x1B[H");
        
        // Print the static frame (only once at start, but we'll print it each time for simplicity)
        println!("╔═══════════════════════════╦══════════════════════════════╗");
        println!("║ LIVE STATS                ║ Mady v2.0 @MissNullMe     ║");
        println!("╠═══════════════════════════╩══════════════════════════════╣");
        
        // Dynamic content - card and result
        print!("║ Card: {:<50} ║\r", self.current_card);
        println!();
        print!("║ Result: {:<48} ║\r", self.current_result);
        println!();
        
        println!("╠══════════════════════════════════════════════════════════╣");
        
        // Dynamic content - progress
        print!("║ Progress: {}/{} cards (Batch {}/{})                   ║\r", 
            self.current_card_index,
            self.total_cards,
            self.current_batch,
            self.total_batches
        );
        println!();
        
        println!("╠══════════════════════════════════════════════════════════╣");
        
        // Dynamic content - counters
        print!("║ ✓ {}   ✗ {}   CVV {}   Insuf {}   Err {}                  ║\r",
            self.approved,
            self.declined,
            self.cvv_mismatch,
            self.insufficient_funds,
            self.errors
        );
        println!();
        
        println!("╠══════════════════════════════════════════════════════════╣");
        
        // Dynamic content - stats
        print!("║ Success: {:>6.1}%  Speed: {:>5.2} c/s  Time: {:>8.1}s          ║\r",
            success_rate,
            speed,
            elapsed
        );
        println!();
        
        println!("╚══════════════════════════════════════════════════════════╝");
        
        // Display approved cards list below the stats box
        if !self.approved_cards.is_empty() {
            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║ ✅ APPROVED CARDS ({} total)                              ║", self.approved_cards.len());
            println!("╠══════════════════════════════════════════════════════════╣");
            
            // Show last 10 approved cards (most recent at bottom)
            let start_idx = if self.approved_cards.len() > 10 {
                self.approved_cards.len() - 10
            } else {
                0
            };
            
            for (i, card) in self.approved_cards[start_idx..].iter().enumerate() {
                let display_num = start_idx + i + 1;
                println!("║ {:>3}. {:<52} ║", display_num, card);
            }
            
            if self.approved_cards.len() > 10 {
                println!("║ ... ({} more above)                                      ║", 
                    self.approved_cards.len() - 10);
            }
            
            println!("╚══════════════════════════════════════════════════════════╝");
        }
        
        // Clear any remaining lines below
        print!("\x1B[J");
        
        io::stdout().flush().unwrap();
    }
    
    fn mask_card(card: &str) -> String {
        let parts: Vec<&str> = card.split('|').collect();
        if parts.len() >= 4 {
            let number = parts[0];
            if number.len() >= 12 {
                format!("{}...{}|{}|{}|{}",
                    &number[..6],
                    &number[number.len()-4..],
                    parts[1],
                    parts[2],
                    parts[3]
                )
            } else {
                card.to_string()
            }
        } else {
            card.to_string()
        }
    }
    
    pub fn get_summary(&self) -> String {
        let success_rate = if self.cards_processed > 0 {
            (self.approved as f64 / self.cards_processed as f64) * 100.0
        } else {
            0.0
        };
        
        format!(
            "\n{}\n\
            ✅ DISCOVERY COMPLETE\n\
            {}\n\n\
            Total Cards: {}\n\
            Approved: {} ({:.1}%)\n\
            Declined: {}\n\
            CVV Mismatch: {}\n\
            Insufficient Funds: {}\n\
            Errors: {}\n\
            \n\
            Time: {:.1}s\n\
            Speed: {:.2} cards/sec\n",
            "═".repeat(60),
            "═".repeat(60),
            self.cards_processed,
            self.approved,
            success_rate,
            self.declined,
            self.cvv_mismatch,
            self.insufficient_funds,
            self.errors,
            self.start_time.elapsed().as_secs_f64(),
            self.cards_processed as f64 / self.start_time.elapsed().as_secs_f64()
        )
    }
}
