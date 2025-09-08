use axum::{debug_handler, extract::Path, http::StatusCode, Extension, Json};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError, ErrorCode};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{CreateModelRequest, Model, UpdateModelRequest},
    queries::{models, providers, user_group_providers},
};

// Model endpoints
#[debug_handler]
pub async fn create_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<CreateModelRequest>,
) -> ApiResult<Json<Model>> {
    match models::create_model(provider_id, request).await {
        Ok(model) => Ok((StatusCode::OK, Json(model))),
        Err(e) => {
            eprintln!("Failed to create model for provider {}: {}", provider_id, e);
            // Handle unique constraint violation for (provider_id, name)
            match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("models_provider_id_name_unique") => {
          Err((StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidInvalidInput,
                            "A model with this ID already exists for this provider. Please use a different model ID.")))
        }
        _ => Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))
      }
        }
    }
}

#[debug_handler]
pub async fn update_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
    Json(request): Json<UpdateModelRequest>,
) -> ApiResult<Json<Model>> {
    match models::update_model(model_id, request).await {
        Ok(Some(model)) => Ok((StatusCode::OK, Json(model))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Err(e) => {
            eprintln!("Failed to update model {}: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn delete_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model database record using proper database query
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model"))),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // Get the provider to check if it's a local provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model provider"))),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // If it's a local provider, handle model shutdown and file deletion
    if provider.provider_type.as_str() == "local" {
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
            "Deleting local model files at: {}",
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
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Err(e) => {
            eprintln!("Failed to delete model {} from database: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

#[debug_handler]
pub async fn get_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<Json<Model>> {
    match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => Ok((StatusCode::OK, Json(model))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Resource"))),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Start a local model
#[debug_handler]
pub async fn start_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model from database
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model"))),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // Get the provider to check if it's a local provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model provider"))),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    match crate::ai::start_model_core_protected(model_id, &model, &provider).await {
        Ok((_pid, _port)) => {
            println!("Successfully updated model {} runtime info", model_id);
            Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// Stop a local model
#[debug_handler]
pub async fn stop_model(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(model_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get the model from database
    let model = match models::get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model"))),
        Err(e) => {
            eprintln!("Failed to get model {}: {}", model_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    // Get the provider to check if it's a local provider
    let provider = match providers::get_provider_by_id(model.provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("Model provider"))),
        Err(e) => {
            eprintln!("Failed to get model provider: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
        }
    };

    if provider.provider_type.as_str() != "local" {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidInvalidInput,
                "Only local models can be stopped",
            ),
        ));
    }

    // Check if model is running
    if crate::ai::verify_model_server_running(&model_id)
        .await
        .is_none()
    {
        // Model is not running, but we should still update the database to ensure consistency
        println!(
            "Model {} is not running, updating database status and clearing port",
            model_id
        );

        let clear_port_result = crate::database::queries::models::update_model_runtime_info(
            &model_id, None, // Clear the port
            None, // Clear the PID
            false,
        )
        .await;

        return match clear_port_result {
            Ok(_) => {
                println!("Successfully cleared model {} port and status", model_id);
                Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
            }
            Err(e) => {
                eprintln!("Failed to clear model {} port: {}", model_id, e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Database operation failed"),
                ))
            }
        };
    }

    // Get the PID and port from the database for this specific model
    let runtime_info =
        match crate::database::queries::models::get_model_runtime_info(&model_id).await {
            Ok(Some((pid, port))) => (pid as u32, port as u16),
            Ok(None) => {
                println!(
                    "Model {} has no runtime info, may already be stopped",
                    model_id
                );
                return Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT));
            }
            Err(e) => {
                eprintln!("Failed to get model runtime info: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Database operation failed"),
                ));
            }
        };

    // Stop the model server process with specific PID and port
    match crate::ai::stop_model(&model_id, runtime_info.0, runtime_info.1).await {
        Ok(()) => {
            println!("Model {} stopped successfully", model_id);

            let clear_port_result = crate::database::queries::models::update_model_runtime_info(
                &model_id, None,  // Clear the port
                None,  // Clear the PID
                false, // Set is_active to false
            )
            .await;

            match clear_port_result {
                Ok(_) => {
                    println!("Successfully cleared model {} port and status", model_id);
                    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
                }
                Err(e) => {
                    eprintln!("Failed to clear model {} port: {}", model_id, e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Database operation failed"),
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to stop model {}: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::new(
                    crate::api::errors::ErrorCode::SystemInternalError,
                    format!("Failed to stop model: {}", e),
                ),
            ))
        }
    }
}

// Enable a model
#[debug_handler]
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
            engine_type: None,
            engine_settings: None,
            file_format: None,
        },
    )
    .await
    {
        Ok(Some(_)) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Model"))),
        Err(e) => {
            eprintln!("Failed to enable model {}: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Disable a model
#[debug_handler]
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
            engine_type: None,
            engine_settings: None,
            file_format: None,
        },
    )
    .await
    {
        Ok(Some(_)) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Model"))),
        Err(e) => {
            eprintln!("Failed to disable model {}: {}", model_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ))
        }
    }
}

// Base function for listing models for a provider with filtering
async fn list_provider_models_base(
    auth_user: &AuthenticatedUser,
    provider_id: Uuid,
    enabled_only: bool,
) -> Result<Vec<Model>, (StatusCode, AppError)> {
    // First verify the user has access to this provider
    let user_providers = match user_group_providers::get_providers_for_user(auth_user.user.id).await
    {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!(
                "Failed to get model providers for user {}: {}",
                auth_user.user.id, e
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user providers"),
            ));
        }
    };

    // Check if user has access to this provider
    if !user_providers.iter().any(|p| p.id == provider_id) {
        return Err((
            StatusCode::FORBIDDEN,
            AppError::new(
                ErrorCode::AuthzInsufficientPermissions,
                "Access denied to this model provider",
            ),
        ));
    }

    // Get models for the provider
    let models = match models::get_models_by_provider_id(provider_id).await {
        Ok(models) => models,
        Err(e) => {
            eprintln!("Failed to get models for provider {}: {}", provider_id, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            ));
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
#[debug_handler]
pub async fn list_provider_models(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Model>>> {
    let models = list_provider_models_base(&auth_user, provider_id, false).await?;
    Ok((StatusCode::OK, Json(models)))
}

/// List active models for a specific provider
#[debug_handler]
pub async fn list_enabled_provider_models(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Model>>> {
    let models = list_provider_models_base(&auth_user, provider_id, true).await?;
    Ok((StatusCode::OK, Json(models)))
}
