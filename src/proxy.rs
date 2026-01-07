use anyhow::{Context, Result};
use std::fs;
use std::sync::{Arc, Mutex};
use colored::*;

/// Represents a proxy configuration
#[derive(Debug, Clone)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl Proxy {
    /// Parse proxy from string format: host:port:username:password
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            anyhow::bail!("Invalid proxy format. Expected: host:port:username:password");
        }
        
        let port = parts[1].parse::<u16>()
            .context("Invalid port number")?;
        
        Ok(Proxy {
            host: parts[0].to_string(),
            port,
            username: parts[2].to_string(),
            password: parts[3].to_string(),
        })
    }
    
    /// Format proxy for ChromeDriver
    pub fn to_chromedriver_format(&self) -> String {
        format!("{}:{}@{}:{}", 
            self.username, 
            self.password, 
            self.host, 
            self.port
        )
    }
    
    /// Format proxy for HTTP authentication
    pub fn to_http_proxy_url(&self) -> String {
        format!("http://{}:{}@{}:{}", 
            self.username, 
            self.password, 
            self.host, 
            self.port
        )
    }
}

/// Proxy pool manager with rotation
pub struct ProxyPool {
    proxies: Vec<Proxy>,
    current_index: Arc<Mutex<usize>>,
    failure_counts: Arc<Mutex<Vec<usize>>>,
    max_failures: usize,
}

impl ProxyPool {
    /// Load proxies from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read proxy file: {}", path))?;
        
        let mut proxies = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            match Proxy::from_string(line) {
                Ok(proxy) => proxies.push(proxy),
                Err(e) => {
                    eprintln!("{} Line {}: {}", 
                        "⚠️  Warning:".yellow(), 
                        line_num + 1, 
                        e
                    );
                }
            }
        }
        
        if proxies.is_empty() {
            anyhow::bail!("No valid proxies found in file");
        }
        
        let failure_counts = vec![0; proxies.len()];
        
        Ok(ProxyPool {
            proxies,
            current_index: Arc::new(Mutex::new(0)),
            failure_counts: Arc::new(Mutex::new(failure_counts)),
            max_failures: 3,
        })
    }
    
    /// Get the next proxy (round-robin)
    pub fn get_next(&self) -> Option<Proxy> {
        let mut index = self.current_index.lock().unwrap();
        let failure_counts = self.failure_counts.lock().unwrap();
        
        // Find next proxy that hasn't exceeded max failures
        let start_index = *index;
        loop {
            if failure_counts[*index] < self.max_failures {
                let proxy = self.proxies[*index].clone();
                *index = (*index + 1) % self.proxies.len();
                return Some(proxy);
            }
            
            *index = (*index + 1) % self.proxies.len();
            
            // If we've checked all proxies and none are available
            if *index == start_index {
                return None;
            }
        }
    }
    
    /// Get a random proxy
    pub fn get_random(&self) -> Option<Proxy> {
        use rand::Rng;
        let failure_counts = self.failure_counts.lock().unwrap();
        
        // Get indices of proxies that haven't exceeded max failures
        let available: Vec<usize> = (0..self.proxies.len())
            .filter(|&i| failure_counts[i] < self.max_failures)
            .collect();
        
        if available.is_empty() {
            return None;
        }
        
        let mut rng = rand::thread_rng();
        let random_idx = available[rng.gen_range(0..available.len())];
        Some(self.proxies[random_idx].clone())
    }
    
    /// Report proxy failure
    pub fn report_failure(&self, proxy: &Proxy) {
        let mut failure_counts = self.failure_counts.lock().unwrap();
        
        // Find the proxy index
        for (i, p) in self.proxies.iter().enumerate() {
            if p.host == proxy.host && p.port == proxy.port && p.username == proxy.username {
                failure_counts[i] += 1;
                if failure_counts[i] >= self.max_failures {
                    println!("{} Proxy {}:{} marked as failed ({}+ failures)", 
                        "⚠️".yellow(),
                        proxy.host,
                        proxy.port,
                        self.max_failures
                    );
                }
                break;
            }
        }
    }
    
    /// Report proxy success (reset failure count)
    pub fn report_success(&self, proxy: &Proxy) {
        let mut failure_counts = self.failure_counts.lock().unwrap();
        
        // Find the proxy index and reset failure count
        for (i, p) in self.proxies.iter().enumerate() {
            if p.host == proxy.host && p.port == proxy.port && p.username == proxy.username {
                failure_counts[i] = 0;
                break;
            }
        }
    }
    
    /// Get total number of proxies
    pub fn len(&self) -> usize {
        self.proxies.len()
    }
    
    /// Get number of available proxies (not failed)
    pub fn available_count(&self) -> usize {
        let failure_counts = self.failure_counts.lock().unwrap();
        failure_counts.iter().filter(|&&count| count < self.max_failures).count()
    }
    
    /// Get statistics
    pub fn stats(&self) -> String {
        let failure_counts = self.failure_counts.lock().unwrap();
        let available = failure_counts.iter().filter(|&&count| count < self.max_failures).count();
        let failed = self.proxies.len() - available;
        
        format!("{} available, {} failed", available, failed)
    }
}
