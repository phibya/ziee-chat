use super::resource_paths::ResourcePaths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HubConfig {
    pub hub_version: String,
    pub github_repo: String,
    pub github_branch: String,
    pub hub_files: Vec<String>,
    pub i18n_supported_languages: Vec<String>,
    pub i18n_files: Vec<String>,
    pub fallback_enabled: bool,
    pub update_check_interval_hours: u64,
}

/// Determines the hub folder path based on the environment
/// Uses the ResourcePaths utility for platform-specific path resolution
pub fn get_hub_folder_path() -> PathBuf {
    let hub_path = ResourcePaths::get_hub_folder();

    // Log the path for debugging
    if hub_path.exists() {
        println!("Found hub folder at: {}", hub_path.display());
    } else {
        println!(
            "Warning: Using hub path at: {} (directory does not exist yet)",
            hub_path.display()
        );
    }

    hub_path
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
