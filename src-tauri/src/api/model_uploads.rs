use axum::{
    extract::Multipart,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    response::Json,
    response::Response,
    Extension,
};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError, ErrorCode},
    middleware::AuthenticatedUser,
};
use crate::database::{
    models::*,
    queries::{models, repositories},
};
use crate::utils::git_service::{GitProgress, GitService};

use crate::utils::model_storage::ModelStorage;

#[derive(Serialize)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub model: Option<Model>,
}

/// Shared model creation and file processing logic
async fn create_model_with_files(
    storage: &ModelStorage,
    provider_id: Uuid,
    name: String,
    alias: String,
    description: Option<String>,
    file_format: String,
    main_filename: String,
    source_dir: PathBuf,
    capabilities: Option<ModelCapabilities>,
    settings: Option<ModelSettings>,
) -> Result<Model, AppError> {
    // Validate provider exists and is of type 'local'
    let provider = crate::database::queries::providers::get_provider_by_id(provider_id)
        .await
        .map_err(|e| AppError::internal_error(&e.to_string()))?
        .ok_or_else(|| AppError::new(ErrorCode::ValidInvalidInput, "Provider not found"))?;

    if provider.provider_type != "local" {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Only Local providers support model uploads",
        ));
    }

    // Generate model ID first (but don't create in database yet)
    let model_id = Uuid::new_v4();
    let model_name = name.clone();

    println!("Processing model with file format: {}", file_format);

    // Create storage directory
    storage
        .create_model_directory(&provider_id, &model_id)
        .await
        .map_err(|e| {
            AppError::internal_error(format!("Failed to create storage directory: {}", e))
        })?;

    // print source directory
    println!("Source directory for model files: {}", source_dir.display());

    // List all files in the source directory
    let source_files = match tokio::fs::read_dir(&source_dir).await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                AppError::internal_error(format!("Failed to read directory entry: {}", e))
            })? {
                if entry
                    .file_type()
                    .await
                    .map_err(|e| {
                        AppError::internal_error(format!("Failed to get file type: {}", e))
                    })?
                    .is_file()
                {
                    files.push(entry.file_name().to_string_lossy().to_string());
                }
            }
            files
        }
        Err(e) => {
            return Err(AppError::internal_error(format!(
                "Failed to read source directory: {}",
                e
            )));
        }
    };

    if source_files.is_empty() {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "No files found in source directory",
        ));
    }

    // Determine which files to copy based on main filename and index files
    let files_to_copy = determine_files_to_copy(&source_files, &main_filename)?;

    if files_to_copy.is_empty() {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            format!(
                "No relevant files found for main filename: {}",
                main_filename
            ),
        ));
    }

    println!(
        "Found {} files to copy: {:?}",
        files_to_copy.len(),
        files_to_copy
    );

    // Copy the necessary files to the model directory and collect file info
    let mut total_size = 0u64;
    let file_count = files_to_copy.len();
    let mut file_records = Vec::new();

    for filename in &files_to_copy {
        let source_path = source_dir.join(filename);
        let dest_path = storage
            .get_model_path(&provider_id, &model_id)
            .join(filename);

        // Get file size
        let metadata = tokio::fs::metadata(&source_path).await.map_err(|e| {
            AppError::internal_error(format!(
                "Failed to get file metadata for {}: {}",
                filename, e
            ))
        })?;
        let file_size = metadata.len();
        total_size += file_size;

        // Copy the file
        tokio::fs::copy(&source_path, &dest_path)
            .await
            .map_err(|e| {
                AppError::internal_error(format!("Failed to copy file {}: {}", filename, e))
            })?;

        // Collect file information for database insertion later
        let file_type = determine_model_file_type(filename).to_string();
        let relative_path = format!("models/{}/{}/{}", provider_id, model_id, filename);

        file_records.push((
            filename.clone(),
            relative_path.clone(),
            file_size,
            file_type.clone(),
        ));

        println!(
            "Copied file: {} -> {} ({} bytes)",
            filename, relative_path, file_size
        );
    }

    // Now that all files are processed successfully, create the model in the database
    let create_request = CreateModelRequest {
        provider_id,
        name,
        alias,
        description,
        enabled: Some(true), // Enable immediately since everything succeeded
        capabilities: capabilities.or_else(|| Some(ModelCapabilities::new())),
        settings,
    };

    // Create the model record with the pre-generated ID
    let _model_db = models::create_local_model(&model_id, &create_request)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("models_provider_id_name_unique") {
                AppError::new(ErrorCode::ValidInvalidInput,
                              format!("Model ID '{}' already exists for this provider. Please use a different model ID.", model_name))
            } else {
                AppError::internal_error(&error_str)
            }
        })?;

    // Create all file records in the database
    for (filename, relative_path, file_size, file_type) in file_records {
        models::create_model_file(
            &model_id,
            &filename,
            &relative_path,
            file_size as i64,
            &file_type,
        )
        .await
        .map_err(|e| AppError::internal_error(&e.to_string()))?;
    }

    // Update model with total size and validation status
    models::update_model_validation(&model_id, "completed", None, Some(total_size as i64))
        .await
        .map_err(AppError::database_error)?;

    // Return the created model with files
    let model = models::get_model_with_files(&model_id)
        .await
        .map_err(|e| AppError::internal_error(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Model"))?;

    println!(
        "Model created successfully: {} files, {} total size",
        file_count, total_size
    );

    Ok(model)
}

/// Determine which files to copy based on main filename and index files
fn determine_files_to_copy(
    source_files: &[String],
    main_filename: &str,
) -> Result<Vec<String>, AppError> {
    let mut files_to_copy = Vec::new();

    // First, check if main_filename ends with .json (if so, it might be an index file already)
    let main_is_json = main_filename.to_lowercase().ends_with(".json");
    
    // If main file doesn't end with .json, look for {main_filename}.index.json
    let index_filename = if !main_is_json {
        format!("{}.index.json", main_filename)
    } else {
        // If main file is already JSON, it might be the index file
        main_filename.to_string()
    };

    // Always check for index file first
    let index_exists = !main_is_json && source_files.contains(&index_filename);
    let main_exists = source_files.contains(&main_filename.to_string());
    
    // Check if index file exists first
    if index_exists {
        println!("Found index file: {}", index_filename);
        
        // Add the index file itself
        files_to_copy.push(index_filename.clone());
        
        // Parse the index file to get weight files
        // Since we're in the determine_files_to_copy function which doesn't have async context,
        // we'll need to identify weight files by pattern matching based on the index file name
        
        // For sharded models, weight files typically follow patterns like:
        // - model-00001-of-00004.safetensors
        // - pytorch_model-00001-of-00005.bin
        // We'll add all files that match the base pattern
        
        let base_name = main_filename.trim_end_matches(".safetensors")
            .trim_end_matches(".bin")
            .trim_end_matches(".pt")
            .trim_end_matches(".pth");
        
        // Add all weight files that match the sharding pattern
        for file in source_files {
            if file.starts_with(base_name) && 
               (file.contains("-of-") || file.contains("_of_")) &&
               (file.ends_with(".safetensors") || file.ends_with(".bin") || 
                file.ends_with(".pt") || file.ends_with(".pth")) {
                files_to_copy.push(file.clone());
            }
        }
    } else if main_is_json && (main_filename.contains("index") || main_filename.ends_with(".index.json")) {
        // Main file is already an index file
        println!("Main file is an index file: {}", main_filename);
        
        files_to_copy.push(main_filename.to_string());
        
        // Extract base name from index file
        let base_name = main_filename
            .replace(".index.json", "")
            .replace("_index.json", "")
            .replace("-index.json", "");
        
        // Add all related weight files
        for file in source_files {
            if file.starts_with(&base_name) && 
               file != main_filename &&
               (file.ends_with(".safetensors") || file.ends_with(".bin") || 
                file.ends_with(".pt") || file.ends_with(".pth")) {
                files_to_copy.push(file.clone());
            }
        }
    } else if main_exists {
        // No index file found but main file exists - only copy the main weight file
        println!("No index file found for {}. Only copying main file.", main_filename);
        files_to_copy.push(main_filename.to_string());
    } else {
        // Neither index file nor main file exists - throw error
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            format!(
                "Neither '{}' nor '{}' found in source directory",
                main_filename,
                if !main_is_json { &index_filename } else { main_filename }
            ),
        ));
    }

    // Always add configuration and tokenizer files regardless of sharding
    for file in source_files {
        if is_config_or_tokenizer_file(file) && !files_to_copy.contains(&file.to_string()) {
            files_to_copy.push(file.clone());
        }
    }

    // Remove duplicates and sort
    files_to_copy.sort();
    files_to_copy.dedup();

    println!("Files to copy: {:?}", files_to_copy);

    Ok(files_to_copy)
}

/// Check if a file is a configuration or tokenizer file
fn is_config_or_tokenizer_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    filename_lower.ends_with("config.json")
        || filename_lower.ends_with("tokenizer.json")
        || filename_lower.ends_with("tokenizer_config.json")
        || filename_lower.ends_with("vocab.json")
        || filename_lower.ends_with("merges.txt")
        || filename_lower.ends_with("special_tokens_map.json")
        || filename_lower.ends_with("vocab.txt")
        || filename_lower.ends_with("spiece.model")
        || filename_lower == "generation_config.json"
}


#[derive(Debug, serde::Serialize)]
pub struct UploadFilesResponse {
    pub session_id: Uuid,
    pub total_size_bytes: u64,
    pub main_filename: String,
    pub provider_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessedFile {
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
    pub file_format: String,
    pub capabilities: Option<ModelCapabilities>,
    pub settings: Option<ModelSettings>,
    pub main_filename: String,
}

#[derive(Deserialize)]
pub struct DownloadFromRepositoryRequest {
    pub provider_id: Uuid,
    pub repository_id: Uuid,
    pub repository_path: String, // e.g., "microsoft/DialoGPT-medium"
    pub repository_branch: Option<String>, // e.g., "main"
    pub name: String,            // model ID
    pub alias: String,           // display name
    pub description: Option<String>,
    pub file_format: String,
    pub main_filename: String,
    pub capabilities: Option<ModelCapabilities>,
    pub settings: Option<ModelSettings>,
}

/// Upload multiple model files and auto-commit as a model
pub async fn upload_multiple_files_and_commit(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> ApiResult<Json<Model>> {
    let storage = ModelStorage::new()
        .await
        .map_err(|e| AppError::internal_error(format!("Storage initialization failed: {}", e)))?;

    let mut uploaded_files = Vec::new();
    let mut main_filename: Option<String> = None;
    let mut provider_id: Option<Uuid> = None;
    let mut name: Option<String> = None;
    let mut alias: Option<String> = None;
    let mut description: Option<String> = None;
    let mut file_format: Option<String> = None;
    let mut capabilities: Option<ModelCapabilities> = None;
    let mut settings: Option<ModelSettings> = None;

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
            "name" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read name: {}", e),
                    )
                })?;
                name = Some(value);
            }
            "alias" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read alias: {}", e),
                    )
                })?;
                alias = Some(value);
            }
            "description" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read description: {}", e),
                    )
                })?;
                description = if value.is_empty() { None } else { Some(value) };
            }
            "file_format" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read file_format: {}", e),
                    )
                })?;
                file_format = Some(value);
            }
            "capabilities" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read capabilities: {}", e),
                    )
                })?;
                if !value.is_empty() {
                    capabilities = serde_json::from_str(&value).map_err(|e| {
                        AppError::new(
                            ErrorCode::ValidInvalidInput,
                            format!("Invalid capabilities JSON: {}", e),
                        )
                    })?;
                }
            }
            "settings" => {
                let value = field.text().await.map_err(|e| {
                    AppError::new(
                        ErrorCode::ValidInvalidInput,
                        format!("Failed to read settings: {}", e),
                    )
                })?;
                if !value.is_empty() {
                    settings = serde_json::from_str(&value).map_err(|e| {
                        AppError::new(
                            ErrorCode::ValidInvalidInput,
                            format!("Invalid settings JSON: {}", e),
                        )
                    })?;
                }
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

    let name = name.ok_or_else(|| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            "Missing name in multipart request",
        )
    })?;

    let alias = alias.ok_or_else(|| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            "Missing alias in multipart request",
        )
    })?;

    let file_format = file_format.ok_or_else(|| {
        AppError::new(
            ErrorCode::ValidInvalidInput,
            "Missing file_format in multipart request",
        )
    })?;

    println!(
        "Processing multipart upload: {} files, main file: {}, name: {}, alias: {}",
        uploaded_files.len(),
        main_filename,
        name,
        alias
    );

    // Step 1: Upload files to temporary storage
    let temp_session_id = Uuid::new_v4();
    let mut total_size = 0u64;

    for (filename, file_data) in uploaded_files {
        total_size += file_data.len() as u64;

        // Check and validate files
        let _file_type = determine_model_file_type(&filename);
        let _validation_issues = validate_file_content(&filename, &file_data);

        // Save files to temporary storage
        let temp_file_id = Uuid::new_v4();
        match storage
            .save_temp_file(&temp_session_id, &temp_file_id, &filename, &file_data)
            .await
        {
            Ok(_temp_file) => {
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

    println!(
        "Files uploaded successfully, total size: {} bytes",
        total_size
    );

    // Step 2: Auto-commit the uploaded files as a model
    let source_dir = crate::APP_DATA_DIR
        .join("temp")
        .join(temp_session_id.to_string());

    // Create model using the existing function
    let model = create_model_with_files(
        &storage,
        provider_id,
        name,
        alias,
        description,
        file_format,
        main_filename,
        source_dir,
        capabilities,
        settings,
    )
    .await
    .map_err(|e| {
        AppError::internal_error(format!("Failed to create model from uploaded files: {}", e))
    })?;

    println!("Model created successfully: {} ({})", model.alias, model.id);

    Ok(Json(model))
}

/// Download model from repository and commit files with SSE progress streaming
pub async fn download_and_commit_repository_files(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<DownloadFromRepositoryRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, Response> {
    // Get repository information
    let repository = match repositories::get_repository_by_id(request.repository_id).await {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            let error_response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Repository not found".into())
                .unwrap();
            return Err(error_response);
        }
        Err(e) => {
            let error_response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Database error: {}", e).into())
                .unwrap();
            return Err(error_response);
        }
    };

    // Build repository URL
    let repository_url =
        GitService::build_repository_url(&repository.url, &request.repository_path);

    // Create progress channels
    let (progress_tx, progress_rx) = mpsc::unbounded_channel::<GitProgress>();
    let (sse_tx, sse_rx) = mpsc::unbounded_channel::<DownloadProgress>();

    // Create git service
    let git_service = GitService::new();

    // Prepare authentication based on repository auth_type
    let auth_token = match repository.auth_type.as_str() {
        "api_key" => repository
            .auth_config
            .as_ref()
            .and_then(|config| config.api_key.clone()),
        "bearer_token" => repository
            .auth_config
            .as_ref()
            .and_then(|config| config.token.clone()),
        "basic_auth" => {
            // For basic auth, return username:password format
            if let Some(config) = &repository.auth_config {
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    Some(format!("{}:{}", username, password))
                } else {
                    None
                }
            } else {
                None
            }
        },
        "none" | _ => None,
    };
    let provider_id = request.provider_id;
    let name = request.name.clone();
    let alias = request.alias.clone();
    let description = request.description.clone();
    let file_format = request.file_format.clone();
    let main_filename = request.main_filename.clone();
    let capabilities = request.capabilities.clone();
    let settings = request.settings.clone();
    let repository_id = request.repository_id;

    // Spawn task to handle git progress and auto-commit
    tokio::spawn(async move {
        // Convert git progress to SSE progress
        let progress_task = {
            let sse_tx = sse_tx.clone();
            tokio::spawn(async move {
                let mut progress_rx = progress_rx;
                while let Some(git_progress) = progress_rx.recv().await {
                    let progress = DownloadProgress {
                        phase: format!("{:?}", git_progress.phase),
                        current: git_progress.current,
                        total: git_progress.total,
                        message: git_progress.message,
                        model: None,
                    };

                    if sse_tx.send(progress).is_err() {
                        break;
                    }

                    // Check if we're complete or errored
                    match git_progress.phase {
                        crate::utils::git_service::GitPhase::Complete => break,
                        crate::utils::git_service::GitPhase::Error => break,
                        _ => {}
                    }
                }
            })
        };

        // Clone repository
        let clone_result = git_service
            .clone_repository(
                &repository_url,
                &repository_id,
                request.repository_branch.as_deref(),
                auth_token.as_deref(),
                progress_tx,
            )
            .await;

        // Wait for progress task to complete
        let _ = progress_task.await;

        match clone_result {
            Ok(cache_path) => {
                // Send committing progress
                let _ = sse_tx.send(DownloadProgress {
                    phase: "Committing".to_string(),
                    current: 90,
                    total: 100,
                    message: "Creating model from downloaded files...".to_string(),
                    model: None,
                });

                // Create storage and commit the repository files
                let storage_result = ModelStorage::new().await;
                match storage_result {
                    Ok(storage) => {
                        match create_model_with_files(
                            &storage,
                            provider_id,
                            name,
                            alias,
                            description,
                            file_format,
                            main_filename,
                            cache_path,
                            capabilities,
                            settings,
                        )
                        .await
                        {
                            Ok(model) => {
                                // Send final success with model
                                let _ = sse_tx.send(DownloadProgress {
                                    phase: "Complete".to_string(),
                                    current: 100,
                                    total: 100,
                                    message: "Model created successfully".to_string(),
                                    model: Some(model),
                                });
                            }
                            Err(e) => {
                                // Send error
                                let _ = sse_tx.send(DownloadProgress {
                                    phase: "Error".to_string(),
                                    current: 0,
                                    total: 100,
                                    message: format!("Failed to create model: {}", e),
                                    model: None,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Send storage error
                        let _ = sse_tx.send(DownloadProgress {
                            phase: "Error".to_string(),
                            current: 0,
                            total: 100,
                            message: format!("Storage initialization failed: {}", e),
                            model: None,
                        });
                    }
                }
            }
            Err(e) => {
                // Send download error
                let _ = sse_tx.send(DownloadProgress {
                    phase: "Error".to_string(),
                    current: 0,
                    total: 100,
                    message: format!("Download failed: {}", e),
                    model: None,
                });
            }
        }
    });

    // Create SSE stream
    let stream = UnboundedReceiverStream::new(sse_rx).map(|progress| {
        let json = serde_json::to_string(&progress).unwrap_or_else(|_| "{}".to_string());

        // Determine event type based on phase
        let event_type = match progress.phase.as_str() {
            "Complete" => "complete",
            "Error" => "error",
            _ => "progress",
        };

        Ok(Event::default().event(event_type).data(json))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
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

    // Index files (for sharded models)
    if filename_lower.contains("index") && filename_lower.ends_with(".json")
        || filename_lower == "pytorch_model.bin.index.json"
        || filename_lower == "model.safetensors.index.json"
        || filename_lower.ends_with(".index.json")
    {
        return ModelFileType::IndexFile;
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

// Hugging Face download functions removed - will be implemented later

/// File type classification for validation
#[derive(Debug, PartialEq)]
enum ModelFileType {
    WeightFile,    // .bin, .safetensors, .gguf, etc.
    IndexFile,     // index.json files for sharded models
    ConfigFile,    // config.json, config_*
    TokenizerFile, // tokenizer.json, tokenizer_config.json
    VocabFile,     // vocab.json, merges.txt, special_tokens_map.json
    UnknownFile,   // Everything else
}

impl std::fmt::Display for ModelFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFileType::WeightFile => write!(f, "weight"),
            ModelFileType::IndexFile => write!(f, "index"),
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
