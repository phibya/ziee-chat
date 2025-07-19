use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub architecture: String,
    pub parameters: Option<String>,
    pub size_bytes: u64,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub files: Vec<ModelFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    pub filename: String,
    pub size_bytes: u64,
    pub file_type: ModelFileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFileType {
    ModelWeights,  // .safetensors, .bin, .pth files
    Tokenizer,     // tokenizer.json
    Config,        // config.json
    Vocabulary,    // vocab.txt, vocab.json
    Readme,        // README.md
    Other(String), // Other files
}

#[derive(Debug, Clone)]
pub struct TempFile {
    pub temp_file_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub is_main_file: bool,
}

#[derive(Debug, Clone)]
pub struct CommittedFile {
    pub filename: String,
    pub file_path: String,
    pub size_bytes: u64,
}

pub struct ModelStorage {
    base_path: PathBuf,
}

impl ModelStorage {
    pub async fn new() -> Result<Self, ModelStorageError> {
        let app_data_path = crate::APP_DATA_DIR.clone();
        let base_path = app_data_path.join("models");

        // Create models directory if it doesn't exist
        if !base_path.exists() {
            println!(
                "Creating ModelStorage base directory: {}",
                base_path.display()
            );
            tokio::fs::create_dir_all(&base_path).await.map_err(|e| {
                ModelStorageError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create base directory {}: {}",
                        base_path.display(),
                        e
                    ),
                ))
            })?;
        }

        // Create temp directory at APP_DATA_DIR level
        let temp_base = app_data_path.join("temp");
        if !temp_base.exists() {
            println!("Creating temp directory: {}", temp_base.display());
            tokio::fs::create_dir_all(&temp_base).await.map_err(|e| {
                ModelStorageError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create temp directory {}: {}",
                        temp_base.display(),
                        e
                    ),
                ))
            })?;
        }

        println!(
            "ModelStorage initialized with base path: {}",
            base_path.display()
        );
        println!("Temp directory: {}", temp_base.display());
        Ok(Self { base_path })
    }

    /// Get the storage path for a specific provider and model
    pub fn get_model_path(&self, provider_id: &Uuid, model_id: &Uuid) -> PathBuf {
        self.base_path
            .join(provider_id.to_string())
            .join(model_id.to_string())
    }

    /// Create a new model directory
    pub async fn create_model_directory(
        &self,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<PathBuf, ModelStorageError> {
        let model_path = self.get_model_path(provider_id, model_id);

        if model_path.exists() {
            return Err(ModelStorageError::ModelAlreadyExists(format!(
                "Model directory already exists: {}",
                model_path.display()
            )));
        }

        tokio::fs::create_dir_all(&model_path).await?;
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
            tokio::fs::create_dir_all(&model_path).await?;
        }

        let file_path = model_path.join(filename);
        tokio::fs::write(&file_path, data).await?;

        let file_type = Self::detect_file_type(filename);
        let size_bytes = data.len() as u64;

        Ok(ModelFile {
            filename: filename.to_string(),
            size_bytes,
            file_type,
        })
    }

    /// Load model metadata
    pub async fn load_metadata(
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

        let metadata_content = tokio::fs::read_to_string(metadata_path).await?;
        let metadata: ModelMetadata = serde_json::from_str(&metadata_content).map_err(|e| {
            ModelStorageError::InvalidModel(format!("Failed to parse metadata: {}", e))
        })?;

        Ok(metadata)
    }

    /// List all models for a provider
    pub async fn list_provider_models(
        &self,
        provider_id: &Uuid,
    ) -> Result<Vec<(Uuid, ModelMetadata)>, ModelStorageError> {
        let provider_path = self.base_path.join(provider_id.to_string());

        if !provider_path.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();

        let mut read_dir = tokio::fs::read_dir(provider_path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Ok(model_id) = entry.file_name().to_string_lossy().parse::<Uuid>() {
                    if let Ok(metadata) = self.load_metadata(provider_id, &model_id).await {
                        models.push((model_id, metadata));
                    }
                }
            }
        }

        Ok(models)
    }

    /// Get total storage size for a provider
    pub async fn get_provider_storage_size(
        &self,
        provider_id: &Uuid,
    ) -> Result<u64, ModelStorageError> {
        let provider_path = self.base_path.join(provider_id.to_string());

        if !provider_path.exists() {
            return Ok(0);
        }

        Ok(Self::calculate_directory_size(&provider_path).await?)
    }

    /// Validate model files
    pub async fn validate_model(
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
        let mut read_dir = tokio::fs::read_dir(&model_path)
            .await
            .map_err(|e| ModelStorageError::Io(e))?;

        let mut has_weights = false;
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| ModelStorageError::Io(e))?
        {
            let filename = entry.file_name().to_string_lossy().to_lowercase();
            if filename.ends_with(".safetensors")
                || filename.ends_with(".bin")
                || filename.ends_with(".pth")
            {
                has_weights = true;
                break;
            }
        }

        if !has_weights {
            issues.push("No model weight files found (.safetensors, .bin, or .pth)".to_string());
        }

        Ok(issues)
    }

    /// Detect file type based on filename
    fn detect_file_type(filename: &str) -> ModelFileType {
        let filename_lower = filename.to_lowercase();

        if filename_lower.ends_with(".safetensors")
            || filename_lower.ends_with(".bin")
            || filename_lower.ends_with(".pth")
        {
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

    // Note: Checksum calculation removed for performance

    /// Calculate directory size recursively
    async fn calculate_directory_size(path: &Path) -> Result<u64, std::io::Error> {
        let mut total_size = 0;
        let mut dirs_to_visit = vec![path.to_path_buf()];

        while let Some(current_path) = dirs_to_visit.pop() {
            let mut read_dir = tokio::fs::read_dir(&current_path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let metadata = entry.metadata().await?;

                if metadata.is_dir() {
                    dirs_to_visit.push(entry.path());
                } else {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Convert absolute file path to relative path (relative to APP_DATA_DIR)
    pub fn get_relative_path(absolute_path: &Path) -> Result<String, ModelStorageError> {
        let app_data_path = crate::APP_DATA_DIR.clone();

        match absolute_path.strip_prefix(&app_data_path) {
            Ok(relative_path) => Ok(relative_path.to_string_lossy().to_string()),
            Err(_) => {
                // If the path is not under APP_DATA_DIR, just use the filename
                Ok(absolute_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string())
            }
        }
    }

    /// Save file to temporary storage
    pub async fn save_temp_file(
        &self,
        _session_id: &Uuid, // Not needed since filenames are unique
        temp_file_id: &Uuid,
        filename: &str,
        data: &[u8],
    ) -> Result<TempFile, ModelStorageError> {
        // Save directly to APP_DATA_DIR/temp/ since filenames are unique
        let temp_base = crate::APP_DATA_DIR.join("temp");

        // Ensure temp directory exists
        if !temp_base.exists() {
            println!("Creating temp directory: {}", temp_base.display());
            tokio::fs::create_dir_all(&temp_base).await.map_err(|e| {
                ModelStorageError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create temp directory {}: {}",
                        temp_base.display(),
                        e
                    ),
                ))
            })?;
        }

        // Debug the input parameters
        println!("save_temp_file called with:");
        println!("  temp_file_id: {}", temp_file_id);
        println!("  filename: '{}'", filename);

        // Sanitize filename to prevent path traversal
        let safe_filename = filename
            .replace('/', "_")
            .replace('\\', "_")
            .replace("..", "_");

        println!("  safe_filename: '{}'", safe_filename);
        println!("  temp_base: {}", temp_base.display());

        let file_path = temp_base.join(format!("{}_{}", temp_file_id, safe_filename));
        println!("Saving temp file to: {}", file_path.display());

        tokio::fs::write(&file_path, data).await.map_err(|e| {
            ModelStorageError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write file {}: {}", file_path.display(), e),
            ))
        })?;

        println!(
            "Successfully saved temp file: {} ({} bytes)",
            file_path.display(),
            data.len()
        );

        Ok(TempFile {
            temp_file_id: *temp_file_id,
            filename: safe_filename,
            file_path: file_path.to_string_lossy().to_string(),
            size_bytes: data.len() as u64,
            is_main_file: false, // This will be set by the caller
        })
    }

    /// Commit temporary file to permanent storage
    pub async fn commit_temp_file(
        &self,
        _session_id: &Uuid, // Not needed since we search by temp_file_id
        temp_file_id: &Uuid,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<CommittedFile, ModelStorageError> {
        let temp_path = crate::APP_DATA_DIR.join("temp");

        // Find the temp file
        let mut read_dir = tokio::fs::read_dir(&temp_path).await?;
        let mut temp_file_path = None;
        let mut original_filename = None;

        while let Some(entry) = read_dir.next_entry().await? {
            let file_name = entry.file_name().to_string_lossy().to_string();

            if file_name.starts_with(&format!("{}_", temp_file_id)) {
                temp_file_path = Some(entry.path());
                original_filename =
                    Some(file_name.split('_').skip(1).collect::<Vec<_>>().join("_"));
                break;
            }
        }

        let temp_file_path = temp_file_path.ok_or_else(|| {
            ModelStorageError::ModelNotFound(format!("Temp file {} not found", temp_file_id))
        })?;

        let filename = original_filename.ok_or_else(|| {
            ModelStorageError::InvalidModel("Could not extract filename from temp file".to_string())
        })?;

        // Create permanent storage location
        let model_path = self.get_model_path(provider_id, model_id);
        if !model_path.exists() {
            tokio::fs::create_dir_all(&model_path).await?;
        }

        let permanent_path = model_path.join(&filename);

        // Move file from temp to permanent storage
        tokio::fs::rename(&temp_file_path, &permanent_path).await?;

        // Read file to calculate checksum
        let data = tokio::fs::read(&permanent_path).await?;
        // Note: Checksum calculation removed for performance

        Ok(CommittedFile {
            filename,
            file_path: Self::get_relative_path(&permanent_path)?,
            size_bytes: data.len() as u64,
        })
    }

    /// Clean up temporary files for a session
    pub async fn cleanup_temp_session(&self, _session_id: &Uuid) -> Result<(), ModelStorageError> {
        let temp_path = crate::APP_DATA_DIR.join("temp");

        if !temp_path.exists() {
            return Ok(()); // Nothing to clean up
        }

        // Find and delete files that belong to this session
        // Since we don't track session->file mapping, we'll need to implement
        // a different cleanup strategy or track session files differently
        println!("Note: Session-based cleanup not implemented with flat temp structure");
        println!("Consider implementing periodic cleanup of old temp files instead");

        Ok(())
    }

    /// Clear all temporary files from the temp directory
    /// Called during app startup and shutdown to ensure clean state
    pub async fn clear_temp_directory() -> Result<(), ModelStorageError> {
        let temp_path = crate::APP_DATA_DIR.join("temp");

        if !temp_path.exists() {
            return Ok(()); // Nothing to clean up
        }

        println!("Clearing temp directory: {}", temp_path.display());

        // Remove all files in the temp directory
        let mut read_dir = tokio::fs::read_dir(&temp_path).await?;
        let mut removed_count = 0;
        let mut error_count = 0;

        while let Some(entry) = read_dir.next_entry().await? {
            let file_path = entry.path();
            match tokio::fs::remove_file(&file_path).await {
                Ok(()) => {
                    removed_count += 1;
                    println!("Removed temp file: {}", file_path.display());
                }
                Err(e) => {
                    error_count += 1;
                    eprintln!("Failed to remove temp file {}: {}", file_path.display(), e);
                }
            }
        }

        if removed_count > 0 {
            println!(
                "Temp directory cleanup complete: {} files removed",
                removed_count
            );
        }
        if error_count > 0 {
            println!("Temp directory cleanup had {} errors", error_count);
        }

        Ok(())
    }
}

/// Utility functions for model management
pub struct ModelUtils;

impl ModelUtils {
    /// Extract model information from config.json
    pub fn extract_model_info(
        config_content: &str,
    ) -> Result<(String, Option<String>), ModelStorageError> {
        let config: serde_json::Value = serde_json::from_str(config_content)
            .map_err(|e| ModelStorageError::InvalidModel(format!("Invalid config.json: {}", e)))?;

        let architecture = config
            .get("architectures")
            .and_then(|a| a.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let model_type = config
            .get("model_type")
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
            return Err(ModelStorageError::InvalidModel(
                "Model name cannot be empty".to_string(),
            ));
        }

        if name.len() > 255 {
            return Err(ModelStorageError::InvalidModel(
                "Model name too long (max 255 characters)".to_string(),
            ));
        }

        // Check for invalid characters
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        if name.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(ModelStorageError::InvalidModel(
                "Model name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if model directory exists using the same logic as candle_models::ModelUtils
    pub fn model_exists(model_path: &str) -> bool {
        crate::ai::models::ModelUtils::model_exists(model_path)
    }

    /// Verify model exists or return ModelNotFound error
    pub fn verify_model_exists(
        model_path: &str,
        model_name: &str,
    ) -> Result<(), ModelStorageError> {
        if !Self::model_exists(model_path) {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Model '{}' not found at path: {}",
                model_name, model_path
            )));
        }
        Ok(())
    }

    /// Discover available models in a directory using ModelDiscovery
    pub fn discover_models(
        path: &str,
    ) -> Result<Vec<crate::ai::models::ModelConfig>, std::io::Error> {
        crate::ai::models::ModelDiscovery::scan_models_directory(path)
    }
}
