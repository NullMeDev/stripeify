use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::proxy::Proxy;

/// Creates a Chrome extension for authenticated proxy support
pub struct ProxyExtension {
    _temp_dir: TempDir,
    extension_path: PathBuf,
}

impl ProxyExtension {
    /// Create a new proxy extension for the given proxy
    pub fn new(proxy: &Proxy) -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let ext_path = temp_dir.path().to_path_buf();
        
        // Create manifest.json
        let manifest = json!({
            "version": "1.0.0",
            "manifest_version": 2,
            "name": "Chrome Proxy Auth",
            "permissions": [
                "proxy",
                "tabs",
                "unlimitedStorage",
                "storage",
                "<all_urls>",
                "webRequest",
                "webRequestBlocking"
            ],
            "background": {
                "scripts": ["background.js"]
            },
            "minimum_chrome_version": "22.0.0"
        });
        
        fs::write(
            ext_path.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?
        )?;
        
        // Create background.js with proxy configuration
        let background_js = format!(r#"
var config = {{
    mode: "fixed_servers",
    rules: {{
        singleProxy: {{
            scheme: "http",
            host: "{}",
            port: parseInt({})
        }},
        bypassList: ["localhost", "127.0.0.1"]
    }}
}};

chrome.proxy.settings.set({{value: config, scope: "regular"}}, function() {{
    console.log("Proxy configured: {}:{}");
}});

function callbackFn(details) {{
    return {{
        authCredentials: {{
            username: "{}",
            password: "{}"
        }}
    }};
}}

chrome.webRequest.onAuthRequired.addListener(
    callbackFn,
    {{urls: ["<all_urls>"]}},
    ['blocking']
);

console.log("Proxy extension loaded successfully");
"#, 
            proxy.host, 
            proxy.port, 
            proxy.host, 
            proxy.port,
            proxy.username, 
            proxy.password
        );
        
        fs::write(ext_path.join("background.js"), background_js)?;
        
        Ok(ProxyExtension {
            _temp_dir: temp_dir,
            extension_path: ext_path,
        })
    }
    
    /// Get the path to the extension directory
    pub fn path(&self) -> &PathBuf {
        &self.extension_path
    }
    
    /// Get the path as a string
    pub fn path_str(&self) -> String {
        self.extension_path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_extension() {
        let proxy = Proxy {
            host: "proxy.example.com".to_string(),
            port: 8080,
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        
        let ext = ProxyExtension::new(&proxy).unwrap();
        
        // Check that manifest.json exists
        assert!(ext.path().join("manifest.json").exists());
        
        // Check that background.js exists
        assert!(ext.path().join("background.js").exists());
        
        // Read and verify manifest
        let manifest_content = fs::read_to_string(ext.path().join("manifest.json")).unwrap();
        assert!(manifest_content.contains("Chrome Proxy Auth"));
        
        // Read and verify background.js
        let background_content = fs::read_to_string(ext.path().join("background.js")).unwrap();
        assert!(background_content.contains("proxy.example.com"));
        assert!(background_content.contains("8080"));
        assert!(background_content.contains("user"));
        assert!(background_content.contains("pass"));
    }
}
