use axum::{
    extract::Path,
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::ai::DeviceType;
use crate::api::errors::{ApiResult, AppError, ErrorCode};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{CreateModelRequest, Model, UpdateModelRequest},
    queries::{models, providers, user_group_providers},
};

/// Build ModelStartParams from a model and its settings
pub fn build_model_start_params_from_model(model: &Model) -> crate::ai::ModelStartParams {
    let settings = model.get_settings();
    let device_ids = settings.device_ids.filter(|ids| !ids.is_empty());

    // Convert device_type from string to DeviceType enum
    let device_type = match settings.device_type.as_deref() {
        Some("cpu") => DeviceType::Cpu,
        Some("cuda") => DeviceType::Cuda,
        Some("metal") => DeviceType::Metal,
        _ => DeviceType::Cpu, // Default to CPU if not specified or unknown
    };

    // Create ModelStartParams from model settings
    let mut params = crate::ai::ModelStartParams::default();
    params.model_path = model.get_model_absolute_path();
    params.device_type = device_type;
    params.device_ids = device_ids;

    // Set model type based on architecture or use run (auto-loader) as default
    params.command = "run".to_string();

    // Apply settings from model configuration (only if specified)
    params.max_seqs = settings.max_seqs;
    params.max_seq_len = settings.max_seq_len;
    params.no_kv_cache = settings.no_kv_cache.unwrap_or(false);
    params.truncate_sequence = settings.truncate_sequence.unwrap_or(false);

    // PagedAttention settings
    params.paged_attn_gpu_mem = settings.paged_attn_gpu_mem;
    params.paged_attn_gpu_mem_usage = settings.paged_attn_gpu_mem_usage;
    params.paged_ctxt_len = settings.paged_ctxt_len;
    params.paged_attn_block_size = settings.paged_attn_block_size;
    params.no_paged_attn = settings.no_paged_attn.unwrap_or(false);
    params.paged_attn = settings.paged_attn.unwrap_or(false);

    // Performance settings
    params.prefix_cache_n = settings.prefix_cache_n;
    params.prompt_chunksize = settings.prompt_chunksize;

    // Model configuration
    params.dtype = settings.dtype.clone();
    params.in_situ_quant = settings.in_situ_quant.clone();
    params.seed = settings.seed;

    // Vision parameters
    params.max_edge = settings.max_edge;
    params.max_num_images = settings.max_num_images;
    params.max_image_length = settings.max_image_length;

    params
}

/// Start a model and update database - reusable core logic
pub async fn start_model_core(
    model_id: Uuid,
    model: &Model,
    provider: &crate::database::models::Provider,
) -> Result<(u32, u16), AppError> {
    // Validate provider type
    if provider.provider_type != "local" {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Only local models can be started",
        ));
    }

    // Check if model is actually running using robust verification
    if let Some((pid, port)) = crate::ai::verify_model_server_running(&model_id).await {
        println!(
            "Model {} is already running on PID {} port {}, updating database",
            model_id, pid, port
        );

        // Update model runtime info (PID and port)
        crate::database::queries::models::update_model_runtime_info(
            &model_id,
            Some(pid as i32),
            Some(port as i32),
            true, // Set is_active to true
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to update model {} runtime info: {}", model_id, e);
            AppError::internal_error("Database operation failed")
        })?;

        return Ok((pid, port));
    }

    // Validate that the model files exist
    let model_path = model.get_model_path();
    if !crate::ai::models::ModelUtils::model_exists(&model_path) {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Model files not found or invalid",
        ));
    }

    // Build start parameters from model settings
    let params = build_model_start_params_from_model(model);

    // Start the model server process
    match crate::ai::start_model(&model_id, params).await {
        Ok(crate::ai::ModelStartResult::Started { port, pid }) => {
            println!("Model {} started successfully on port {}", model_id, port);

            // Update model runtime info in database
            crate::database::queries::models::update_model_runtime_info(
                &model_id,
                Some(pid as i32),
                Some(port as i32),
                true,
            )
            .await
            .map_err(|e| {
                eprintln!("Failed to update model {} runtime info: {}", model_id, e);
                // If database update fails, try to stop the model to avoid orphaned processes
                let _ = tokio::spawn(async move {
                    if let Err(stop_err) = crate::ai::stop_model(&model_id, pid, port).await {
                        eprintln!("Also failed to stop orphaned model {}: {}", model_id, stop_err);
                    }
                });
                AppError::internal_error("Database operation failed")
            })?;

            Ok((pid, port))
        }
        Ok(crate::ai::ModelStartResult::AlreadyRunning { port, pid }) => {
            println!(
                "Model {} is already running on port {}, updating database status",
                model_id, port
            );

            // Update model runtime info in database
            crate::database::queries::models::update_model_runtime_info(
                &model_id,
                Some(pid as i32),
                Some(port as i32),
                true, // Set is_active to true
            )
            .await
            .map_err(|e| {
                eprintln!("Failed to update model {} runtime info: {}", model_id, e);
                AppError::internal_error("Database operation failed")
            })?;

            Ok((pid, port))
        }
        Ok(crate::ai::ModelStartResult::Failed { error, stdout_stderr_log_path }) => {
            eprintln!("Model {} failed to start: {}", model_id, error);
            eprintln!("Error logs available at: {}", stdout_stderr_log_path);
            
            // Read the log file contents
            let log_contents = match std::fs::read_to_string(&stdout_stderr_log_path) {
                Ok(contents) => {
                    if contents.trim().is_empty() {
                        "No output captured in log file.".to_string()
                    } else {
                        contents
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read log file {}: {}", stdout_stderr_log_path, e);
                    format!("Could not read log file: {}", e)
                }
            };
            
            Err(AppError::new(
                ErrorCode::SystemInternalError,
                format!("Failed to start model: {}\n\n--- Process Output ---\n{}", error, log_contents),
            ))
        }
        Err(e) => {
            eprintln!("Failed to start model {}: {}", model_id, e);
            Err(AppError::new(
                ErrorCode::SystemInternalError,
                format!("Failed to start model: {}", e),
            ))
        }
    }
}

// Model endpoints
pub async fn create_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<CreateModelRequest>,
) -> ApiResult<Json<Model>> {
    match models::create_model(provider_id, request).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => {
            eprintln!("Failed to create model for provider {}: {}", provider_id, e);
            // Handle unique constraint violation for (provider_id, name)
            match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("models_provider_id_name_unique") => {
          Err(AppError::new(ErrorCode::ValidInvalidInput,
                            "A model with this ID already exists for this provider. Please use a different model ID."))
        }
        _ => Err(AppError::internal_error("Database operation failed"))
      }
        }
    }
}

pub async fn update_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UpdateModelRequest>,
) -> ApiResult<Json<Model>> {
    match models::update_model(model_id, request).await {
        Ok(Some(model)) => Ok(Json(model)),
        Ok(None) => Err(AppError::not_found("Resource")),
        Err(e) => {
            eprintln!("Failed to update model {}: {}", model_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

#[axum::debug_handler]
pub async fn delete_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model database record using proper database query
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err(AppError::not_found("Model")),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // Get the provider to check if it's a Candle provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err(AppError::not_found("Model provider")),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // If it's a Candle provider, handle model shutdown and file deletion
    if provider.provider_type == "local" {
        // First, stop the model if it's running
        println!(
            "Checking and cleaning up model {} before deletion",
            model_id
        );
        match crate::ai::check_and_cleanup_model(&model_id).await {
            Ok(()) => {
                println!("Successfully cleaned up model {} for deletion", model_id);
            }
            Err(e) => {
                eprintln!("Warning: Failed to cleanup model {}: {}", model_id, e);
                // Continue with deletion anyway
            }
        }

        // Delete the physical model files
        let model_path = model.get_model_path();
        let full_model_path = crate::get_app_data_dir().join(&model_path);

        println!(
            "Deleting Candle model files at: {}",
            full_model_path.display()
        );

        if full_model_path.exists() {
            match std::fs::remove_dir_all(&full_model_path) {
                Ok(()) => {
                    println!("Successfully deleted model files for model {}", model_id);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to delete model files for model {}: {}",
                        model_id, e
                    );
                    // Continue with database deletion even if file deletion fails
                    // This prevents orphaned database records
                }
            }
        } else {
            println!(
                "Model files not found at {}, skipping file deletion",
                full_model_path.display()
            );
        }
    }

    // Delete the model from the database
    match models::delete_model(model_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(AppError::not_found("Resource")),
        Err(e) => {
            eprintln!("Failed to delete model {} from database: {}", model_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

pub async fn get_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<Json<Model>> {
    match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => Ok(Json(model)),
        Ok(None) => Err(AppError::not_found("Resource")),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Start a Candle model
#[axum::debug_handler]
pub async fn start_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model from database
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err(AppError::not_found("Model")),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // Get the provider to check if it's a Candle provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err(AppError::not_found("Model provider")),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // Use the common start_model_core logic
    match start_model_core(model_id, &model, &provider).await {
        Ok((_pid, _port)) => {
            println!("Successfully updated model {} runtime info", model_id);
            Ok(StatusCode::OK)
        }
        Err(e) => Err(e),
    }
}

// Stop a Candle model
#[axum::debug_handler]
pub async fn stop_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model from database
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err(AppError::not_found("Model")),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // Get the provider to check if it's a Candle provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err(AppError::not_found("Model provider")),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    if provider.provider_type != "local" {
        return Err(AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Only Candle models can be stopped",
        ));
    }

    // Check if model is running
    if crate::ai::is_model_running(&model_id).await.is_none() {
        // Model is not running, but we should still update the database to ensure consistency
        println!(
            "Model {} is not running, updating database status and clearing port",
            model_id
        );

        let clear_port_result =
            crate::database::queries::models::update_model_runtime_info(
                &model_id, None, // Clear the port
                None, // Clear the PID
                false,
            )
            .await;

        return match clear_port_result {
            Ok(_) => {
                println!("Successfully cleared model {} port and status", model_id);
                Ok(StatusCode::OK)
            }
            Err(e) => {
                eprintln!("Failed to clear model {} port: {}", model_id, e);
                Err(AppError::internal_error("Database operation failed"))
            }
        };
    }

    // Get the PID and port from the database for this specific model
    let runtime_info =
        match crate::database::queries::models::get_model_runtime_info(&model_id)
            .await
        {
            Ok(Some((pid, port))) => (pid as u32, port as u16),
            Ok(None) => {
                println!(
                    "Model {} has no runtime info, may already be stopped",
                    model_id
                );
                return Ok(StatusCode::OK);
            }
            Err(e) => {
                eprintln!("Failed to get model runtime info: {}", e);
                return Err(AppError::internal_error("Database operation failed"));
            }
        };

    // Stop the model server process with specific PID and port
    match crate::ai::stop_model(&model_id, runtime_info.0, runtime_info.1).await {
        Ok(()) => {
            println!("Model {} stopped successfully", model_id);

            let clear_port_result =
                crate::database::queries::models::update_model_runtime_info(
                    &model_id, None,  // Clear the port
                    None,  // Clear the PID
                    false, // Set is_active to false
                )
                .await;

            match clear_port_result {
                Ok(_) => {
                    println!("Successfully cleared model {} port and status", model_id);
                    Ok(StatusCode::OK)
                }
                Err(e) => {
                    eprintln!("Failed to clear model {} port: {}", model_id, e);
                    Err(AppError::internal_error("Database operation failed"))
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to stop model {}: {}", model_id, e);
            Err(AppError::new(
                crate::api::errors::ErrorCode::SystemInternalError,
                format!("Failed to stop model: {}", e),
            ))
        }
    }
}

// Enable a model
pub async fn enable_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match models::update_model(
        model_id,
        UpdateModelRequest {
            name: None,
            alias: None,
            description: None,
            parameters: None,
            enabled: Some(true),
            is_active: None,
            capabilities: None,
            settings: None,
        },
    )
    .await
    {
        Ok(Some(_)) => Ok(StatusCode::OK),
        Ok(None) => Err(AppError::not_found("Model")),
        Err(e) => {
            eprintln!("Failed to enable model {}: {}", model_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Disable a model
pub async fn disable_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    match models::update_model(
        model_id,
        UpdateModelRequest {
            name: None,
            alias: None,
            description: None,
            parameters: None,
            enabled: Some(false),
            is_active: None,
            capabilities: None,
            settings: None,
        },
    )
    .await
    {
        Ok(Some(_)) => Ok(StatusCode::OK),
        Ok(None) => Err(AppError::not_found("Model")),
        Err(e) => {
            eprintln!("Failed to disable model {}: {}", model_id, e);
            Err(AppError::internal_error("Database operation failed"))
        }
    }
}

// Base function for listing models for a provider with filtering
async fn list_provider_models_base(
    auth_user: &AuthenticatedUser,
    provider_id: Uuid,
    enabled_only: bool,
) -> ApiResult<Vec<Model>> {
    // First verify the user has access to this provider
    let user_providers = match user_group_providers::get_providers_for_user(auth_user.user.id).await
    {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!(
                "Failed to get model providers for user {}: {}",
                auth_user.user.id, e
            );
            return Err(e.into());
        }
    };

    // Check if user has access to this provider
    if !user_providers.iter().any(|p| p.id == provider_id) {
        return Err(AppError::new(
            ErrorCode::AuthzInsufficientPermissions,
            "Access denied to this model provider",
        ));
    }

    // Get models for the provider
    let models = match models::get_models_by_provider_id(provider_id).await {
        Ok(models) => models,
        Err(e) => {
            eprintln!("Failed to get models for provider {}: {}", provider_id, e);
            return Err(AppError::internal_error("Database operation failed"));
        }
    };

    // Apply enabled_only filter if requested
    let filtered_models = if enabled_only {
        models.into_iter().filter(|m| m.enabled).collect()
    } else {
        models
    };

    Ok(filtered_models)
}

/// List models for a specific provider
pub async fn list_provider_models(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Model>>> {
    let models = list_provider_models_base(&auth_user, provider_id, false).await?;
    Ok(Json(models))
}

/// List active models for a specific provider
pub async fn list_enabled_provider_models(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Model>>> {
    let models = list_provider_models_base(&auth_user, provider_id, true).await?;
    Ok(Json(models))
}