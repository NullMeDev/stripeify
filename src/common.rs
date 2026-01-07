use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a Shopify gate/donation site
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Gate {
    pub url: String,
    #[serde(default)]
    pub gateway: String,
    #[serde(default)]
    pub donation_form: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub has_shopify: bool,
    #[serde(default)]
    pub has_shopify_payments: bool,
    #[serde(default)]
    pub payment_gateway: Option<String>,
    #[serde(default)]
    pub donation_keywords_count: usize,
}

/// Represents card data for testing
#[derive(Debug, Clone)]
pub struct CardData {
    pub number: String,
    pub month: String,
    pub year: String,
    pub cvv: String,
}

impl CardData {
    /// Parse card data from pipe-separated string
    pub fn from_string(s: &str) -> Result<Self> {
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
    
    /// Get masked version of card number for display
    pub fn masked(&self) -> String {
        if self.number.len() >= 9 {
            format!("{}...{}", &self.number[..6], &self.number[self.number.len()-3..])
        } else {
            "***".to_string()
        }
    }
}

/// Result of checking a card on a gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub gate: String,
    pub card: String,
    pub amount: f64,
    pub status: String,
    pub success: bool,
}

/// BIN lookup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinInfo {
    pub bin: String,
    pub card_type: String,      // VISA, MASTERCARD, etc.
    pub card_level: String,     // 3D, 3DS, 2D
    pub bank: String,
    pub country: String,
    pub country_flag: String,
}

impl Default for BinInfo {
    fn default() -> Self {
        BinInfo {
            bin: "UNKNOWN".to_string(),
            card_type: "UNKNOWN".to_string(),
            card_level: "UNKNOWN".to_string(),
            bank: "UNKNOWN".to_string(),
            country: "UNKNOWN".to_string(),
            country_flag: "🌍".to_string(),
        }
    }
}

/// Telegram configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub group_id: String,
    pub bot_credit: String,
}

/// Exponential backoff amounts for testing
pub const BACKOFF_AMOUNTS: [f64; 6] = [35.00, 25.00, 14.99, 4.99, 2.00, 1.00];

/// Donation keywords for URL analysis
pub const DONATION_KEYWORDS: &[&str] = &[
    "donate", "donation", "charity", "foundation", "nonprofit", "non-profit",
    "fundrais", "giving", "support", "contribute", "help", "cause",
    "relief", "aid", "mission", "humanitarian", "welfare", "benevolent"
];

/// E-commerce keywords to filter out
pub const ECOMMERCE_KEYWORDS: &[&str] = &[
    "shop", "store", "buy", "cart", "product", "clothing", "fashion",
    "apparel", "jewelry", "accessories", "shoes", "bags", "electronics"
];
