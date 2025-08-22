use reqwest;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};
use tokio::time::Duration;
use uuid::Uuid;

// Import the engine abstraction
use crate::ai::engines::llamacpp::LlamaCppEngine;
use crate::ai::engines::mistralrs::MistralRsEngine;
use crate::ai::engines::LocalEngine;

// Import AI provider types
use crate::ai::{
    core::AIProvider,
    providers::{
        anthropic::AnthropicProvider, custom::CustomProvider, deepseek::DeepSeekProvider,
        gemini::GeminiProvider, groq::GroqProvider, huggingface::HuggingFaceProvider,
        local::LocalProvider, mistral::MistralProvider, openai::OpenAIProvider,
    },
};
use crate::database::queries::models::get_model_by_id;
use crate::utils::proxy::create_proxy_config;

// Macro to create standard providers with common parameters
macro_rules! create_standard_provider {
    ($provider_type:ident, $provider:expr, $proxy_config:expr) => {
        {
            let provider_instance = $provider_type::new(
                $provider.api_key.as_ref().unwrap_or(&String::new()).clone(),
                $provider.base_url.clone(),
                $proxy_config,
                $provider.id,
            )?;
            Ok(Box::new(provider_instance))
        }
    };
}

// Structure to hold process information
#[derive(Debug)]
struct ModelProcess {
    child: Child,
    pid: u32,
    port: u16,
}

// Global registry to track running model processes with their child handles
static MODEL_REGISTRY: std::sync::LazyLock<Arc<RwLock<HashMap<Uuid, ModelProcess>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

// Global mutex for all model starting operations to prevent race conditions
static GLOBAL_MODEL_START_MUTEX: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

/// Acquire the global model start mutex to prevent race conditions
pub async fn acquire_global_start_mutex() -> tokio::sync::MutexGuard<'static, ()> {
    GLOBAL_MODEL_START_MUTEX.lock().await
}

#[derive(Debug, Clone)]
pub enum ModelStartResult {
    Started {
        port: u16,
        pid: u32,
    },
    AlreadyRunning {
        port: u16,
        pid: u32,
    },
    Failed {
        error: String,
        stdout_stderr_log_path: String,
    },
}

/// Check if a process is running by PID (enhanced version)
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        match Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        match Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
}

/// Verify that the server at the given port is running the expected model
async fn verify_model_uuid_match(
    port: u16,
    expected_model_id: &Uuid,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let models_url = format!("http://127.0.0.1:{}/v1/models", port);

    // Short timeout for this check
    let response =
        tokio::time::timeout(Duration::from_secs(5), client.get(&models_url).send()).await??;

    if !response.status().is_success() {
        return Err(format!("Models request failed with status: {}", response.status()).into());
    }

    let models_response: serde_json::Value = response.json().await?;

    // Extract models data from response
    let models_data = models_response
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("No data array in models response")?;

    // Check if any model ID contains or matches the expected model UUID
    let expected_uuid_str = expected_model_id.to_string();
    for model in models_data {
        if let Some(model_id) = model.get("id").and_then(|v| v.as_str()) {
            // Check if model ID exactly matches UUID or contains UUID (for path-based IDs)
            if model_id == expected_uuid_str || model_id.contains(&expected_uuid_str) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Robust multi-stage verification that a model server is running correctly
pub async fn verify_model_server_running(model_id: &Uuid) -> Option<(u32, u16)> {
    // Stage 1: Get PID and port from database
    let (pid, port) = match crate::database::queries::models::get_model_runtime_info(model_id).await
    {
        Ok(Some((pid, port))) => (pid as u32, port as u16),
        Ok(None) => {
            println!("Model {} has no runtime info in database", model_id);
            return None;
        }
        Err(e) => {
            eprintln!("Failed to get runtime info for model {}: {}", model_id, e);
            return None;
        }
    };

    // Stage 2: Check if PID exists in system
    if !is_process_running(pid) {
        println!("Process {} for model {} is not running", pid, model_id);
        // Clean up stale database entry
        let _ = crate::database::queries::models::update_model_runtime_info(
            model_id, None, None, false,
        )
        .await;
        return None;
    }

    // Stage 3: Request models to verify model UUID match
    match verify_model_uuid_match(port, model_id).await {
        Ok(true) => {
            println!(
                "Model {} verified running on PID {} port {} with correct UUID",
                model_id, pid, port
            );
            Some((pid, port))
        }
        Ok(false) => {
            println!(
                "Model {} PID {} port {} is running different model UUID",
                model_id, pid, port
            );
            // Clean up incorrect database entry
            let _ = crate::database::queries::models::update_model_runtime_info(
                model_id, None, None, false,
            )
            .await;
            None
        }
        Err(e) => {
            println!(
                "Failed to verify model {} at port {}: {}",
                model_id, port, e
            );
            // Server might be starting up or unhealthy, don't clean database yet
            None
        }
    }
}

pub async fn start_model_with_engine(
    model_id: &Uuid,
    model: &crate::database::models::model::Model,
) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
    // Create the appropriate engine based on model's engine_type
    let engine: Box<dyn LocalEngine> = match model.engine_type {
        crate::api::engines::EngineType::Mistralrs => Box::new(MistralRsEngine::new()),
        crate::api::engines::EngineType::Llamacpp => Box::new(LlamaCppEngine::new()),
        crate::api::engines::EngineType::None => {
            return Err("Cannot start local engine for remote model (engine_type: None)".into());
        }
    };

    // Start the engine with the model
    match engine.start(model).await {
        Ok(instance) => {
            let port = instance.port;
            let pid = instance.pid.unwrap_or(0);

            // Register the instance in our registry
            if let Ok(_registry) = MODEL_REGISTRY.write() {
                // For now, we'll still track the Child process handle for backward compatibility
                // This will be refactored once we fully migrate to the engine system
                println!(
                    "Engine {} started model {} on PID {} port {}",
                    engine.name(),
                    model_id,
                    pid,
                    port
                );
            }

            Ok(ModelStartResult::Started { port, pid })
        }
        Err(e) => {
            let error_msg = format!("Engine {} failed to start model: {}", engine.name(), e);
            // Create a default log path for error reporting
            let stdout_stderr_log_path = {
                let log_dir = crate::get_app_data_dir().join("logs/models");
                if !log_dir.exists() {
                    std::fs::create_dir_all(&log_dir).ok();
                }
                log_dir
                    .join(format!("{}_engine.log", model_id))
                    .to_string_lossy()
                    .to_string()
            };
            Ok(ModelStartResult::Failed {
                error: error_msg,
                stdout_stderr_log_path,
            })
        }
    }
}

// Main function - uses engine abstraction
pub async fn start_model(
    model_id: &Uuid,
) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
    // Check if already running using process inspection
    if let Some((pid, port)) = verify_model_server_running(model_id).await {
        return Ok(ModelStartResult::AlreadyRunning { port, pid });
    }

    // Load the model from database to get engine information
    let model = crate::database::queries::models::get_model_by_id(*model_id)
        .await
        .map_err(|e| format!("Failed to load model from database: {}", e))?
        .ok_or_else(|| format!("Model {} not found", model_id))?;

    // Use the new engine-based start function
    start_model_with_engine(model_id, &model).await
}

pub async fn stop_model(
    model_id: &Uuid,
    pid: u32,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Stopping mistralrs-server with model_id: {}, PID: {}, port: {}",
        model_id, pid, port
    );

    // First try to get the child process from our registry and kill it properly
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        if let Some(mut model_process) = registry.remove(model_id) {
            println!("Found process in registry, attempting graceful shutdown");

            // Try to kill the child process gracefully
            match model_process.child.kill() {
                Ok(_) => {
                    println!(
                        "Successfully killed child process with model_id: {}",
                        model_id
                    );
                    // Clean up the child to prevent zombie processes
                    let _ = model_process.child.wait();
                }
                Err(e) => {
                    eprintln!("Failed to kill child process for model {}: {}", model_id, e);
                }
            }
        } else {
            println!("Process not found in registry for model_id: {}", model_id);
        }
    }

    // If registry approach fails, try to kill by PID directly
    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let pid = Pid::from_raw(pid as i32);
        match signal::kill(pid, Signal::SIGTERM) {
            Ok(_) => {
                println!("Sent SIGTERM to process {}", pid);

                // Give it a few seconds to shut down gracefully
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Check if still running, if so, use SIGKILL
                match signal::kill(pid, Signal::SIGKILL) {
                    Ok(_) => println!("Sent SIGKILL to process {}", pid),
                    Err(_) => {} // Process likely already terminated
                }
            }
            Err(e) => {
                eprintln!("Failed to send SIGTERM to process {}: {}", pid, e);
            }
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Successfully killed process {}", pid);
                } else {
                    eprintln!(
                        "Failed to kill process {}: {}",
                        pid,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to execute taskkill for process {}: {}", pid, e);
            }
        }
    }

    Ok(())
}

pub async fn check_and_cleanup_model(
    model_id: &Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if model is running and get its runtime info
    if let Some((pid, port)) = verify_model_server_running(model_id).await {
        // Model is running, stop it
        stop_model(model_id, pid, port).await?;
    } else {
        // Even if not running, clean up from registry in case of stale entries
        if let Ok(mut registry) = MODEL_REGISTRY.write() {
            if let Some(mut model_process) = registry.remove(model_id) {
                println!("Cleaning up stale registry entry for model {}", model_id);
                // Try to wait on the child process to clean up any zombies
                let _ = model_process.child.wait();
            }
        }
    }

    Ok(())
}

/// Clean up dead processes and prevent zombie processes
/// This should be called periodically to maintain process hygiene
pub async fn cleanup_dead_processes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        let mut dead_processes = Vec::new();

        // Check each process in the registry
        for (model_id, model_process) in registry.iter_mut() {
            // Try to check if the child process has exited without blocking
            match model_process.child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    dead_processes.push(*model_id);
                }
                Ok(None) => {
                    // Process is still running - no action needed
                }
                Err(e) => {
                    eprintln!(
                        "Error checking process status for model {}: {}",
                        model_id, e
                    );
                    dead_processes.push(*model_id);
                }
            }
        }

        // Remove dead processes from registry
        for dead_id in dead_processes {
            if let Some(mut dead_process) = registry.remove(&dead_id) {
                println!("Cleaning up dead process for model {}", dead_id);
                // Try to wait on it to fully clean up (should be instant since it's already dead)
                let _ = dead_process.child.wait();
            }
        }
    }

    Ok(())
}

/// Reconcile database model states with actual running processes on startup
pub async fn reconcile_model_states() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting model state reconciliation...");

    // Get all models marked as active in database
    let active_models = match crate::database::queries::models::get_all_active_models().await {
        Ok(models) => models,
        Err(e) => {
            eprintln!("Failed to query active models: {}", e);
            return Err(e.into());
        }
    };

    if active_models.is_empty() {
        println!("No active models found in database");
        return Ok(());
    }

    println!(
        "Found {} models marked as active in database",
        active_models.len()
    );

    let mut reconciled_count = 0;
    let mut errors = Vec::new();

    for model in active_models {
        println!("Checking model {} ({})", model.name, model.id);

        // Verify if the model is actually running
        match verify_model_server_running(&model.id).await {
            Some((pid, port)) => {
                // Model is running and verified
                println!(
                    "Model {} is correctly running on PID {} port {}",
                    model.id, pid, port
                );

                // Ensure database has correct runtime info
                if model.pid != Some(pid as i32) || model.port != Some(port as i32) {
                    println!("Updating database runtime info for model {}", model.id);
                    if let Err(e) = crate::database::queries::models::update_model_runtime_info(
                        &model.id,
                        Some(pid as i32),
                        Some(port as i32),
                        true,
                    )
                    .await
                    {
                        eprintln!(
                            "Failed to update runtime info for model {}: {}",
                            model.id, e
                        );
                        errors.push(format!("Update runtime info for {}: {}", model.id, e));
                    } else {
                        reconciled_count += 1;
                    }
                }
            }
            None => {
                // Model is not running or verification failed
                println!(
                    "Model {} is marked active but not running, cleaning database state",
                    model.id
                );

                // Clear database state
                if let Err(e) = crate::database::queries::models::update_model_runtime_info(
                    &model.id, None, None, false,
                )
                .await
                {
                    eprintln!("Failed to clear model {} state: {}", model.id, e);
                    errors.push(format!("Clear state for {}: {}", model.id, e));
                } else {
                    println!("Successfully cleared state for model {}", model.id);
                    reconciled_count += 1;
                }
            }
        }
    }

    if errors.is_empty() {
        println!(
            "Model state reconciliation completed successfully. Reconciled {} models.",
            reconciled_count
        );
        Ok(())
    } else {
        let error_msg = format!(
            "Model reconciliation completed with {} errors: {}",
            errors.len(),
            errors.join("; ")
        );
        eprintln!("{}", error_msg);
        Err(error_msg.into())
    }
}

/// Shutdown all running models gracefully
pub async fn shutdown_all_models() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting graceful shutdown of all models...");

    let mut models_to_stop = Vec::new();
    let mut shutdown_errors = Vec::new();

    // 1. Collect models from MODEL_REGISTRY
    if let Ok(registry) = MODEL_REGISTRY.read() {
        for (model_id, model_process) in registry.iter() {
            models_to_stop.push((*model_id, model_process.pid, model_process.port));
        }
        println!("Found {} models in registry", models_to_stop.len());
    }

    // 2. Get additional active models from database (in case registry is out of sync)
    match crate::database::queries::models::get_all_active_models().await {
        Ok(db_models) => {
            for model in db_models {
                if let (Some(pid), Some(port)) = (model.pid, model.port) {
                    let model_info = (model.id, pid as u32, port as u16);
                    // Only add if not already in registry
                    if !models_to_stop.iter().any(|(id, _, _)| *id == model.id) {
                        models_to_stop.push(model_info);
                        println!("Added model {} from database", model.id);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to query database for active models: {}", e);
            shutdown_errors.push(format!("Database query failed: {}", e));
        }
    }

    if models_to_stop.is_empty() {
        println!("No models to shutdown");
        return Ok(());
    }

    println!("Stopping {} models...", models_to_stop.len());

    // 3. Stop each model
    for (model_id, pid, port) in models_to_stop {
        println!("Stopping model {} (PID: {}, Port: {})", model_id, pid, port);

        match stop_model(&model_id, pid, port).await {
            Ok(()) => {
                // Update database state
                if let Err(e) = crate::database::queries::models::update_model_runtime_info(
                    &model_id, None, None, false,
                )
                .await
                {
                    eprintln!(
                        "Failed to update database for stopped model {}: {}",
                        model_id, e
                    );
                    shutdown_errors.push(format!("Database update for {}: {}", model_id, e));
                } else {
                    println!("Successfully stopped model {}", model_id);
                }
            }
            Err(e) => {
                eprintln!("Failed to stop model {}: {}", model_id, e);
                shutdown_errors.push(format!("Stop model {}: {}", model_id, e));
            }
        }
    }

    // 4. Clear MODEL_REGISTRY
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        registry.clear();
        println!("Cleared model registry");
    }

    // 5. Final cleanup - ensure no zombie processes
    if let Err(e) = cleanup_dead_processes().await {
        eprintln!("Failed to cleanup dead processes: {}", e);
        shutdown_errors.push(format!("Cleanup dead processes: {}", e));
    }

    if shutdown_errors.is_empty() {
        println!("All models shutdown successfully");
        Ok(())
    } else {
        let error_msg = format!(
            "Model shutdown completed with {} errors: {}",
            shutdown_errors.len(),
            shutdown_errors.join("; ")
        );
        eprintln!("{}", error_msg);
        // Return Ok to allow app shutdown to continue
        Ok(())
    }
}

/// Helper function to create AI provider instances with optional model ID for Candle providers
pub async fn create_ai_provider_with_model_id(
    provider: &crate::database::models::Provider,
    model_id: Option<Uuid>,
) -> Result<Box<dyn AIProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let proxy_config = provider
        .proxy_settings
        .as_ref()
        .and_then(create_proxy_config);

    match provider.provider_type.as_str() {
        "openai" => create_standard_provider!(OpenAIProvider, provider, proxy_config),
        "anthropic" => create_standard_provider!(AnthropicProvider, provider, proxy_config),
        "groq" => create_standard_provider!(GroqProvider, provider, proxy_config),
        "gemini" => create_standard_provider!(GeminiProvider, provider, proxy_config),
        "mistral" => create_standard_provider!(MistralProvider, provider, proxy_config),
        "deepseek" => create_standard_provider!(DeepSeekProvider, provider, proxy_config),
        "custom" => create_standard_provider!(CustomProvider, provider, proxy_config),
        "huggingface" => create_standard_provider!(HuggingFaceProvider, provider, proxy_config),
        "local" => {
            let model_id = model_id.ok_or("Model ID is required for local providers")?;

            // Get model from database
            let model = match get_model_by_id(model_id).await {
                Ok(Some(model)) => model,
                Ok(None) => return Err("Model not found".into()),
                Err(e) => {
                    eprintln!("Failed to get model {}: {}", model_id, e);
                    return Err("Database operation failed".into());
                }
            };

            // Multi-stage verification if model is running correctly
            let port = match verify_model_server_running(&model_id).await {
                Some((_pid, port)) => {
                    println!("Model {} verified running on port {}", model_id, port);

                    // Register access for auto-unload tracking
                    crate::ai::register_model_access(&model_id).await;

                    port
                }
                None => {
                    // Model not running or verification failed, auto-start using protected logic
                    println!(
                        "Auto-starting model {} for chat request (with global mutex protection)",
                        model_id
                    );

                    match crate::ai::start_model_core_protected(model_id, &model, provider).await {
                        Ok((_pid, port)) => {
                            // Register access for auto-unload tracking
                            crate::ai::register_model_access(&model_id).await;
                            port
                        }
                        Err(e) => {
                            return Err(format!("Failed to auto-start model: {}", e).into());
                        }
                    }
                }
            };

            // Create the Local provider with the model's port and name (no proxy for local connections)
            let local_provider = LocalProvider::new(port, model.name.clone(), provider.id)?;

            Ok(Box::new(local_provider))
        }
        _ => Err(format!(
            "Unsupported provider type: {}",
            provider.provider_type.as_str()
        )
        .into()),
    }
}
