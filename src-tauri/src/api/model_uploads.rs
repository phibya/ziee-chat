use axum::{
    extract::{Multipart, Path, Query},
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError, ErrorCode},
    middleware::AuthenticatedUser,
};
use crate::database::{get_database_pool, model_operations::ModelOperations, models::*};

#[derive(Deserialize)]
pub struct UpdateModelStatusRequest {
    enabled: Option<bool>,
    is_active: Option<bool>,
}
use crate::utils::model_storage::ModelStorage;

#[derive(Deserialize)]
pub struct UploadFileRequest {
    filename: String,
    file_size: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct UploadFilesResponse {
    pub session_id: Uuid,
    pub files: Vec<ProcessedFile>,
    pub total_size_bytes: u64,
    pub main_filename: String,
    pub provider_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessedFile {
    pub temp_file_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub validation_issues: Vec<String>,
    pub is_main_file: bool,
}

#[derive(Deserialize)]
pub struct CommitUploadRequest {
    pub session_id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub architecture: String,
    pub file_format: String,
    pub selected_files: Vec<Uuid>, // temp_file_ids to commit
}

// create_model function removed - use upload_model_file_multipart and commit_uploaded_files workflow instead

/// Upload model file
pub async fn upload_model_file(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UploadFileRequest>,
) -> ApiResult<Json<ModelUploadResponse>> {
    let pool = get_database_pool()?;

    // Get model info
    let model = ModelOperations::get_model_by_id(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;

    // Initialize storage
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    // Get the model directory path using {provider_id}/{id} pattern
    let model_path = format!("models/{}/{}", model.provider_id, model_id);
    let file_path = crate::APP_DATA_DIR
        .join(&model_path)
        .join(&request.filename);

    // For local folder uploads, we expect the files to already be accessible
    // This is a simplified implementation - in production you'd handle actual file uploads
    println!(
        "Processing upload for file: {} ({}bytes)",
        request.filename, request.file_size
    );

    // Calculate checksum if file exists
    if file_path.exists() {
        match std::fs::read(&file_path) {
            Ok(file_data) => {
                // Save the file through ModelStorage which will calculate checksum
                match storage
                    .save_model_file(&model.provider_id, &model_id, &request.filename, &file_data)
                    .await
                {
                    Ok(model_file) => {
                        // Note: Checksum calculation removed for performance
                    }
                    Err(e) => {
                        eprintln!("Failed to save model file: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read file for checksum: {}", e);
            }
        }
    }

    // Update model status to indicate file processing
    ModelOperations::update_model_validation(
        pool.as_ref(),
        &model_id,
        "processing",
        None,
        Some(request.file_size as i64),
    )
    .await
    .map_err(AppError::database_error)?;

    let response = ModelUploadResponse {
        model_id,
        upload_url: None,
        chunk_uploaded: true,
        upload_complete: true,
        next_chunk_index: None,
    };

    Ok(Json(response))
}

/// Upload multiple model files in a single multipart request
pub async fn upload_model_file_multipart(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> ApiResult<Json<UploadFilesResponse>> {
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    let mut uploaded_files = Vec::new();
    let mut main_filename: Option<String> = None;
    let mut provider_id: Option<Uuid> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "files" => {
                // Get filename from the field
                if let Some(file_name) = field.file_name() {
                    // Extract just the filename, not the full path
                    let filename = std::path::Path::new(file_name)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(file_name)
                        .to_string();

                    println!(
                        "Original file_name: '{}', extracted filename: '{}'",
                        file_name, filename
                    );

                    // Read file data
                    let data = field.bytes().await.map_err(|e| {
                        AppError::new(
                            ErrorCode::ValidInvalidInput,
                            format!("Failed to read file data: {}", e),
                        )
                    })?;

                    uploaded_files.push((filename, data.to_vec()));
                }
            }
            "main_filename" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read main_filename: {}", e),
                    )
                })?;
                main_filename = Some(value);
            }
            "provider_id" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read provider_id: {}", e),
                    )
                })?;
                provider_id = Some(Uuid::parse_str(&value).map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Invalid provider_id format: {}", e),
                    )
                })?);
            }
            _ => {
                // Skip unknown fields
                continue;
            }
        }
    }

    // Validate required fields
    if uploaded_files.is_empty() {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "No files provided in multipart request",
        ));
    }

    let provider_id = provider_id.ok_or_else(|| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            "Missing provider_id in multipart request",
        )
    })?;

    let main_filename = main_filename.ok_or_else(|| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            "Missing main_filename in multipart request",
        )
    })?;

    println!(
        "Processing multipart upload: {} files, main file: {}",
        uploaded_files.len(),
        main_filename
    );

    // Step 1: Upload files to temporary storage
    let temp_session_id = Uuid::new_v4();
    let mut processed_files = Vec::new();
    let mut total_size = 0u64;

    for (filename, file_data) in uploaded_files {
        total_size += file_data.len() as u64;

        // Step 2: Check and validate files
        let file_type = determine_model_file_type(&filename);
        let validation_issues = validate_file_content(&filename, &file_data);

        // Step 3: Save files to temporary storage
        let temp_file_id = Uuid::new_v4();
        match storage
            .save_temp_file(&temp_session_id, &temp_file_id, &filename, &file_data)
            .await
        {
            Ok(temp_file) => {
                processed_files.push(ProcessedFile {
                    temp_file_id,
                    filename: filename.clone(),
                    file_type: file_type.to_string(),
                    size_bytes: file_data.len() as u64,
                    validation_issues,
                    is_main_file: filename == main_filename,
                });

                println!("Saved temp file: {} (ID: {})", filename, temp_file_id);
            }
            Err(e) => {
                println!("Failed to save temp file {}: {}", filename, e);
                return Err(AppError::internal_error(format!(
                    "Failed to save file {}: {}",
                    filename, e
                )));
            }
        }
    }

    // Step 4: Return file IDs and metadata to client
    let response = UploadFilesResponse {
        session_id: temp_session_id,
        files: processed_files,
        total_size_bytes: total_size,
        main_filename,
        provider_id,
    };

    Ok(Json(response))
}

/// Commit uploaded files as a model
pub async fn commit_uploaded_files(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CommitUploadRequest>,
) -> ApiResult<Json<Model>> {
    let pool = get_database_pool()?;
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    // Validate provider exists and is of type 'candle'
    let provider = crate::database::queries::providers::get_provider_by_id(request.provider_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::new(ErrorCode::ValidInvalidInput, "Provider not found"))?;

    if provider.provider_type != "candle" {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Only Candle providers support model uploads",
        ));
    }

    // Create the model record
    let model_id = Uuid::new_v4();
    let model_path = storage.get_model_path(&request.provider_id, &model_id);

    // Convert absolute path to relative path for database storage
    let _relative_model_path = match ModelStorage::to_relative_path(&model_path) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Warning: Failed to convert model path to relative: {}", e);
            // Fallback to just the directory name
            format!("models/{}/{}", request.provider_id, model_id)
        }
    };

    // Create model request
    let model_name = request.name.clone();
    let architecture = request.architecture.clone();
    let file_format = request.file_format.clone();
    let create_request = crate::database::models::CreateModelRequest {
        provider_id: request.provider_id,
        name: request.name,
        alias: request.alias,
        description: request.description,
        enabled: Some(false),
        capabilities: Some(serde_json::json!({})),
        settings: None,
    };

    println!("Processing model with file format: {}", file_format);

    let model_db = ModelOperations::create_candle_model(pool.as_ref(), &create_request, &architecture)
    .await
    .map_err(|e| {
      // Handle unique constraint violation for (provider_id, name)
      match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("models_provider_id_name_unique") => {
          AppError::new(ErrorCode::ValidInvalidInput,
                        format!("Model ID '{}' already exists for this provider. Please use a different model ID.", model_name))
        }
        _ => AppError::database_error(e)
      }
    })?;

    // Create storage directory
    storage
        .create_model_directory(&request.provider_id, &model_db.id)
        .await
        .map_err(|e| {
            AppError::internal_error(format!("Failed to create storage directory: {}", e))
        })?;

    // Move temp files to permanent storage and create file records
    let mut total_size = 0u64;
    // Note: Checksum calculation removed for performance

    for temp_file_id in &request.selected_files {
        match storage
            .commit_temp_file(
                &request.session_id,
                temp_file_id,
                &request.provider_id,
                &model_db.id,
            )
            .await
        {
            Ok(committed_file) => {
                total_size += committed_file.size_bytes;

                // Create model file record
                let file_type = determine_model_file_type(&committed_file.filename).to_string();
                let file_type_str = file_type.as_str();

                ModelOperations::create_model_file(
                    pool.as_ref(),
                    &model_db.id,
                    &committed_file.filename,
                    &committed_file.file_path,
                    committed_file.size_bytes as i64,
                    file_type_str,
                )
                .await
                .map_err(AppError::database_error)?;

                // Note: Checksum calculation removed for performance

                println!(
                    "Committed file: {} -> {}",
                    committed_file.filename, committed_file.file_path
                );
            }
            Err(e) => {
                println!("Failed to commit temp file {}: {}", temp_file_id, e);
                return Err(AppError::internal_error(format!(
                    "Failed to commit file: {}",
                    e
                )));
            }
        }
    }

    // Update model with total size (checksum removed for performance)

    // Update validation status to completed and enable the model
    ModelOperations::update_model_validation(
        pool.as_ref(),
        &model_db.id,
        "completed",
        None,
        Some(total_size as i64),
    )
    .await
    .map_err(AppError::database_error)?;

    ModelOperations::update_model_status(
        pool.as_ref(),
        &model_db.id,
        Some(true), // enabled = true
        None,       // don't change is_active
    )
    .await
    .map_err(AppError::database_error)?;

    // Clean up temp files
    if let Err(e) = storage.cleanup_temp_session(&request.session_id).await {
        println!(
            "Warning: Failed to cleanup temp session {}: {}",
            request.session_id, e
        );
    }

    // Return the created model
    let model = ModelOperations::get_model_with_files(pool.as_ref(), &model_db.id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;

    Ok(Json(model))
}

/// Determine model file type based on filename
fn determine_model_file_type(filename: &str) -> ModelFileType {
    let filename_lower = filename.to_lowercase();

    // Weight files (actual model parameters)
    if filename_lower.ends_with(".bin")
        || filename_lower.ends_with(".pt")
        || filename_lower.ends_with(".pth")
        || filename_lower.ends_with(".safetensors")
        || filename_lower.ends_with(".gguf")
        || filename_lower.ends_with(".ggml")
    {
        return ModelFileType::WeightFile;
    }

    // Configuration files
    if filename_lower == "config.json"
        || filename_lower.starts_with("config_")
        || filename_lower == "generation_config.json"
    {
        return ModelFileType::ConfigFile;
    }

    // Tokenizer files
    if filename_lower == "tokenizer.json"
        || filename_lower == "tokenizer_config.json"
        || filename_lower.starts_with("tokenizer_")
    {
        return ModelFileType::TokenizerFile;
    }

    // Vocabulary and token files
    if filename_lower == "vocab.json"
        || filename_lower == "merges.txt"
        || filename_lower == "special_tokens_map.json"
        || filename_lower == "vocab.txt"
        || filename_lower == "spiece.model"
    {
        return ModelFileType::VocabFile;
    }

    ModelFileType::UnknownFile
}

/// Validate file content and return any issues
fn validate_file_content(filename: &str, file_data: &[u8]) -> Vec<String> {
    let mut issues = Vec::new();

    if file_data.is_empty() {
        issues.push("File is empty".to_string());
        return issues;
    }

    let filename_lower = filename.to_lowercase();
    let file_type = determine_model_file_type(&filename_lower);

    match file_type {
        ModelFileType::WeightFile => {
            if file_data.len() < 1024 {
                issues.push("Model weight file is suspiciously small (< 1KB)".to_string());
            }
        }
        ModelFileType::ConfigFile => {
            // Try to parse as JSON
            if let Err(_) = serde_json::from_slice::<serde_json::Value>(file_data) {
                issues.push("Config file is not valid JSON".to_string());
            }
        }
        ModelFileType::TokenizerFile => {
            if filename_lower == "tokenizer.json" {
                if let Err(_) = serde_json::from_slice::<serde_json::Value>(file_data) {
                    issues.push("Tokenizer file is not valid JSON".to_string());
                }
            }
        }
        _ => {
            // Basic validation for other files
        }
    }

    // Check for HTML content (error pages)
    if file_data.len() >= 4 {
        let first_4_bytes = &file_data[0..4];
        if matches!(
            first_4_bytes,
            [0x3C, 0x21, _, _] | [0x3C, 0x68, 0x74, 0x6D] | [0x3C, 0x48, 0x54, 0x4D]
        ) {
            issues.push("File appears to be HTML content (possibly an error page)".to_string());
        }
    }

    issues
}

/// Validate model
pub async fn validate_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<Json<ModelValidationResult>> {
    let pool = get_database_pool()?;

    // Get model and files
    let model = ModelOperations::get_model_by_id(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;

    let _files = ModelOperations::get_model_files(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?;

    // Initialize storage
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    // Validate model using both storage and utils
    let mut validation_issues = storage
        .validate_model(&model.provider_id, &model_id)
        .await
        .map_err(|e| AppError::internal_error(format!("Validation failed: {}", e)))?;

    // Additional validation using ModelUtils
    let model_path = format!("models/{}/{}", model.provider_id, model_id);

    // Validate model name
    if let Err(e) = crate::utils::model_storage::ModelUtils::validate_model_name(&model.name) {
        validation_issues.push(format!("Invalid model name: {}", e));
    }

    // Check if model exists using verification function
    if let Err(e) =
        crate::utils::model_storage::ModelUtils::verify_model_exists(&model_path, &model.name)
    {
        validation_issues.push(format!("Model verification failed: {}", e));
    }

    // Get and validate model size
    if let Ok(model_size) =
        crate::ai::candle_server::models::ModelUtils::get_model_size(&model_path)
    {
        let formatted_size = crate::utils::model_storage::ModelUtils::format_model_size(model_size);
        println!("Model size: {}", formatted_size);

        // Only warn if the total model weight files are extremely small
        // Count only actual weight files, not config/tokenizer files
        let model_path_buf = crate::APP_DATA_DIR.join(&model_path);
        if let Ok(mut read_dir) = tokio::fs::read_dir(&model_path_buf).await {
            let mut weight_files_size: u64 = 0;

            while let Ok(Some(entry)) = read_dir.next_entry().await {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let file_type = determine_file_type(&file_name.to_lowercase());
                    if matches!(file_type, ModelFileType::WeightFile) {
                        if let Ok(metadata) = entry.metadata().await {
                            weight_files_size += metadata.len();
                        }
                    }
                }
            }

            if weight_files_size > 0 && weight_files_size < 100 * 1024 {
                // Less than 100KB of weight files
                validation_issues
                    .push("Model weight files appear to be very small (< 100KB)".to_string());
            }
        }
    }

    // Extract and validate model info from config.json if present
    let config_path = crate::APP_DATA_DIR.join(&model_path).join("config.json");
    if config_path.exists() {
        if let Ok(config_content) = tokio::fs::read_to_string(&config_path).await {
            match crate::utils::model_storage::ModelUtils::extract_model_info(&config_content) {
                Ok((extracted_name, _description)) => {
                    println!("Extracted model info: name={}", extracted_name);
                }
                Err(e) => {
                    validation_issues.push(format!("Invalid config.json: {}", e));
                }
            }
        }
    }

    // Discover and validate model using ModelDiscovery
    match crate::utils::model_storage::ModelUtils::discover_models(&model_path) {
        Ok(discovered_models) => {
            println!("Discovered {} models in directory", discovered_models.len());
            if discovered_models.is_empty() {
                validation_issues.push("No valid models found in directory".to_string());
            }
        }
        Err(e) => {
            validation_issues.push(format!("Model discovery failed: {}", e));
        }
    }

    // List available models in the directory
    if let Ok(model_list) = crate::ai::candle_server::models::ModelUtils::list_models(&model_path) {
        println!("Available models: {:?}", model_list);
        if model_list.is_empty() {
            validation_issues.push("No model directories found".to_string());
        }
    }

    let is_valid = validation_issues.is_empty();
    let validation_status = if is_valid { "valid" } else { "invalid" };

    // Update validation status in database
    ModelOperations::update_model_validation(
        pool.as_ref(),
        &model_id,
        validation_status,
        Some(&validation_issues),
        None,
    )
    .await
    .map_err(AppError::database_error)?;

    // Create validation result
    let validation_result = ModelValidationResult {
        is_valid,
        issues: validation_issues.clone(),
        required_files: vec!["tokenizer.json".to_string(), "config.json".to_string()],
        present_files: vec![], // Would be populated in a real implementation
    };

    Ok(Json(validation_result))
}

/// Update model status
pub async fn update_model_status(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UpdateModelStatusRequest>,
) -> ApiResult<Json<Model>> {
    let pool = get_database_pool()?;

    // Update model status
    ModelOperations::update_model_status(
        pool.as_ref(),
        &model_id,
        request.enabled,
        request.is_active,
    )
    .await
    .map_err(AppError::database_error)?;

    // Return updated model
    let model = ModelOperations::get_model_with_files(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;

    Ok(Json(model))
}

/// Get storage statistics
pub async fn get_storage_stats(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<ModelStorageInfo>> {
    let pool = get_database_pool()?;

    let provider_id = params
        .get("provider_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::ValidInvalidInput,
                "provider_id parameter required",
            )
        })?;

    // Get stats from database
    let mut stats = ModelOperations::get_provider_storage_stats(pool.as_ref(), &provider_id)
        .await
        .map_err(AppError::database_error)?;

    // Enhanced stats using ModelStorage
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    // Get actual storage size from filesystem
    if let Ok(actual_size) = storage.get_provider_storage_size(&provider_id).await {
        stats.total_storage_bytes = actual_size as u64;
        println!(
            "Provider {} actual storage size: {} bytes",
            provider_id, actual_size
        );
    }

    // List all models in storage
    if let Ok(stored_models) = storage.list_provider_models(&provider_id).await {
        println!(
            "Found {} models in storage for provider {}",
            stored_models.len(),
            provider_id
        );

        // Validate each model and update stats if needed
        for (model_id, _metadata) in &stored_models {
            match storage.validate_model(&provider_id, model_id).await {
                Ok(issues) => {
                    if !issues.is_empty() {
                        println!("Model {} has validation issues: {:?}", model_id, issues);
                    }
                }
                Err(e) => {
                    println!("Failed to validate model {}: {}", model_id, e);
                }
            }
        }
    }

    Ok(Json(stats))
}

// Hugging Face download functions removed - will be implemented later

/// File type classification for validation
#[derive(Debug, PartialEq)]
enum ModelFileType {
    WeightFile,    // .bin, .safetensors, .gguf, etc.
    ConfigFile,    // config.json, config_*
    TokenizerFile, // tokenizer.json, tokenizer_config.json
    VocabFile,     // vocab.json, merges.txt, special_tokens_map.json
    UnknownFile,   // Everything else
}

impl std::fmt::Display for ModelFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFileType::WeightFile => write!(f, "weight"),
            ModelFileType::ConfigFile => write!(f, "config"),
            ModelFileType::TokenizerFile => write!(f, "tokenizer"),
            ModelFileType::VocabFile => write!(f, "vocab"),
            ModelFileType::UnknownFile => write!(f, "unknown"),
        }
    }
}

/// Determine the type of model file based on filename
fn determine_file_type(filename_lower: &str) -> ModelFileType {
    // Weight files (actual model parameters)
    if filename_lower.ends_with(".bin")
        || filename_lower.ends_with(".pt")
        || filename_lower.ends_with(".pth")
        || filename_lower.ends_with(".safetensors")
        || filename_lower.ends_with(".gguf")
        || filename_lower.ends_with(".ggml")
    {
        return ModelFileType::WeightFile;
    }

    // Configuration files
    if filename_lower == "config.json"
        || filename_lower.starts_with("config_")
        || filename_lower == "generation_config.json"
    {
        return ModelFileType::ConfigFile;
    }

    // Tokenizer files
    if filename_lower == "tokenizer.json"
        || filename_lower == "tokenizer_config.json"
        || filename_lower.starts_with("tokenizer_")
    {
        return ModelFileType::TokenizerFile;
    }

    // Vocabulary and token files
    if filename_lower == "vocab.json"
        || filename_lower == "merges.txt"
        || filename_lower == "special_tokens_map.json"
        || filename_lower == "vocab.txt"
        || filename_lower == "spiece.model"
    {
        return ModelFileType::VocabFile;
    }

    ModelFileType::UnknownFile
}
