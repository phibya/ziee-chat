use axum::{response::Json, Extension};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use tokio::sync::mpsc;
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

/// Progress tracker for calculating speed and ETA
#[derive(Debug, Clone)]
struct ProgressTracker {
    start_time: std::time::Instant,
    last_update_time: std::time::Instant,
    last_bytes: u64,
}

impl ProgressTracker {
    fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time: now,
            last_update_time: now,
            last_bytes: 0,
        }
    }

    fn update(&mut self, current_bytes: u64) -> (Option<f64>, Option<u64>) {
        let now = std::time::Instant::now();

        // Calculate overall speed (bytes per second)
        let total_elapsed = now.duration_since(self.start_time).as_secs_f64();
        let overall_speed = if total_elapsed > 0.0 {
            current_bytes as f64 / total_elapsed
        } else {
            0.0
        };

        // Calculate recent speed for more responsive updates
        let recent_elapsed = now.duration_since(self.last_update_time).as_secs_f64();
        let recent_speed = if recent_elapsed > 1.0 {
            // Only calculate if at least 1 second elapsed
            let bytes_diff = current_bytes.saturating_sub(self.last_bytes) as f64;
            bytes_diff / recent_elapsed
        } else {
            overall_speed // Fall back to overall speed
        };

        // Use recent speed if it's reasonable, otherwise use overall speed
        let speed_bps = if recent_speed > 0.0 && recent_elapsed > 1.0 {
            recent_speed
        } else {
            overall_speed
        };

        // Update tracking state
        if recent_elapsed > 1.0 {
            self.last_update_time = now;
            self.last_bytes = current_bytes;
        }

        (Some(speed_bps), None) // ETA will be calculated separately
    }

    fn calculate_eta(
        &self,
        current_bytes: u64,
        total_bytes: u64,
        speed_bps: Option<f64>,
    ) -> Option<u64> {
        if let Some(speed) = speed_bps {
            if speed > 0.0 && total_bytes > current_bytes {
                let remaining_bytes = total_bytes - current_bytes;
                let eta_seconds = remaining_bytes as f64 / speed;
                Some(eta_seconds as u64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Serialize)]
pub struct DownloadProgress {
    pub phase: String,
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub model: Option<Model>,
}

/// Request struct for creating a model with files
#[derive(Debug)]
pub struct CreateModelWithFilesRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub file_format: String,
    pub main_filename: String,
    pub source_dir: PathBuf,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub settings: Option<ModelSettings>,
}

/// Shared model creation and file processing logic
async fn create_model_with_files(request: CreateModelWithFilesRequest) -> Result<Model, AppError> {
    // Initialize storage
    let storage = ModelStorage::new().await.map_err(|e| {
        AppError::internal_error(format!("Failed to initialize storage: {}", e))
    })?;
    // Validate provider exists and is of type 'local'
    let provider = crate::database::queries::providers::get_provider_by_id(request.provider_id)
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
    let model_name = request.name.clone();

    println!("Processing model with file format: {}", request.file_format);

    // Create storage directory
    storage
        .create_model_directory(&request.provider_id, &model_id)
        .await
        .map_err(|e| {
            AppError::internal_error(format!("Failed to create storage directory: {}", e))
        })?;

    // print source directory
    println!("Source directory for model files: {}", request.source_dir.display());

    // List all files in the source directory
    let source_files = match tokio::fs::read_dir(&request.source_dir).await {
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
    let files_to_copy = determine_files_to_copy(&source_files, &request.main_filename)?;

    if files_to_copy.is_empty() {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            format!(
                "No relevant files found for main filename: {}",
                request.main_filename
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
        let source_path = request.source_dir.join(filename);
        let dest_path = storage
            .get_model_path(&request.provider_id, &model_id)
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
        let relative_path = format!("models/{}/{}/{}", request.provider_id, model_id, filename);

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
    let create_request = crate::database::models::CreateModelRequest {
        provider_id: request.provider_id,
        name: request.name,
        alias: request.alias,
        description: request.description,
        enabled: Some(true), // Enable immediately since everything succeeded
        capabilities: request.capabilities.or_else(|| Some(ModelCapabilities::new())),
        parameters: request.parameters,
        settings: request.settings,
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

        let base_name = main_filename
            .trim_end_matches(".safetensors")
            .trim_end_matches(".bin")
            .trim_end_matches(".pt")
            .trim_end_matches(".pth");

        // Add all weight files that match the sharding pattern
        for file in source_files {
            if file.starts_with(base_name)
                && (file.contains("-of-") || file.contains("_of_"))
                && (file.ends_with(".safetensors")
                    || file.ends_with(".bin")
                    || file.ends_with(".pt")
                    || file.ends_with(".pth"))
            {
                files_to_copy.push(file.clone());
            }
        }
    } else if main_is_json
        && (main_filename.contains("index") || main_filename.ends_with(".index.json"))
    {
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
            if file.starts_with(&base_name)
                && file != main_filename
                && (file.ends_with(".safetensors")
                    || file.ends_with(".bin")
                    || file.ends_with(".pt")
                    || file.ends_with(".pth"))
            {
                files_to_copy.push(file.clone());
            }
        }
    } else if main_exists {
        // No index file found but main file exists - only copy the main weight file
        println!(
            "No index file found for {}. Only copying main file.",
            main_filename
        );
        files_to_copy.push(main_filename.to_string());
    } else {
        // Neither index file nor main file exists - throw error
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            format!(
                "Neither '{}' nor '{}' found in source directory",
                main_filename,
                if !main_is_json {
                    &index_filename
                } else {
                    main_filename
                }
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
    pub parameters: Option<ModelParameters>,
    pub settings: Option<ModelSettings>,
}

/// Initiate model download from repository (returns JSON with download ID immediately)
pub async fn initiate_repository_download(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<DownloadFromRepositoryRequest>,
) -> ApiResult<Json<DownloadInstance>> {
    // Get repository information
    let repository = repositories::get_repository_by_id(request.repository_id)
        .await
        .map_err(|e| AppError::internal_error(&e.to_string()))?
        .ok_or_else(|| AppError::not_found("Repository"))?;

    // Create download instance in the database
    let download_request = CreateDownloadInstanceRequest {
        provider_id: request.provider_id,
        repository_id: request.repository_id,
        request_data: DownloadRequestData {
            model_name: request.name.clone(),
            revision: request.repository_branch.clone(),
            files: None, // Download all files
            quantization: None,
            repository_path: Some(request.repository_path.clone()),
            alias: Some(request.alias.clone()),
            description: request.description.clone(),
            file_format: Some(request.file_format.clone()),
            main_filename: Some(request.main_filename.clone()),
            capabilities: request.capabilities.clone(),
            parameters: request.parameters.clone(),
            settings: request.settings.clone(),
        },
    };

    let download_instance =
        crate::database::queries::download_instances::create_download_instance(download_request)
            .await
            .map_err(|e| AppError::database_error(e))?;

    // Clone necessary data for the background task
    let download_id = download_instance.id;
    let repository_url =
        GitService::build_repository_url(&repository.url, &request.repository_path);
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
            if let Some(config) = &repository.auth_config {
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    Some(format!("{}:{}", username, password))
                } else {
                    None
                }
            } else {
                None
            }
        }
        "none" | _ => None,
    };

    // Spawn background task to handle the download
    tokio::spawn(async move {
        // Update status to downloading
        let _ = crate::database::queries::download_instances::update_download_status(
            download_id,
            UpdateDownloadStatusRequest {
                status: DownloadStatus::Downloading,
                error_message: None,
                model_id: None,
            },
        )
        .await;

        // Create progress channel
        let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<GitProgress>();

        // Create git service
        let git_service = GitService::new();

        // Spawn task to update download progress in database
        let download_id_progress = download_id;
        let progress_task = tokio::spawn(async move {
            let mut tracker = ProgressTracker::new();
            while let Some(git_progress) = progress_rx.recv().await {
                // Calculate speed and ETA
                let (speed_bps_f64, _) = tracker.update(git_progress.current);
                let speed_bps = speed_bps_f64.map(|s| s as i64);
                let eta_seconds = tracker
                    .calculate_eta(git_progress.current, git_progress.total, speed_bps_f64)
                    .map(|eta| eta as i64);

                let progress_data = DownloadProgressData {
                    phase: Some(format!("{:?}", git_progress.phase)),
                    current: Some(git_progress.current as i64),
                    total: Some(git_progress.total as i64),
                    message: Some(git_progress.message.clone()),
                    speed_bps,
                    eta_seconds,
                };

                let status = match git_progress.phase {
                    crate::utils::git_service::GitPhase::Error => Some(DownloadStatus::Failed),
                    _ => None,
                };

                let _ = crate::database::queries::download_instances::update_download_progress(
                    download_id_progress,
                    UpdateDownloadProgressRequest {
                        progress_data,
                        status,
                    },
                )
                .await;

                // Break on error phase
                if matches!(
                    git_progress.phase,
                    crate::utils::git_service::GitPhase::Error
                ) {
                    break;
                }
            }
        });

        println!(
            "Starting download for repository: {} (ID: {})",
            request.repository_path, request.repository_id
        );

        // Clone repository without LFS files
        let clone_result = git_service
            .clone_repository_without_lfs(
                &repository_url,
                &request.repository_id,
                request.repository_branch.as_deref(),
                auth_token.as_deref(),
                progress_tx.clone(),
            )
            .await;

        // Drop the progress sender to signal completion to the progress task
        drop(progress_tx);

        // Wait for progress task with timeout to ensure it processes any final messages
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), progress_task).await;

        println!("Clone result: {:?}", clone_result);

        match clone_result {
            Ok(cache_path) => {
                // Update progress: Analyzing files
                let _ = crate::database::queries::download_instances::update_download_progress(
                    download_id,
                    UpdateDownloadProgressRequest {
                        progress_data: DownloadProgressData {
                            phase: Some("Analyzing".to_string()),
                            current: Some(10),
                            total: Some(100),
                            message: Some("Analyzing repository files...".to_string()),
                            speed_bps: None,
                            eta_seconds: None,
                        },
                        status: None,
                    },
                )
                .await;

                // List files in the repository
                let source_files = match std::fs::read_dir(&cache_path) {
                    Ok(entries) => entries
                        .filter_map(|entry| {
                            entry
                                .ok()
                                .and_then(|e| e.file_name().to_str().map(|s| s.to_string()))
                        })
                        .filter(|name| !name.starts_with('.'))
                        .collect::<Vec<String>>(),
                    Err(e) => {
                        let _ =
                            crate::database::queries::download_instances::update_download_status(
                                download_id,
                                UpdateDownloadStatusRequest {
                                    status: DownloadStatus::Failed,
                                    error_message: Some(format!(
                                        "Failed to read repository directory: {}",
                                        e
                                    )),
                                    model_id: None,
                                },
                            )
                            .await;
                        return;
                    }
                };

                // Determine which files to copy
                let files_to_copy =
                    match determine_files_to_copy(&source_files, &request.main_filename) {
                        Ok(files) => files,
                        Err(e) => {
                            let _ =
                            crate::database::queries::download_instances::update_download_status(
                                download_id,
                                UpdateDownloadStatusRequest {
                                    status: DownloadStatus::Failed,
                                    error_message: Some(format!(
                                        "Failed to determine files to copy: {}",
                                        e
                                    )),
                                    model_id: None,
                                },
                            )
                            .await;
                            return;
                        }
                    };

                // Update progress: Downloading LFS files
                let _ = crate::database::queries::download_instances::update_download_progress(
                    download_id,
                    UpdateDownloadProgressRequest {
                        progress_data: DownloadProgressData {
                            phase: Some("Downloading".to_string()),
                            current: Some(20),
                            total: Some(100),
                            message: Some("Checking for LFS files...".to_string()),
                            speed_bps: None,
                            eta_seconds: None,
                        },
                        status: None,
                    },
                )
                .await;

                // Create new progress channel for LFS
                let (lfs_progress_tx, mut lfs_progress_rx) =
                    mpsc::unbounded_channel::<GitProgress>();

                // Spawn task to update LFS progress
                let download_id_lfs = download_id;
                let lfs_progress_task = tokio::spawn(async move {
                    let mut lfs_tracker = ProgressTracker::new();
                    while let Some(git_progress) = lfs_progress_rx.recv().await {
                        let (speed_bps_f64, _) = lfs_tracker.update(git_progress.current);
                        let speed_bps = speed_bps_f64.map(|s| s as i64);
                        let eta_seconds = lfs_tracker
                            .calculate_eta(git_progress.current, git_progress.total, speed_bps_f64)
                            .map(|eta| eta as i64);

                        let _ =
                            crate::database::queries::download_instances::update_download_progress(
                                download_id_lfs,
                                UpdateDownloadProgressRequest {
                                    progress_data: DownloadProgressData {
                                        phase: Some("Downloading LFS files".to_string()),
                                        current: Some(git_progress.current as i64),
                                        total: Some(git_progress.total as i64),
                                        message: Some(git_progress.message),
                                        speed_bps,
                                        eta_seconds,
                                    },
                                    status: None,
                                },
                            )
                            .await;
                    }
                });

                // Pull LFS files
                let lfs_result = git_service
                    .pull_lfs_files(
                        &cache_path,
                        &files_to_copy,
                        auth_token.as_deref(),
                        lfs_progress_tx,
                    )
                    .await;

                // Wait for LFS progress task with timeout (the sender is dropped by pull_lfs_files)
                let _ = tokio::time::timeout(std::time::Duration::from_secs(5), lfs_progress_task)
                    .await;

                // Check LFS result after progress task is done
                if let Err(e) = lfs_result {
                    let _ = crate::database::queries::download_instances::update_download_status(
                        download_id,
                        UpdateDownloadStatusRequest {
                            status: DownloadStatus::Failed,
                            error_message: Some(format!("Failed to download LFS files: {}", e)),
                            model_id: None,
                        },
                    )
                    .await;
                    return;
                }

                // Update progress: Creating model
                let _ = crate::database::queries::download_instances::update_download_progress(
                    download_id,
                    UpdateDownloadProgressRequest {
                        progress_data: DownloadProgressData {
                            phase: Some("Committing".to_string()),
                            current: Some(90),
                            total: Some(100),
                            message: Some("Creating model from downloaded files...".to_string()),
                            speed_bps: None,
                            eta_seconds: None,
                        },
                        status: None,
                    },
                )
                .await;

                // Create model with files
                match create_model_with_files(CreateModelWithFilesRequest {
                    provider_id: request.provider_id,
                    name: request.name,
                    alias: request.alias,
                    description: request.description,
                    file_format: request.file_format,
                    main_filename: request.main_filename,
                    source_dir: cache_path,
                    capabilities: request.capabilities,
                    parameters: request.parameters,
                    settings: request.settings,
                })
                .await
                {
                    Ok(model) => {
                        // Update download as completed with model ID
                        let _ = crate::database::queries::download_instances::update_download_status(
                            download_id,
                            UpdateDownloadStatusRequest {
                                status: DownloadStatus::Completed,
                                error_message: None,
                                model_id: Some(model.id),
                            },
                        )
                        .await;

                        // Spawn cleanup task to remove the download record after 60 seconds
                        // This gives clients time to see the completion status
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                            let _ = crate::database::queries::download_instances::delete_download_instance(download_id).await;
                        });
                    }
                    Err(e) => {
                        let _ = crate::database::queries::download_instances::update_download_status(
                            download_id,
                            UpdateDownloadStatusRequest {
                                status: DownloadStatus::Failed,
                                error_message: Some(format!("Failed to create model: {}", e)),
                                model_id: None,
                            },
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                // Create a more descriptive error message
                let error_msg = if e.to_string().contains("403")
                    || e.to_string().contains("HTTP status code: 403")
                {
                    format!("Access denied (403): Authentication failed or insufficient permissions. {}", e)
                } else if e.to_string().contains("401")
                    || e.to_string().contains("HTTP status code: 401")
                {
                    format!(
                        "Authentication required (401): Invalid or missing credentials. {}",
                        e
                    )
                } else {
                    format!("Download failed: {}", e)
                };

                let _ = crate::database::queries::download_instances::update_download_status(
                    download_id,
                    UpdateDownloadStatusRequest {
                        status: DownloadStatus::Failed,
                        error_message: Some(error_msg),
                        model_id: None,
                    },
                )
                .await;
            }
        }
    });

    // Return the download instance immediately
    Ok(Json(download_instance))
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
