use crate::common::{BinInfo, TelegramConfig};
use anyhow::Result;
use chrono::{DateTime, Utc};
use chrono_tz::America::New_York;
use colored::*;
use std::collections::HashMap;

/// Format the success message for Telegram (matches your Charged format)
pub fn format_success_message(
    card: &str,
    gate: &str,
    amount: f64,
    status: &str,
    bin_info: &BinInfo,
    bot_credit: &str,
) -> String {
    // Get current time in New York timezone
    let now_utc: DateTime<Utc> = Utc::now();
    let now_ny = now_utc.with_timezone(&New_York);
    let time_str = now_ny.format("%Y-%m-%d %H:%M:%S %Z").to_string();
    
    // Format the response based on status
    let (emoji, status_text, response_text) = match status {
        "CHARGED" => ("✅", "Approved", format!("${:.2} USD Charged!", amount)),
        "CVV_MISMATCH" => ("✅", "Approved", "CVV Mismatch".to_string()),
        "INSUFFICIENT_FUNDS" => ("✅", "Approved", "Insufficient Funds".to_string()),
        "DECLINED" => ("❌", "Declined", "Card Declined".to_string()),
        _ => ("⚠️", "Unknown", status.to_string()),
    };
    
    format!(
        "━━━━━━━━━━━━━━━━━━━━\n\
          Mady 𝗖𝗛𝗘𝗖𝗞𝗘𝗥 \n\
        ━━━━━━━━━━━━━━━━━━━━\n\
        \n\
        𝗖𝗖 : {}\n\
        𝗦𝘁𝗮𝘁𝘂𝘀 : {} {}\n\
        𝗥𝗲𝘀𝗽𝗼𝗻𝘀𝗲 : {}\n\
        \n\
        ━━━━━━━━━━━━━━━━━━━━\n\
           𝗖𝗔𝗥𝗗 𝗜𝗡𝗙𝗢\n\
        ━━━━━━━━━━━━━━━━━━━━\n\
        \n\
        𝗕𝗜𝗡 : {}\n\
        𝗧𝘆𝗽𝗲 : {}\n\
        𝗟𝗲𝘃𝗲𝗹 : {} 🔐\n\
        𝗕𝗮𝗻𝗸 : {}\n\
        𝗖𝗼𝘂𝗻𝘁𝗿𝘆 : {} {}\n\
        \n\
        ━━━━━━━━━━━━━━━━━━━━\n\
             𝗚𝗔𝗧𝗘 𝗜𝗡𝗙𝗢\n\
        ━━━━━━━━━━━━━━━━━━━━\n\
        \n\
        𝗚𝗮𝘁𝗲 : Shopify Authorization\n\
        𝗨𝗥𝗟 : {}\n\
        \n\
        ━━━━━━━━━━━━━━━━━━━━\n\
        𝗧𝗜𝗠𝗘 : {}\n\
        𝗕𝗼𝘁 𝗕𝘆 : <a href=\"https://t.me/{}\">{}</a>\n\
        ━━━━━━━━━━━━━━━━━━━━",
        card,
        status_text,
        emoji,
        response_text,
        bin_info.bin,
        bin_info.card_type,
        bin_info.card_level,
        bin_info.bank,
        bin_info.country,
        bin_info.country_flag,
        gate,
        time_str,
        bot_credit.trim_start_matches('@'),
        bot_credit
    )
}

/// Send message to Telegram
pub fn send_telegram_message(
    config: &TelegramConfig,
    message: &str,
) -> Result<()> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );
    
    let mut params = HashMap::new();
    params.insert("chat_id", config.group_id.as_str());
    params.insert("text", message);
    params.insert("parse_mode", "HTML");
    
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&params)
        .timeout(std::time::Duration::from_secs(10))
        .send()?;
    
    if response.status().is_success() {
        println!("{} Message sent to Telegram", "✓".green());
        Ok(())
    } else {
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Telegram API error: {}", error_text)
    }
}

/// Send success notification to Telegram
pub fn notify_success(
    config: &TelegramConfig,
    card: &str,
    gate: &str,
    amount: f64,
    status: &str,
    bin_info: &BinInfo,
) -> Result<()> {
    let message = format_success_message(
        card,
        gate,
        amount,
        status,
        bin_info,
        &config.bot_credit,
    );
    
    send_telegram_message(config, &message)
}

/// Load Telegram configuration from JSON file
pub fn load_config(path: &str) -> Result<TelegramConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: TelegramConfig = serde_json::from_str(&content)?;
    
    // Validate config
    if config.bot_token.is_empty() {
        anyhow::bail!("bot_token is empty in config file");
    }
    if config.group_id.is_empty() {
        anyhow::bail!("group_id is empty in config file");
    }
    
    Ok(config)
}
