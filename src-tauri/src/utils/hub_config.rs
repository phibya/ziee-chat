use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubConfig {
    pub hub_version: String,
    pub github_repo: String,
    pub github_branch: String,
    pub hub_files: Vec<String>,
    pub fallback_enabled: bool,
    pub update_check_interval_hours: u64,
}

/// Determines the hub folder path based on the environment
pub fn get_hub_folder_path() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    
    // Check if we're in development (hub folder exists in same directory)
    let dev_hub_path = exe_dir.join("hub");
    if dev_hub_path.exists() {
        return dev_hub_path;
    }
    
    // Production environment
    if cfg!(target_os = "macos") {
        // On macOS, check ../Resources/hub
        let resources_hub_path = exe_dir.join("../Resources/hub");
        if resources_hub_path.exists() {
            return resources_hub_path;
        }
    }
    
    // Fallback to same folder as binary
    exe_dir.join("hub")
}

impl HubConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Try to load from dynamic hub folder path
        let hub_folder = get_hub_folder_path();
        let config_path = hub_folder.join("hub-config.json");
        
        if config_path.exists() {
            let config_str = std::fs::read_to_string(config_path)?;
            let config: HubConfig = serde_json::from_str(&config_str)?;
            Ok(config)
        } else {
            Err(format!("Hub config file not found at: {}", config_path.display()).into())
        }
    }
}
