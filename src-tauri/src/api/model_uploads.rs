use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::Deserialize;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;
use futures::StreamExt;

use crate::api::{
    errors::{AppError, ApiResult, ErrorCode},
    middleware::AuthenticatedUser,
};
use crate::database::{
    get_database_pool,
    model_operations::ModelOperations,
    models::*,
};
use crate::utils::model_storage::ModelStorage;

pub fn _create_router() -> Router<()> {
    Router::new()
        .route("/", post(create_model))
        .route("/", get(list_models))
        .route("/:model_id", get(get_model))
        .route("/:model_id", put(update_model))
        .route("/:model_id", delete(delete_model))
        .route("/:model_id/upload", post(upload_model_file))
        .route("/:model_id/validate", post(validate_model))
        .route("/:model_id/status", put(update_model_status))
        .route("/storage-stats", get(get_storage_stats))
}

#[derive(Deserialize)]
pub struct ListModelsQuery {
    provider_id: Uuid,
    page: Option<i32>,
    per_page: Option<i32>,
}

#[derive(Deserialize)]
pub struct UploadFileRequest {
    filename: String,
    file_size: u64,
}

#[derive(Deserialize)]
pub struct CreateUploadModelRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub architecture: String,
    pub file_format: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateModelStatusRequest {
    enabled: Option<bool>,
    is_active: Option<bool>,
}

/// Create a new model for upload
pub async fn create_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUploadModelRequest>,
) -> ApiResult<Json<UploadedModel>> {
    let pool = get_database_pool()?;
    
    // Validate provider exists and is of type 'candle'
    let provider_row = sqlx::query(
        "SELECT id, provider_type FROM model_providers WHERE id = $1"
    )
    .bind(request.provider_id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(AppError::database_error)?;
    
    let provider_row = provider_row.ok_or_else(|| {
        AppError::new(ErrorCode::ValidInvalidInput,"Provider not found")
    })?;
    
    let provider_type: String = provider_row.get("provider_type");
    
    if provider_type != "candle" {
        return Err(AppError::new(ErrorCode::ValidInvalidInput,
            "Only Candle providers support model uploads"
        ));
    }
    
    // Create storage path
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    let model_id = Uuid::new_v4();
    let model_path = storage.get_model_path(&request.provider_id, &model_id);
    let model_path_str = model_path.to_string_lossy().to_string();
    
    // Create default parameters with file_format if provided
    let mut parameters = serde_json::json!({
        "max_tokens": 512,
        "temperature": 0.7,
        "top_p": 0.9,
        "repeat_penalty": 1.1,
        "repeat_last_n": 64
    });
    
    if let Some(file_format) = &request.file_format {
        parameters["file_format"] = serde_json::Value::String(file_format.clone());
    }
    
    // Merge metadata into parameters if provided
    if let Some(metadata) = &request.metadata {
        if let serde_json::Value::Object(metadata_obj) = metadata {
            if let serde_json::Value::Object(ref mut params_obj) = parameters {
                for (key, value) in metadata_obj {
                    params_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    // Convert to CreateUploadedModelRequest
    let create_request = crate::database::models::CreateUploadedModelRequest {
        provider_id: request.provider_id,
        name: request.name,
        alias: request.alias,
        description: request.description,
        architecture: request.architecture,
        quantization: None,
        capabilities: Some(serde_json::json!({})),
        parameters: Some(parameters),
    };
    
    // Create the model record
    let model_db = ModelOperations::create_model(pool.as_ref(), &create_request, &model_path_str)
        .await
        .map_err(AppError::database_error)?;
    
    // Create storage directory
    storage.create_model_directory(&request.provider_id, &model_db.id)
        .map_err(|e| AppError::internal_error(format!("Failed to create storage directory: {}", e)))?;
    
    // Save initial metadata
    let metadata = crate::utils::model_storage::ModelMetadata {
        name: create_request.name.clone(),
        architecture: create_request.architecture.clone(),
        parameters: None,
        quantization: None,
        size_bytes: 0,
        uploaded_at: chrono::Utc::now(),
        files: vec![],
    };
    
    if let Err(e) = storage.save_metadata(&request.provider_id, &model_db.id, &metadata) {
        eprintln!("Warning: Failed to save model metadata: {}", e);
    } else {
        println!("Saved initial metadata for model {}", model_db.id);
    }
    
    // Check if this is a Hugging Face download or local folder upload request
    if let Some(metadata) = &request.metadata {
        if let Some(source) = metadata.get("source") {
            if source == "huggingface" {
                // Start Hugging Face download in background
                let model_id = model_db.id;
                let provider_id = request.provider_id;
                let metadata_clone = metadata.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = download_from_huggingface(model_id, provider_id, metadata_clone).await {
                        eprintln!("Failed to download model from Hugging Face: {}", e);
                        
                        // Update model status to failed
                        if let Ok(pool) = get_database_pool() {
                            let error_message = vec![format!("Download failed: {}", e)];
                            if let Err(update_err) = ModelOperations::update_model_validation(
                                pool.as_ref(),
                                &model_id,
                                "failed",
                                Some(&error_message),
                                None,
                            ).await {
                                eprintln!("Failed to update model status to failed: {}", update_err);
                            }
                        }
                    }
                });
            } else if source == "local_folder" {
                // Update model status to await_upload for local folder uploads
                ModelOperations::update_model_validation(
                    pool.as_ref(),
                    &model_db.id,
                    "await_upload",
                    None,
                    None,
                ).await.map_err(AppError::database_error)?;
                
                println!("Model created for local folder upload: {} files expected", 
                    metadata.get("total_files").and_then(|v| v.as_u64()).unwrap_or(0));
            }
        }
    }
    
    // Convert to API response
    let model = UploadedModel::from_db(model_db, vec![]);
    
    Ok(Json(model))
}

/// List models for a provider
pub async fn list_models(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListModelsQuery>,
) -> ApiResult<Json<ModelListResponse>> {
    let pool = get_database_pool()?;
    
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    
    let (models, total) = ModelOperations::list_models_with_files_for_provider(
        pool.as_ref(),
        &query.provider_id,
        page,
        per_page,
    )
    .await
    .map_err(AppError::database_error)?;
    
    // Calculate total storage
    let total_storage_bytes = models.iter()
        .map(|m| m.file_size_bytes as u64)
        .sum();
    
    let response = ModelListResponse {
        models,
        total,
        page,
        per_page,
        total_storage_bytes,
    };
    
    Ok(Json(response))
}

/// Get model details
pub async fn get_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<Json<ModelDetailsResponse>> {
    let pool = get_database_pool()?;
    
    let model = ModelOperations::get_model_with_files(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;
    
    let files = model.files.clone();
    let storage_size_bytes = model.file_size_bytes as u64;
    let validation_issues = model.validation_issues.clone().unwrap_or_default();
    
    // Try to load additional metadata from storage
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    if let Ok(stored_metadata) = storage.load_metadata(&model.provider_id, &model_id) {
        println!("Loaded stored metadata for model {}: {} files", model_id, stored_metadata.files.len());
    }
    
    let response = ModelDetailsResponse {
        model,
        files,
        storage_size_bytes,
        validation_issues,
    };
    
    Ok(Json(response))
}

/// Update model metadata
pub async fn update_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UpdateUploadedModelRequest>,
) -> ApiResult<Json<UploadedModel>> {
    let pool = get_database_pool()?;
    
    // Update model
    ModelOperations::update_model(pool.as_ref(), &model_id, &request)
        .await
        .map_err(AppError::database_error)?;
    
    // Return updated model
    let model = ModelOperations::get_model_with_files(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;
    
    Ok(Json(model))
}

/// Delete model
pub async fn delete_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let pool = get_database_pool()?;
    
    // Get model info for cleanup
    let model = ModelOperations::get_model_by_id(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?
        .ok_or_else(|| AppError::not_found("Model"))?;
    
    // Delete from storage
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    if let Err(e) = storage.delete_model(&model.provider_id, &model_id) {
        eprintln!("Warning: Failed to delete model files: {}", e);
    }
    
    // Delete from database
    ModelOperations::delete_model(pool.as_ref(), &model_id)
        .await
        .map_err(AppError::database_error)?;
    
    Ok(StatusCode::NO_CONTENT)
}

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
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    // Get the model directory path
    let model_path = storage.get_model_path(&model.provider_id, &model_id);
    let file_path = model_path.join(&request.filename);
    
    // For local folder uploads, we expect the files to already be accessible
    // This is a simplified implementation - in production you'd handle actual file uploads
    println!("Processing upload for file: {} ({}bytes)", request.filename, request.file_size);
    
    // Calculate checksum if file exists
    if file_path.exists() {
        match std::fs::read(&file_path) {
            Ok(file_data) => {
                // Save the file through ModelStorage which will calculate checksum
                match storage.save_model_file(&model.provider_id, &model_id, &request.filename, &file_data).await {
                    Ok(model_file) => {
                        if let Some(checksum) = model_file.checksum {
                            // Update model checksum in database
                            ModelOperations::update_model_checksum(
                                pool.as_ref(),
                                &model_id,
                                &checksum,
                            ).await.map_err(AppError::database_error)?;
                            
                            println!("Updated model checksum: {}", checksum);
                        }
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
    ).await.map_err(AppError::database_error)?;
    
    let response = ModelUploadResponse {
        model_id,
        upload_url: None,
        chunk_uploaded: true,
        upload_complete: true,
        next_chunk_index: None,
    };
    
    Ok(Json(response))
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
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    // Validate model using both storage and utils
    let mut validation_issues = storage.validate_model(&model.provider_id, &model_id)
        .map_err(|e| AppError::internal_error(format!("Validation failed: {}", e)))?;
    
    // Additional validation using ModelUtils
    let model_path = storage.get_model_path(&model.provider_id, &model_id);
    if let Some(model_path_str) = model_path.to_str() {
        // Validate model name
        if let Err(e) = crate::utils::model_storage::ModelUtils::validate_model_name(&model.name) {
            validation_issues.push(format!("Invalid model name: {}", e));
        }
        
        // Check if model exists using verification function
        if let Err(e) = crate::utils::model_storage::ModelUtils::verify_model_exists(model_path_str, &model.name) {
            validation_issues.push(format!("Model verification failed: {}", e));
        }
        
        // Get and validate model size
        if let Ok(model_size) = crate::ai::candle_models::ModelUtils::get_model_size(model_path_str) {
            let formatted_size = crate::utils::model_storage::ModelUtils::format_model_size(model_size);
            println!("Model size: {}", formatted_size);
            
            // Warn if model is suspiciously small
            if model_size < 1024 * 1024 { // Less than 1MB
                validation_issues.push("Model files appear to be very small (< 1MB)".to_string());
            }
        }
        
        // Extract and validate model info from config.json if present
        let config_path = model_path.join("config.json");
        if config_path.exists() {
            if let Ok(config_content) = std::fs::read_to_string(&config_path) {
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
        match crate::utils::model_storage::ModelUtils::discover_models(model_path_str) {
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
        if let Ok(model_list) = crate::ai::candle_models::ModelUtils::list_models(model_path_str) {
            println!("Available models: {:?}", model_list);
            if model_list.is_empty() {
                validation_issues.push("No model directories found".to_string());
            }
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
) -> ApiResult<Json<UploadedModel>> {
    let pool = get_database_pool()?;
    
    // Update model status
    ModelOperations::update_model_status(pool.as_ref(), &model_id, request.enabled, request.is_active)
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
    
    let provider_id = params.get("provider_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::new(ErrorCode::ValidInvalidInput,"provider_id parameter required"))?;
    
    // Get stats from database
    let mut stats = ModelOperations::get_provider_storage_stats(pool.as_ref(), &provider_id)
        .await
        .map_err(AppError::database_error)?;
    
    // Enhanced stats using ModelStorage
    let storage = ModelStorage::new().map_err(|e| {
        AppError::internal_error(format!("Storage initialization failed: {}", e))
    })?;
    
    // Get actual storage size from filesystem
    if let Ok(actual_size) = storage.get_provider_storage_size(&provider_id) {
        stats.total_storage_bytes = actual_size as u64;
        println!("Provider {} actual storage size: {} bytes", provider_id, actual_size);
    }
    
    // List all models in storage
    if let Ok(stored_models) = storage.list_provider_models(&provider_id) {
        println!("Found {} models in storage for provider {}", stored_models.len(), provider_id);
        
        // Validate each model and update stats if needed
        for (model_id, _metadata) in &stored_models {
            match storage.validate_model(&provider_id, model_id) {
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

/// Download a model from Hugging Face
async fn download_from_huggingface(
    model_id: Uuid,
    provider_id: Uuid,
    metadata: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = get_database_pool()?;
    
    // Extract Hugging Face parameters from metadata
    let hf_repo = metadata.get("hf_repo")
        .and_then(|v| v.as_str())
        .ok_or("Missing hf_repo in metadata")?;
    let hf_filename = metadata.get("hf_filename")
        .and_then(|v| v.as_str())
        .ok_or("Missing hf_filename in metadata")?;
    let hf_branch = metadata.get("hf_branch")
        .and_then(|v| v.as_str())
        .unwrap_or("main");
    let hf_token = metadata.get("hf_token")
        .and_then(|v| v.as_str());
    
    // Define required files for a complete model
    let mut files_to_download = vec![hf_filename.to_string()];
    
    // Add essential config and tokenizer files
    let essential_files = vec![
        "config.json",
        "tokenizer.json",
        "tokenizer_config.json",
        "vocab.json",
        "merges.txt",
        "special_tokens_map.json",
    ];
    
    for file in essential_files {
        if !files_to_download.contains(&file.to_string()) {
            files_to_download.push(file.to_string());
        }
    }
    
    // Create HTTP client without authentication first
    let client = reqwest::Client::builder()
        .user_agent("ziee-desktop/1.0.0")
        .build()?;
    
    // Check if repo is public by trying to access without token
    let test_url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        hf_repo,
        hf_branch,
        hf_filename
    );
    
    let test_response = client.head(&test_url).send().await?;
    let needs_token = test_response.status() == 401 || test_response.status() == 403;
    
    // Create client with token only if needed
    let client = if needs_token && hf_token.is_some() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", hf_token.unwrap()))?
        );
        reqwest::Client::builder()
            .user_agent("ziee-desktop/1.0.0")
            .default_headers(headers)
            .build()?
    } else if needs_token {
        return Err("Repository requires authentication but no token provided".into());
    } else {
        println!("Repository is public, downloading without authentication");
        client
    };
    
    // Get model storage path
    let storage = ModelStorage::new()?;
    let model_path = storage.get_model_path(&provider_id, &model_id);
    
    // Ensure parent directory exists
    std::fs::create_dir_all(&model_path)?;
    
    // Update model status to downloading
    ModelOperations::update_model_validation(
        pool.as_ref(),
        &model_id,
        "downloading",
        None,
        None,
    ).await?;
    
    let mut downloaded_files = Vec::new();
    let mut total_size = 0u64;
    
    // Download each file
    for (index, filename) in files_to_download.iter().enumerate() {
        println!("Downloading file {}/{}: {}", index + 1, files_to_download.len(), filename);
        
        match download_single_file(
            &client,
            hf_repo,
            hf_branch,
            filename,
            &model_path,
        ).await {
            Ok((file_size, file_checksum)) => {
                downloaded_files.push((filename.clone(), file_size, file_checksum));
                total_size += file_size;
                println!("Successfully downloaded: {}", filename);
            }
            Err(e) => {
                // For essential files like the main model file, fail the entire download
                if filename == hf_filename {
                    return Err(format!("Failed to download main model file '{}': {}", filename, e).into());
                }
                // For optional files like config/tokenizer, just log the error and continue
                println!("Warning: Failed to download optional file '{}': {}", filename, e);
            }
        }
    }
    
    // Create model file records for all downloaded files
    for (filename, file_size, checksum) in &downloaded_files {
        let file_path = model_path.join(filename);
        let file_type = if filename == hf_filename { "model" } else { "config" };
        
        ModelOperations::create_model_file(
            pool.as_ref(),
            &model_id,
            filename,
            &file_path.to_string_lossy(),
            *file_size as i64,
            file_type,
            checksum,
        ).await.map_err(|e| format!("Failed to create model file record for '{}': {}", filename, e))?;
    }
    
    // Validate the downloaded files
    let mut all_validation_issues = Vec::new();
    for (filename, file_size, _) in &downloaded_files {
        let file_path = model_path.join(filename);
        if let Ok(issues) = validate_downloaded_model(&file_path, filename, *file_size as usize) {
            all_validation_issues.extend(issues);
        }
    }
    
    let final_status = if all_validation_issues.is_empty() {
        "completed"
    } else {
        "validation_warning"
    };
    
    // Update model with total file size and set final status
    ModelOperations::update_model_validation(
        pool.as_ref(),
        &model_id,
        final_status,
        if all_validation_issues.is_empty() { None } else { Some(&all_validation_issues) },
        Some(total_size as i64),
    ).await.map_err(|e| format!("Failed to update model validation status: {}", e))?;
    
    if all_validation_issues.is_empty() {
        println!("Successfully downloaded and validated {} files from Hugging Face ({})", downloaded_files.len(), hf_repo);
    } else {
        println!("Downloaded {} files from Hugging Face ({}) with validation warnings: {:?}", downloaded_files.len(), hf_repo, all_validation_issues);
    }
    
    Ok(())
}

/// Download a single file from Hugging Face
async fn download_single_file(
    client: &reqwest::Client,
    hf_repo: &str,
    hf_branch: &str,
    filename: &str,
    model_path: &std::path::Path,
) -> Result<(u64, String), Box<dyn std::error::Error + Send + Sync>> {
    // Build Hugging Face URL for this file
    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        hf_repo,
        hf_branch,
        filename
    );
    
    // Download the file
    let response = client.get(&url).send().await.map_err(|e| {
        format!("Network error while downloading '{}' from Hugging Face: {}", filename, e)
    })?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!(
            "Failed to download file '{}' from Hugging Face: HTTP {} - {}",
            filename,
            status,
            if error_body.is_empty() { "No error details" } else { &error_body }
        ).into());
    }
    
    let content_length = response.content_length().unwrap_or(0);
    let file_path = model_path.join(filename);
    let mut file = std::fs::File::create(&file_path).map_err(|e| {
        format!("Failed to create file {}: {}", file_path.display(), e)
    })?;
    let mut downloaded = 0u64;
    
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error during download of '{}': {}", filename, e))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| {
            format!("Failed to write to file {}: {}", file_path.display(), e)
        })?;
        downloaded += chunk.len() as u64;
        
        // Report download progress for larger files (>1MB)
        if content_length > 1024 * 1024 {
            if content_length > 0 {
                let progress = (downloaded as f64 / content_length as f64) * 100.0;
                
                // Log progress every 25% for individual files
                if (progress as u32) % 25 == 0 && (progress as u32) != 0 {
                    println!("Download progress for '{}': {:.1}%", filename, progress);
                }
            }
        }
    }
    
    // Calculate file checksum
    let file_content = std::fs::read(&file_path).map_err(|e| {
        format!("Failed to read downloaded file '{}' for checksum: {}", filename, e)
    })?;
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let checksum = format!("{:x}", hasher.finalize());
    
    Ok((downloaded, checksum))
}

/// Validate a downloaded model file
fn validate_downloaded_model(
    file_path: &std::path::Path,
    filename: &str,
    file_size: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut issues = Vec::new();
    
    // Check if file exists and is not empty
    if !file_path.exists() {
        issues.push("Downloaded file does not exist".to_string());
        return Ok(issues);
    }
    
    if file_size == 0 {
        issues.push("Downloaded file is empty".to_string());
    }
    
    // Check file size is reasonable (at least 1KB for model files)
    if file_size < 1024 {
        issues.push("Downloaded file is suspiciously small (< 1KB)".to_string());
    }
    
    // Check file extension matches expected types
    let filename_lower = filename.to_lowercase();
    let valid_extensions = [".bin", ".pt", ".pth", ".safetensors", ".gguf", ".ggml"];
    let has_valid_extension = valid_extensions.iter().any(|ext| filename_lower.ends_with(ext));
    
    if !has_valid_extension {
        issues.push(format!("File '{}' has unexpected extension for a model file", filename));
    }
    
    // Basic file content validation
    if let Ok(first_bytes) = std::fs::read(&file_path) {
        if first_bytes.len() >= 4 {
            let first_4_bytes = &first_bytes[0..4];
            
            // Check for some common file format signatures
            match &first_4_bytes {
                // ZIP file signature (could be problematic for direct model loading)
                [0x50, 0x4B, 0x03, 0x04] | [0x50, 0x4B, 0x05, 0x06] | [0x50, 0x4B, 0x07, 0x08] => {
                    if !filename_lower.ends_with(".gguf") { // GGUF files might contain ZIP-like headers
                        issues.push("File appears to be a ZIP archive, which may not be directly loadable".to_string());
                    }
                },
                // Check for HTML (error pages)
                [0x3C, 0x21, _, _] | [0x3C, 0x68, 0x74, 0x6D] | [0x3C, 0x48, 0x54, 0x4D] => {
                    issues.push("File appears to be HTML content (possibly an error page)".to_string());
                },
                _ => {} // Unknown format, could be fine
            }
        }
    }
    
    Ok(issues)
}