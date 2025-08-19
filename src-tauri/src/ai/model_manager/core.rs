use uuid::Uuid;
use crate::api::errors::{AppError, ErrorCode};
use crate::database::models::{Model, Provider};

/// Start a model with global mutex protection to prevent race conditions
pub async fn start_model_core_protected(
    model_id: Uuid,
    model: &Model,
    provider: &Provider,
) -> Result<(u32, u16), AppError> {
    // Acquire global mutex for all model starting operations
    let _guard = super::acquire_global_start_mutex().await;

    println!("Acquired global start mutex for model {}", model_id);

    // Re-check if model is running (someone else might have started it while we waited)
    if let Some((pid, port)) = super::verify_model_server_running(&model_id).await {
        println!(
            "Model {} already running after acquiring mutex, returning existing",
            model_id
        );
        return Ok((pid, port));
    }

    // Proceed with exclusive model starting
    start_model_core_internal(model_id, model, provider).await
}

/// Start a model and update database - internal implementation without mutex protection
async fn start_model_core_internal(
    model_id: Uuid,
    model: &Model,
    provider: &Provider,
) -> Result<(u32, u16), AppError> {
    // Validate provider type
    if provider.provider_type.as_str() != "local" {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Only local models can be started",
        ));
    }

    // Check if model is actually running using robust verification
    if let Some((pid, port)) = super::verify_model_server_running(&model_id).await {
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
    if !crate::ai::utils::models::ModelUtils::model_exists(&model_path) {
        return Err(AppError::new(
            ErrorCode::ValidInvalidInput,
            "Model files not found or invalid",
        ));
    }

    // Start the model server process (using engine settings from database)
    match super::start_model(&model_id).await {
        Ok(super::ModelStartResult::Started { port, pid }) => {
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
                    if let Err(stop_err) = super::stop_model(&model_id, pid, port).await {
                        eprintln!(
                            "Also failed to stop orphaned model {}: {}",
                            model_id, stop_err
                        );
                    }
                });
                AppError::internal_error("Database operation failed")
            })?;

            Ok((pid, port))
        }
        Ok(super::ModelStartResult::AlreadyRunning { port, pid }) => {
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
        Ok(super::ModelStartResult::Failed {
            error,
            stdout_stderr_log_path,
        }) => {
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
                format!(
                    "Failed to start model: {}\n\n--- Process Output ---\n{}",
                    error, log_contents
                ),
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