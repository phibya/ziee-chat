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
        let app_data_path = crate::get_app_data_dir();
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
        let app_data_path = crate::get_app_data_dir();

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
        session_id: &Uuid,
        temp_file_id: &Uuid,
        filename: &str,
        data: &[u8],
    ) -> Result<TempFile, ModelStorageError> {
        // Save to APP_DATA_DIR/temp/session_id/safe_filename
        let temp_base = crate::get_app_data_dir().join("temp");
        let session_dir = temp_base.join(session_id.to_string());

        // Ensure session temp directory exists
        if !session_dir.exists() {
            println!("Creating session temp directory: {}", session_dir.display());
            tokio::fs::create_dir_all(&session_dir).await.map_err(|e| {
                ModelStorageError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create session temp directory {}: {}",
                        session_dir.display(),
                        e
                    ),
                ))
            })?;
        }

        // Debug the input parameters
        println!("save_temp_file called with:");
        println!("  session_id: {}", session_id);
        println!("  temp_file_id: {}", temp_file_id);
        println!("  filename: '{}'", filename);

        // Sanitize filename to prevent path traversal
        let safe_filename = filename
            .replace('/', "_")
            .replace('\\', "_")
            .replace("..", "_");

        println!("  safe_filename: '{}'", safe_filename);
        println!("  session_dir: {}", session_dir.display());

        let file_path = session_dir.join(&safe_filename);
        println!("Saving temp file to: {}", file_path.display());

        tokio::fs::write(&file_path, data).await.map_err(|e| {
            ModelStorageError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write file {}: {}", file_path.display(), e),
            ))
        })?;

        // Create metadata file to map temp_file_id to original filename
        let metadata = serde_json::json!({
            "temp_file_id": temp_file_id,
            "filename": filename,
            "safe_filename": safe_filename,
            "size_bytes": data.len()
        });

        let metadata_path = session_dir.join(format!("{}.meta", temp_file_id));
        tokio::fs::write(&metadata_path, metadata.to_string())
            .await
            .map_err(|e| {
                ModelStorageError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to write metadata file {}: {}",
                        metadata_path.display(),
                        e
                    ),
                ))
            })?;

        println!(
            "Successfully saved temp file: {} ({} bytes)",
            file_path.display(),
            data.len()
        );

        Ok(TempFile {
            temp_file_id: *temp_file_id,
            filename: filename.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            size_bytes: data.len() as u64,
            is_main_file: false, // This will be set by the caller
        })
    }

    /// Commit temporary file to permanent storage
    pub async fn commit_temp_file(
        &self,
        session_id: &Uuid,
        filename: &str,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<CommittedFile, ModelStorageError> {
        let session_temp_path = crate::get_app_data_dir()
            .join("temp")
            .join(session_id.to_string());

        // Find the temp file in the session directory
        if !session_temp_path.exists() {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Session temp directory {} not found",
                session_id
            )));
        }

        // Sanitize filename the same way we did in save_temp_file
        let safe_filename = filename
            .replace('/', "_")
            .replace('\\', "_")
            .replace("..", "_");

        let temp_file_path = session_temp_path.join(&safe_filename);

        if !temp_file_path.exists() {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Temp file '{}' not found in session {}",
                safe_filename, session_id
            )));
        }

        // Create permanent storage location
        let model_path = self.get_model_path(provider_id, model_id);
        if !model_path.exists() {
            tokio::fs::create_dir_all(&model_path).await?;
        }

        let permanent_path = model_path.join(filename);

        // Move file from temp to permanent storage
        tokio::fs::rename(&temp_file_path, &permanent_path).await?;

        // Read file to calculate checksum
        let data = tokio::fs::read(&permanent_path).await?;
        // Note: Checksum calculation removed for performance

        Ok(CommittedFile {
            filename: filename.to_string(),
            file_path: Self::get_relative_path(&permanent_path)?,
            size_bytes: data.len() as u64,
        })
    }

    /// List all files in a session directory
    pub async fn list_session_files(
        &self,
        session_id: &Uuid,
    ) -> Result<Vec<String>, ModelStorageError> {
        let session_temp_path = crate::get_app_data_dir()
            .join("temp")
            .join(session_id.to_string());

        if !session_temp_path.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&session_temp_path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_type = entry.file_type().await?;
            if entry_type.is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    // Skip metadata files (.meta)
                    if !filename.ends_with(".meta") {
                        files.push(filename.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    /// List all files in a cache directory (for repository downloads)
    pub async fn list_cache_files(
        &self,
        cache_path: &str,
    ) -> Result<Vec<String>, ModelStorageError> {
        let cache_dir = crate::get_app_data_dir().join("caches").join(cache_path);

        if !cache_dir.exists() {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Cache directory not found: {}",
                cache_dir.display()
            )));
        }

        let mut files = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&cache_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_type = entry.file_type().await?;
            if entry_type.is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    // Include all files from cache directory
                    files.push(filename.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Copy a file from cache directory to permanent model storage
    pub async fn commit_cache_file(
        &self,
        cache_path: &str,
        filename: &str,
        provider_id: &Uuid,
        model_id: &Uuid,
    ) -> Result<CommittedFile, ModelStorageError> {
        let cache_dir = crate::get_app_data_dir().join("caches").join(cache_path);
        let source_file_path = cache_dir.join(filename);

        if !source_file_path.exists() {
            return Err(ModelStorageError::ModelNotFound(format!(
                "Cache file '{}' not found in {}",
                filename,
                cache_dir.display()
            )));
        }

        // Create permanent storage location
        let model_path = self.get_model_path(provider_id, model_id);
        if !model_path.exists() {
            tokio::fs::create_dir_all(&model_path).await?;
        }

        let permanent_path = model_path.join(filename);

        // Copy file from cache to permanent storage
        tokio::fs::copy(&source_file_path, &permanent_path).await?;

        // Read file to get size
        let metadata = tokio::fs::metadata(&permanent_path).await?;
        let size_bytes = metadata.len();

        Ok(CommittedFile {
            filename: filename.to_string(),
            file_path: Self::get_relative_path(&permanent_path)?,
            size_bytes,
        })
    }

    /// Clean up temporary files for a session
    pub async fn cleanup_temp_session(&self, session_id: &Uuid) -> Result<(), ModelStorageError> {
        let session_temp_path = crate::get_app_data_dir()
            .join("temp")
            .join(session_id.to_string());

        if !session_temp_path.exists() {
            return Ok(()); // Nothing to clean up
        }

        // Remove the entire session directory
        println!(
            "Cleaning up session temp directory: {}",
            session_temp_path.display()
        );
        match tokio::fs::remove_dir_all(&session_temp_path).await {
            Ok(()) => {
                println!("Successfully cleaned up session {}", session_id);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to clean up session {}: {}", session_id, e);
                Err(ModelStorageError::Io(e))
            }
        }
    }

    /// Clear all temporary files from the temp directory
    /// Called during app startup and shutdown to ensure clean state
    pub async fn clear_temp_directory() -> Result<(), ModelStorageError> {
        let temp_path = crate::get_app_data_dir().join("temp");

        if !temp_path.exists() {
            return Ok(()); // Nothing to clean up
        }

        println!("Clearing temp directory: {}", temp_path.display());

        // Remove all session directories and files in the temp directory
        let mut read_dir = tokio::fs::read_dir(&temp_path).await?;
        let mut removed_sessions = 0;
        let mut removed_files = 0;
        let mut error_count = 0;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            let entry_type = entry.file_type().await?;

            if entry_type.is_dir() {
                // Remove session directory
                match tokio::fs::remove_dir_all(&entry_path).await {
                    Ok(()) => {
                        removed_sessions += 1;
                        println!("Removed temp session directory: {}", entry_path.display());
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!(
                            "Failed to remove temp session directory {}: {}",
                            entry_path.display(),
                            e
                        );
                    }
                }
            } else {
                // Remove individual files (legacy flat structure)
                match tokio::fs::remove_file(&entry_path).await {
                    Ok(()) => {
                        removed_files += 1;
                        println!("Removed temp file: {}", entry_path.display());
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("Failed to remove temp file {}: {}", entry_path.display(), e);
                    }
                }
            }
        }

        if removed_sessions > 0 || removed_files > 0 {
            println!(
                "Temp directory cleanup complete: {} session directories and {} files removed",
                removed_sessions, removed_files
            );
        }
        if error_count > 0 {
            println!("Temp directory cleanup had {} errors", error_count);
        }

        Ok(())
    }

    /// Clean up old temp sessions that are older than the specified duration
    /// Useful for preventing disk space issues from abandoned upload sessions
    pub async fn cleanup_old_temp_sessions(max_age_hours: u64) -> Result<(), ModelStorageError> {
        let temp_path = crate::get_app_data_dir().join("temp");

        if !temp_path.exists() {
            return Ok(()); // Nothing to clean up
        }

        let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
        let now = std::time::SystemTime::now();

        println!(
            "Cleaning up temp sessions older than {} hours",
            max_age_hours
        );

        let mut read_dir = tokio::fs::read_dir(&temp_path).await?;
        let mut removed_sessions = 0;
        let mut error_count = 0;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            let entry_type = entry.file_type().await?;

            if entry_type.is_dir() {
                // Check if this is a session directory (UUID format)
                if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if uuid::Uuid::parse_str(dir_name).is_ok() {
                        // Get directory creation/modification time
                        match tokio::fs::metadata(&entry_path).await {
                            Ok(metadata) => {
                                if let Ok(created) =
                                    metadata.created().or_else(|_| metadata.modified())
                                {
                                    if let Ok(age) = now.duration_since(created) {
                                        if age > max_age {
                                            // Session is too old, remove it
                                            match tokio::fs::remove_dir_all(&entry_path).await {
                                                Ok(()) => {
                                                    removed_sessions += 1;
                                                    println!(
                                                        "Removed old temp session ({}h old): {}",
                                                        age.as_secs() / 3600,
                                                        entry_path.display()
                                                    );
                                                }
                                                Err(e) => {
                                                    error_count += 1;
                                                    eprintln!(
                                                        "Failed to remove old temp session {}: {}",
                                                        entry_path.display(),
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to get metadata for {}: {}",
                                    entry_path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        if removed_sessions > 0 {
            println!(
                "Cleanup of old temp sessions complete: {} sessions removed",
                removed_sessions
            );
        }
        if error_count > 0 {
            println!("Cleanup of old temp sessions had {} errors", error_count);
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

    /// Check if model directory exists using the same logic as local_models::ModelUtils
    pub fn model_exists(model_path: &str) -> bool {
        crate::ai::utils::models::ModelUtils::model_exists(model_path)
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
    ) -> Result<Vec<crate::ai::utils::models::ModelConfig>, std::io::Error> {
        crate::ai::utils::models::ModelDiscovery::scan_models_directory(path)
    }
}
