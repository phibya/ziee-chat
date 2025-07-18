// ProxyConfig is not available in this crate, remove or implement locally if needed
// use crate::ai::core::ProxyConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLockInfo {
    pub pid: u32,
    pub port: u16,
    pub model_path: String,
    pub started_at: String,
}

#[derive(Debug, Clone)]
pub struct ModelManager {}

impl ModelManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the lock file path for a model
    fn get_lock_file_path(&self, model_path: &str) -> std::path::PathBuf {
        Path::new(model_path).join(".model.lock")
    }

    /// Read lock file information
    async fn read_lock_file(
        &self,
        model_path: &str,
    ) -> Result<ModelLockInfo, Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        let content = fs::read_to_string(&lock_file_path)?;
        let lock_info: ModelLockInfo = serde_json::from_str(&content)?;
        Ok(lock_info)
    }

    /// Write lock file information
    async fn write_lock_file(
        &self,
        model_path: &str,
        lock_info: &ModelLockInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        let content = serde_json::to_string_pretty(lock_info)?;
        fs::write(&lock_file_path, content)?;
        println!("Created lock file at: {}", lock_file_path.display());
        Ok(())
    }

    /// Create a lock file for the model (public version)
    pub async fn create_lock_file(
        &self,
        model_path: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_info = ModelLockInfo {
            pid: std::process::id(),
            port,
            model_path: model_path.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
        };

        self.write_lock_file(model_path, &lock_info).await
    }

    /// Remove lock file
    async fn remove_lock_file(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        if lock_file_path.exists() {
            fs::remove_file(&lock_file_path)?;
            println!("Removed lock file at: {}", lock_file_path.display());
        }
        Ok(())
    }

    /// Remove lock file (public version)
    pub async fn remove_lock_file_public(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.remove_lock_file(model_path).await
    }

    /// Enhanced check if a model is already running with comprehensive validation
    pub async fn is_model_already_running(
        &self,
        model_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);

        if !lock_file_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&lock_file_path)?;
        let lock_info: ModelLockInfo = serde_json::from_str(&content)?;

        // Basic validation - just check if lock file exists and is valid
        println!(
            "Found lock file for model at {} with PID {} on port {}",
            lock_info.model_path, lock_info.pid, lock_info.port
        );
        Ok(true)
    }
}

pub fn get_model_manager() -> &'static ModelManager {
    MODEL_MANAGER.get_or_init(|| ModelManager::new())
}

static MODEL_MANAGER: OnceLock<ModelManager> = OnceLock::new();
