use crate::database::models::ModelProviderModelDb;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

#[derive(Debug)]
pub struct ModelProcess {
    pub model_id: Uuid,
    pub provider_id: Uuid,
    pub port: u16,
    pub process: Child,
    pub status: ModelStatus,
    pub started_at: DateTime<Utc>,
    pub model_path: String,
    pub architecture: String,
}

pub struct ModelManager {
    running_models: Arc<Mutex<HashMap<Uuid, ModelProcess>>>,
    port_range: (u16, u16),
}

#[derive(Debug, thiserror::Error)]
pub enum ModelManagerError {
    #[error("Model not found: {0}")]
    ModelNotFound(Uuid),
    #[error("Model already running: {0}")]
    ModelAlreadyRunning(Uuid),
    #[error("No available ports in range {0}-{1}")]
    NoAvailablePorts(u16, u16),
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(String),
    #[error("Model startup timeout")]
    StartupTimeout,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lock file error: {0}")]
    LockFileError(String),
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            running_models: Arc::new(Mutex::new(HashMap::new())),
            port_range: (8001, 8999), // Port range for model servers
        }
    }

    /// Start a model server process
    pub async fn start_model(
        &self,
        model: &ModelProviderModelDb,
    ) -> Result<u16, ModelManagerError> {
        let mut running_models = self.running_models.lock().await;

        // Check if model is already running
        if running_models.contains_key(&model.id) {
            return Err(ModelManagerError::ModelAlreadyRunning(model.id));
        }

        // Find available port
        let port = self.find_available_port(&running_models).await?;

        // Get model path using the new pattern
        let model_path = model.get_model_path();
        let full_model_path = crate::APP_DATA_DIR.join(&model_path);

        // Check for existing lock file
        let lock_file = full_model_path.join(".model.lock");
        if lock_file.exists() {
            // Try to read the lock file and check if process is still running
            if let Ok(lock_content) = std::fs::read_to_string(&lock_file) {
                if let Some((pid_str, _port_str)) = lock_content.split_once(':') {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if process_is_running(pid) {
                            return Err(ModelManagerError::LockFileError(
                                "Model is already running".to_string(),
                            ));
                        }
                    }
                }
            }
            // Remove stale lock file
            if let Err(e) = std::fs::remove_file(&lock_file) {
                eprintln!("Warning: Could not remove stale lock file: {}", e);
            }
        }

        // Detect model format and get specific file paths
        let detected_architecture = match crate::ai::candle_models::ModelUtils::detect_model_format(&model_path) {
            Ok(arch) => arch,
            Err(e) => {
                eprintln!("Warning: Could not detect model format: {}, using default", e);
                "llama".to_string()
            }
        };
        
        // Use the detected architecture or fall back to the model's architecture
        let architecture = model.architecture.as_deref().unwrap_or(&detected_architecture);

        println!(
            "Starting model server for model {} on port {} (detected format: {})",
            model.id, port, architecture
        );

        // Get specific file paths for this model
        let file_paths = match crate::ai::candle_models::ModelUtils::get_model_file_paths(&model_path, architecture) {
            Ok(paths) => paths,
            Err(e) => {
                eprintln!("Warning: Could not get specific file paths: {}, using defaults", e);
                return Err(ModelManagerError::ProcessSpawnFailed(format!("Failed to get model file paths: {}", e)));
            }
        };

        // Log the files that will be used
        println!("Model files detected:");
        if let Some(ref config) = file_paths.config_file {
            println!("  Config: {}", config);
        }
        if let Some(ref tokenizer) = file_paths.tokenizer_file {
            println!("  Tokenizer: {}", tokenizer);
        }
        if !file_paths.weight_files.is_empty() {
            println!("  Weight files: {:?}", file_paths.weight_files);
        }

        // Get the path to the model-server binary
        let model_server_path = get_model_server_binary_path();

        // Build command with specific file paths
        let mut cmd = Command::new(model_server_path);
        cmd.arg("--model-path")
            .arg(&model_path)
            .arg("--architecture")
            .arg(architecture)
            .arg("--port")
            .arg(port.to_string())
            .arg("--model-id")
            .arg(model.id.to_string())
            .arg("--model-name")
            .arg(&model.name);

        // Add specific file paths if available
        if let Some(config_file) = &file_paths.config_file {
            if let Some(filename) = std::path::Path::new(config_file).file_name() {
                cmd.arg("--config-file").arg(filename.to_string_lossy().as_ref());
            }
        }

        if let Some(tokenizer_file) = &file_paths.tokenizer_file {
            if let Some(filename) = std::path::Path::new(tokenizer_file).file_name() {
                cmd.arg("--tokenizer-file").arg(filename.to_string_lossy().as_ref());
            }
        }

        if let Some(vocab_file) = &file_paths.vocab_file {
            if let Some(filename) = std::path::Path::new(vocab_file).file_name() {
                cmd.arg("--vocab-file").arg(filename.to_string_lossy().as_ref());
            }
        }

        if let Some(special_tokens_file) = &file_paths.special_tokens_file {
            if let Some(filename) = std::path::Path::new(special_tokens_file).file_name() {
                cmd.arg("--special-tokens-file").arg(filename.to_string_lossy().as_ref());
            }
        }

        // Add weight files
        if !file_paths.weight_files.is_empty() {
            // Use the first weight file as primary
            if let Some(filename) = std::path::Path::new(&file_paths.weight_files[0]).file_name() {
                cmd.arg("--weight-file").arg(filename.to_string_lossy().as_ref());
            }
            
            // Add additional weight files if there are more than one
            if file_paths.weight_files.len() > 1 {
                let additional_files: Vec<String> = file_paths.weight_files[1..]
                    .iter()
                    .filter_map(|f| std::path::Path::new(f).file_name())
                    .map(|f| f.to_string_lossy().to_string())
                    .collect();
                
                if !additional_files.is_empty() {
                    cmd.arg("--additional-weight-files").arg(additional_files.join(","));
                }
            }
        }

        // Spawn model server process
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ModelManagerError::ProcessSpawnFailed(e.to_string()))?;

        println!("Model server process spawned with PID: {:?}", child.id());

        // Wait for server to be ready
        let startup_result = self.wait_for_model_ready(port).await;

        match startup_result {
            Ok(()) => {
                // Server is ready, add to running models
                let model_process = ModelProcess {
                    model_id: model.id,
                    provider_id: model.provider_id,
                    port,
                    process: child,
                    status: ModelStatus::Running,
                    started_at: Utc::now(),
                    model_path: model_path.clone(),
                    architecture: architecture.to_string(),
                };

                running_models.insert(model.id, model_process);
                println!("Model {} started successfully on port {}", model.id, port);
                Ok(port)
            }
            Err(e) => {
                // Startup failed, kill the process and clean up
                eprintln!("Model startup failed: {}", e);
                let _ = child.kill();
                let _ = child.wait();

                // Clean up any files that might have been created
                self.cleanup_model_files(&full_model_path).await;

                Err(e)
            }
        }
    }

    /// Stop a model server process
    pub async fn stop_model(&self, model_id: Uuid) -> Result<(), ModelManagerError> {
        let mut running_models = self.running_models.lock().await;

        let mut model_process = running_models
            .remove(&model_id)
            .ok_or(ModelManagerError::ModelNotFound(model_id))?;

        println!("Stopping model {} on port {}", model_id, model_process.port);

        // Try graceful shutdown first
        let shutdown_result = self.request_graceful_shutdown(model_process.port).await;

        if shutdown_result.is_ok() {
            // Wait a bit for graceful shutdown
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Force kill if still running
        match model_process.process.try_wait() {
            Ok(Some(_)) => {
                println!("Model {} stopped gracefully", model_id);
            }
            Ok(None) => {
                println!("Force killing model {} process", model_id);
                if let Err(e) = model_process.process.kill() {
                    eprintln!("Failed to kill process: {}", e);
                }
                let _ = model_process.process.wait();
            }
            Err(e) => {
                eprintln!("Error checking process status: {}", e);
            }
        }

        // Clean up model files
        let full_model_path = crate::APP_DATA_DIR.join(&model_process.model_path);
        self.cleanup_model_files(&full_model_path).await;

        println!("Model {} stopped and cleaned up", model_id);
        Ok(())
    }

    /// Get the status of a running model
    pub async fn get_model_status(&self, model_id: Uuid) -> Option<ModelStatus> {
        let running_models = self.running_models.lock().await;
        running_models.get(&model_id).map(|p| p.status.clone())
    }

    /// List all running models
    pub async fn list_running_models(&self) -> Vec<(Uuid, u16, ModelStatus)> {
        let running_models = self.running_models.lock().await;
        running_models
            .values()
            .map(|p| (p.model_id, p.port, p.status.clone()))
            .collect()
    }

    /// Check if a model is running
    pub async fn is_model_running(&self, model_id: Uuid) -> bool {
        let running_models = self.running_models.lock().await;
        running_models.contains_key(&model_id)
    }

    /// Get the port for a running model
    pub async fn get_model_port(&self, model_id: Uuid) -> Option<u16> {
        let running_models = self.running_models.lock().await;
        running_models.get(&model_id).map(|p| p.port)
    }

    /// Stop all running models (for shutdown)
    pub async fn stop_all_models(&self) -> Result<(), ModelManagerError> {
        let model_ids: Vec<Uuid> = {
            let running_models = self.running_models.lock().await;
            running_models.keys().cloned().collect()
        };

        for model_id in model_ids {
            if let Err(e) = self.stop_model(model_id).await {
                eprintln!("Error stopping model {}: {}", model_id, e);
            }
        }

        Ok(())
    }

    // Private helper methods

    async fn find_available_port(
        &self,
        running_models: &HashMap<Uuid, ModelProcess>,
    ) -> Result<u16, ModelManagerError> {
        let used_ports: std::collections::HashSet<u16> =
            running_models.values().map(|p| p.port).collect();

        for port in self.port_range.0..=self.port_range.1 {
            if !used_ports.contains(&port) && port_is_available(port).await {
                return Ok(port);
            }
        }

        Err(ModelManagerError::NoAvailablePorts(
            self.port_range.0,
            self.port_range.1,
        ))
    }

    async fn wait_for_model_ready(&self, port: u16) -> Result<(), ModelManagerError> {
        let client = reqwest::Client::new();
        let health_url = format!("http://localhost:{}/health", port);
        let max_attempts = 30; // 30 seconds timeout
        let delay = Duration::from_secs(1);

        for attempt in 1..=max_attempts {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("Model server ready on port {} (attempt {})", port, attempt);
                    return Ok(());
                }
                Ok(response) => {
                    println!(
                        "Health check failed with status {}: attempt {}",
                        response.status(),
                        attempt
                    );
                }
                Err(e) => {
                    println!("Health check error (attempt {}): {}", attempt, e);
                }
            }

            if attempt < max_attempts {
                tokio::time::sleep(delay).await;
            }
        }

        Err(ModelManagerError::StartupTimeout)
    }

    async fn request_graceful_shutdown(&self, port: u16) -> Result<(), ModelManagerError> {
        let client = reqwest::Client::new();
        let shutdown_url = format!("http://localhost:{}/shutdown", port);

        match client.post(&shutdown_url).send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Graceful shutdown request failed: {}", e);
                Ok(()) // Don't fail if graceful shutdown doesn't work
            }
        }
    }

    async fn cleanup_model_files(&self, model_path: &std::path::Path) {
        let files_to_remove = [".model.lock", ".model.pid", ".model.port"];

        for file in &files_to_remove {
            let file_path = model_path.join(file);
            if file_path.exists() {
                if let Err(e) = std::fs::remove_file(&file_path) {
                    eprintln!("Warning: Could not remove {}: {}", file_path.display(), e);
                }
            }
        }
    }
}

// Utility functions

async fn port_is_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

fn process_is_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

fn get_model_server_binary_path() -> std::path::PathBuf {
    // In development, the binary is in target/debug/
    // In production, it should be alongside the main binary
    let current_exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./"));
    let exe_dir = current_exe.parent().unwrap_or(std::path::Path::new("./"));

    // Try different possible locations
    let possible_paths = [
        exe_dir.join("model-server"),
        exe_dir.join("model-server.exe"),
        std::path::PathBuf::from("./target/debug/model-server"),
        std::path::PathBuf::from("./target/debug/model-server.exe"),
        std::path::PathBuf::from("./target/release/model-server"),
        std::path::PathBuf::from("./target/release/model-server.exe"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Fallback to just "model-server" and hope it's in PATH
    std::path::PathBuf::from("model-server")
}

// Global model manager instance
lazy_static::lazy_static! {
    static ref MODEL_MANAGER: ModelManager = ModelManager::new();
}

pub fn get_model_manager() -> &'static ModelManager {
    &MODEL_MANAGER
}
