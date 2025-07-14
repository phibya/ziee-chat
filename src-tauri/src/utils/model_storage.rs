use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ModelStorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid model file: {0}")]
    InvalidModel(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Model already exists: {0}")]
    ModelAlreadyExists(String),
    #[error("Environment error: {0}")]
    Environment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub architecture: String,
    pub parameters: Option<String>,
    pub quantization: Option<String>,
    pub size_bytes: u64,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub files: Vec<ModelFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    pub filename: String,
    pub size_bytes: u64,
    pub file_type: ModelFileType,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFileType {
    ModelWeights,     // .safetensors, .bin, .pth files
    Tokenizer,        // tokenizer.json
    Config,           // config.json
    Vocabulary,       // vocab.txt, vocab.json
    Readme,           // README.md
    Other(String),    // Other files
}

pub struct ModelStorage {
    base_path: PathBuf,
}

impl ModelStorage {
    pub fn new() -> Result<Self, ModelStorageError> {
        let app_data_dir = std::env::var("APP_DATA_DIR")
            .map_err(|_| ModelStorageError::Environment("APP_DATA_DIR not set".to_string()))?;
        
        let base_path = PathBuf::from(app_data_dir).join("models").join("candle");
        
        // Create base directory if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;
        }
        
        Ok(Self { base_path })
    }

    /// Get the storage path for a specific provider and model
    pub fn get_model_path(&self, provider_id: &Uuid, model_id: &Uuid) -> PathBuf {
        self.base_path
            .join(provider_id.to_string())
            .join(model_id.to_string())
    }

    /// Create a new model directory
    pub fn create_model_directory(&self, provider_id: &Uuid, model_id: &Uuid) -> Result<PathBuf, ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        
        if model_path.exists() {
            return Err(ModelStorageError::ModelAlreadyExists(format!(
                "Model directory already exists: {}", 
                model_path.display()
            )));
        }
        
        fs::create_dir_all(&model_path)?;
        Ok(model_path)
    }

    /// Save a model file to the storage
    pub async fn save_model_file(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
        filename: &str,
        data: &[u8],
    ) -> Result<ModelFile, ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        
        if !model_path.exists() {
            fs::create_dir_all(&model_path)?;
        }
        
        let file_path = model_path.join(filename);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        
        let file_type = Self::detect_file_type(filename);
        let size_bytes = data.len() as u64;
        
        Ok(ModelFile {
            filename: filename.to_string(),
            size_bytes,
            file_type,
            checksum: Some(Self::calculate_checksum(data)),
        })
    }

    /// Save model metadata
    pub fn save_metadata(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
        metadata: &ModelMetadata,
    ) -> Result<(), ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        let metadata_path = model_path.join("metadata.json");
        
        let metadata_json = serde_json::to_string_pretty(metadata)
            .map_err(|e| ModelStorageError::InvalidModel(format!("Failed to serialize metadata: {}", e)))?;
        
        fs::write(metadata_path, metadata_json)?;
        Ok(())
    }

    /// Load model metadata
    pub fn load_metadata(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<ModelMetadata, ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        let metadata_path = model_path.join("metadata.json");
        
        if !metadata_path.exists() {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Metadata not found for model {} in provider {}", 
                model_id, provider_id
            )));
        }
        
        let metadata_content = fs::read_to_string(metadata_path)?;
        let metadata: ModelMetadata = serde_json::from_str(&metadata_content)
            .map_err(|e| ModelStorageError::InvalidModel(format!("Failed to parse metadata: {}", e)))?;
        
        Ok(metadata)
    }

    /// Delete a model and all its files
    pub fn delete_model(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<(), ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        
        if model_path.exists() {
            fs::remove_dir_all(model_path)?;
        }
        
        Ok(())
    }

    /// List all models for a provider
    pub fn list_provider_models(
        &self,
        provider_id: &Uuid,
    ) -> Result<Vec<(Uuid, ModelMetadata)>, ModelStorageError> {
        let provider_path = self.base_path.join(provider_id.to_string());
        
        if !provider_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut models = Vec::new();
        
        for entry in fs::read_dir(provider_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(model_id) = entry.file_name().to_string_lossy().parse::<Uuid>() {
                    if let Ok(metadata) = self.load_metadata(provider_id, &model_id) {
                        models.push((model_id, metadata));
                    }
                }
            }
        }
        
        Ok(models)
    }

    /// Get total storage size for a provider
    pub fn get_provider_storage_size(
        &self,
        provider_id: &Uuid,
    ) -> Result<u64, ModelStorageError> {
        let provider_path = self.base_path.join(provider_id.to_string());
        
        if !provider_path.exists() {
            return Ok(0);
        }
        
        Ok(Self::calculate_directory_size(&provider_path)?)
    }

    /// Validate model files
    pub fn validate_model(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<Vec<String>, ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);
        let mut issues = Vec::new();
        
        if !model_path.exists() {
            issues.push("Model directory does not exist".to_string());
            return Ok(issues);
        }
        
        // Check for required files
        let tokenizer_path = model_path.join("tokenizer.json");
        if !tokenizer_path.exists() {
            issues.push("Missing tokenizer.json file".to_string());
        }
        
        let config_path = model_path.join("config.json");
        if !config_path.exists() {
            issues.push("Missing config.json file".to_string());
        }
        
        // Check for model weight files
        let has_weights = model_path.read_dir()
            .map_err(|e| ModelStorageError::Io(e))?
            .any(|entry| {
                if let Ok(entry) = entry {
                    let filename = entry.file_name().to_string_lossy().to_lowercase();
                    filename.ends_with(".safetensors") || 
                    filename.ends_with(".bin") || 
                    filename.ends_with(".pth")
                } else {
                    false
                }
            });
        
        if !has_weights {
            issues.push("No model weight files found (.safetensors, .bin, or .pth)".to_string());
        }
        
        Ok(issues)
    }

    /// Detect file type based on filename
    fn detect_file_type(filename: &str) -> ModelFileType {
        let filename_lower = filename.to_lowercase();
        
        if filename_lower.ends_with(".safetensors") || 
           filename_lower.ends_with(".bin") || 
           filename_lower.ends_with(".pth") {
            ModelFileType::ModelWeights
        } else if filename_lower == "tokenizer.json" {
            ModelFileType::Tokenizer
        } else if filename_lower == "config.json" {
            ModelFileType::Config
        } else if filename_lower.starts_with("vocab.") {
            ModelFileType::Vocabulary
        } else if filename_lower == "readme.md" {
            ModelFileType::Readme
        } else {
            ModelFileType::Other(filename.to_string())
        }
    }

    /// Calculate SHA256 checksum
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Calculate directory size recursively
    fn calculate_directory_size(path: &Path) -> Result<u64, std::io::Error> {
        let mut total_size = 0;
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            if metadata.is_dir() {
                total_size += Self::calculate_directory_size(&entry.path())?;
            } else {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }
}

/// Utility functions for model management
pub struct ModelUtils;

impl ModelUtils {
    /// Extract model information from config.json
    pub fn extract_model_info(config_content: &str) -> Result<(String, Option<String>), ModelStorageError> {
        let config: serde_json::Value = serde_json::from_str(config_content)
            .map_err(|e| ModelStorageError::InvalidModel(format!("Invalid config.json: {}", e)))?;
        
        let architecture = config.get("architectures")
            .and_then(|a| a.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();
        
        let model_type = config.get("model_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok((architecture, model_type))
    }

    /// Format model size for display
    pub fn format_model_size(size_bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size_bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size_bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Validate model name
    pub fn validate_model_name(name: &str) -> Result<(), ModelStorageError> {
        if name.is_empty() {
            return Err(ModelStorageError::InvalidModel("Model name cannot be empty".to_string()));
        }
        
        if name.len() > 255 {
            return Err(ModelStorageError::InvalidModel("Model name too long (max 255 characters)".to_string()));
        }
        
        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(ModelStorageError::InvalidModel("Model name contains invalid characters".to_string()));
        }
        
        Ok(())
    }
    
    /// Check if model directory exists using the same logic as candle_models::ModelUtils
    pub fn model_exists(model_path: &str) -> bool {
        crate::ai::candle_models::ModelUtils::model_exists(model_path)
    }
    
    /// Verify model exists or return ModelNotFound error
    pub fn verify_model_exists(model_path: &str, model_name: &str) -> Result<(), ModelStorageError> {
        if !Self::model_exists(model_path) {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Model '{}' not found at path: {}", 
                model_name, model_path
            )));
        }
        Ok(())
    }
    
    /// Discover available models in a directory using ModelDiscovery
    pub fn discover_models(path: &str) -> Result<Vec<crate::ai::candle_config::ModelConfig>, std::io::Error> {
        crate::ai::candle_config::ModelDiscovery::scan_models_directory(path)
    }
}