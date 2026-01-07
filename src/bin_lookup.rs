use crate::common::BinInfo;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref BIN_CACHE: Mutex<HashMap<String, BinInfo>> = Mutex::new(HashMap::new());
}

/// Get country flag emoji from country code
fn get_country_flag(country_code: &str) -> String {
    match country_code.to_uppercase().as_str() {
        "US" => "🇺🇸",
        "CA" => "🇨🇦",
        "GB" => "🇬🇧",
        "AU" => "🇦🇺",
        "DE" => "🇩🇪",
        "FR" => "🇫🇷",
        "IT" => "🇮🇹",
        "ES" => "🇪🇸",
        "BR" => "🇧🇷",
        "MX" => "🇲🇽",
        "IN" => "🇮🇳",
        "CN" => "🇨🇳",
        "JP" => "🇯🇵",
        "KR" => "🇰🇷",
        _ => "🌍",
    }
    .to_string()
}

/// Determine card level (3D, 3DS, 2D) - simplified heuristic
fn determine_card_level(card_type: &str) -> String {
    // This is a simplified heuristic
    // In reality, you'd need more sophisticated detection
    match card_type.to_uppercase().as_str() {
        "VISA" | "MASTERCARD" => "3DS".to_string(),
        "AMEX" | "AMERICAN EXPRESS" => "3D".to_string(),
        _ => "2D".to_string(),
    }
}

/// Lookup BIN information using free API
pub fn lookup_bin(bin: &str) -> Result<BinInfo> {
    // Check cache first
    {
        let cache = BIN_CACHE.lock().unwrap();
        if let Some(info) = cache.get(bin) {
            return Ok(info.clone());
        }
    }
    
    // Try to lookup from API
    let url = format!("https://lookup.binlist.net/{}", bin);
    
    match reqwest::blocking::Client::new()
        .get(&url)
        .header("Accept-Version", "3")
        .timeout(std::time::Duration::from_secs(5))
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>() {
                    let card_type = json["scheme"]
                        .as_str()
                        .unwrap_or("UNKNOWN")
                        .to_uppercase();
                    
                    let card_subtype = json["type"]
                        .as_str()
                        .unwrap_or("")
                        .to_uppercase();
                    
                    let bank = json["bank"]["name"]
                        .as_str()
                        .unwrap_or("UNKNOWN")
                        .to_uppercase();
                    
                    let country_name = json["country"]["name"]
                        .as_str()
                        .unwrap_or("UNKNOWN")
                        .to_uppercase();
                    
                    let country_code = json["country"]["alpha2"]
                        .as_str()
                        .unwrap_or("");
                    
                    let card_level = determine_card_level(&card_type);
                    
                    let full_type = if !card_subtype.is_empty() {
                        format!("{} {}", card_type, card_subtype)
                    } else {
                        card_type.clone()
                    };
                    
                    let bin_info = BinInfo {
                        bin: bin.to_string(),
                        card_type: full_type,
                        card_level,
                        bank,
                        country: country_name,
                        country_flag: get_country_flag(country_code),
                    };
                    
                    // Cache the result
                    {
                        let mut cache = BIN_CACHE.lock().unwrap();
                        cache.insert(bin.to_string(), bin_info.clone());
                    }
                    
                    return Ok(bin_info);
                }
            }
        }
        Err(_) => {
            // API failed, return default
        }
    }
    
    // Return default if lookup failed
    let default_info = BinInfo {
        bin: bin.to_string(),
        ..Default::default()
    };
    
    // Cache the default to avoid repeated failed lookups
    {
        let mut cache = BIN_CACHE.lock().unwrap();
        cache.insert(bin.to_string(), default_info.clone());
    }
    
    Ok(default_info)
}
