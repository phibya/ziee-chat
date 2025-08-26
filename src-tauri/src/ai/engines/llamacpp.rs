use super::{EngineError, EngineInstance, EngineType, LocalEngine};
use crate::database::models::model::Model;
use crate::utils::resource_paths::ResourcePaths;
use std::fs::{metadata, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

pub struct LlamaCppEngine;

impl LlamaCppEngine {
    pub fn new() -> Self {
        LlamaCppEngine
    }

    fn get_available_port() -> u16 {
        portpicker::pick_unused_port().unwrap_or_else(|| {
            eprintln!("Warning: Could not find available port using portpicker, falling back to system allocation");
            // Fallback to system allocation if portpicker fails
            use std::net::TcpListener;
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        })
    }

    /// Calculate the total size of model files in bytes
    fn calculate_model_size(
        &self,
        model: &Model,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let model_path = model.get_model_absolute_path();
        let model_dir = Path::new(&model_path);
        let mut total_size = 0u64;
        let mut model_files = Vec::new();

        if model_dir.is_dir() {
            // If it's a directory, sum up all files recursively
            self.scan_model_files(model_dir, &mut total_size, &mut model_files)?;
        } else if model_dir.is_file() {
            // If it's a single file, get its size
            let file_size = metadata(model_dir)?.len();
            total_size = file_size;
            model_files.push((model_dir.to_path_buf(), file_size));
        }

        println!(
            "Found {} model file(s) with total size: {} bytes ({:.2} GB)",
            model_files.len(),
            total_size,
            total_size as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        Ok(total_size)
    }

    /// Recursively scan for model files in a directory
    fn scan_model_files(
        &self,
        dir: &Path,
        total_size: &mut u64,
        model_files: &mut Vec<(std::path::PathBuf, u64)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy().to_lowercase();

                    // Check for GGUF model files
                    if self.is_model_file(&path, &file_name_str) {
                        let file_size = metadata(&path)?.len();
                        *total_size += file_size;
                        model_files.push((path.clone(), file_size));

                        // Log individual files for debugging
                        let size_mb = file_size as f64 / (1024.0 * 1024.0);
                        if size_mb > 100.0 {
                            println!("Found model file: {} ({:.1} MB)", path.display(), size_mb);
                        }
                    }
                }
            } else if path.is_dir() {
                // Recursively check subdirectories
                self.scan_model_files(&path, total_size, model_files)?;
            }
        }
        Ok(())
    }

    /// Check if a file is a GGUF model file
    fn is_model_file(&self, path: &Path, file_name_lower: &str) -> bool {
        // Check by extension - LlamaCpp primarily uses GGUF format
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ext == "gguf" || ext == "ggml" || ext == "bin" {
                return true;
            }
        }

        // Check for common GGUF patterns
        if file_name_lower.contains("gguf") || file_name_lower.contains("ggml") {
            return true;
        }

        // Skip config files
        if file_name_lower == "params.json" || file_name_lower == "config.json" {
            return false;
        }

        false
    }

    /// Calculate timeout based on model size
    fn calculate_timeout_for_model_size(&self, model_size_bytes: u64) -> u64 {
        const BASE_TIMEOUT: u64 = 60; // 1 minute
        const SECONDS_PER_GB: u64 = 20; // 20 seconds per GB
        const MAX_TIMEOUT: u64 = 1200; // 20 minutes
        const MIN_TIMEOUT: u64 = 60; // 1 minute

        let size_gb = model_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let additional_time = (size_gb * SECONDS_PER_GB as f64) as u64;
        let calculated_timeout = BASE_TIMEOUT + additional_time;

        // Clamp between min and max
        let timeout = calculated_timeout.clamp(MIN_TIMEOUT, MAX_TIMEOUT);

        println!(
            "Model size: {:.2} GB, calculated timeout: {} seconds ({:.1} minutes)",
            size_gb,
            timeout,
            timeout as f64 / 60.0
        );

        timeout
    }

    /// Check if the model server is healthy (quick health check without timeout)
    async fn check_model_server_health(
        &self,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let health_url = format!("http://127.0.0.1:{}/health", port);
        let client = reqwest::Client::new();

        // Quick health check with short timeout
        let response = tokio::time::timeout(
            Duration::from_secs(3), // 3 second timeout
            client.get(&health_url).send(),
        )
        .await;

        match response {
            Ok(Ok(resp)) => {
                if resp.status().is_success() {
                    // Parse JSON response and check for {"status":"ok"}
                    let body = resp.text().await?;
                    let health_response: serde_json::Value = serde_json::from_str(&body)
                        .map_err(|e| format!("Failed to parse health check JSON: {}", e))?;

                    if let Some(status) = health_response.get("status") {
                        if status == "ok" {
                            Ok(())
                        } else {
                            Err(format!(
                                "Health check failed: status is '{}', expected 'ok'",
                                status
                            )
                            .into())
                        }
                    } else {
                        Err("Health check failed: response missing 'status' field".into())
                    }
                } else {
                    Err(format!("Health check failed with status: {}", resp.status()).into())
                }
            }
            Ok(Err(e)) => Err(format!("Health check request failed: {}", e).into()),
            Err(_) => Err("Health check timed out".into()),
        }
    }

    /// Wait for the model server to be healthy and ready (with timeout for startup)
    async fn wait_for_model_health(
        &self,
        instance: &EngineInstance,
        timeout_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let health_url = format!("http://127.0.0.1:{}/health", instance.port);
        let models_url = format!("http://127.0.0.1:{}/v1/models", instance.port);

        let client = reqwest::Client::new();
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_seconds);

        println!(
            "Waiting for llama-server to be healthy at {} (timeout: {} minutes)",
            health_url,
            timeout_seconds / 60
        );

        loop {
            if start_time.elapsed() > timeout_duration {
                return Err(format!(
                    "Model server health check timed out after {} seconds ({} minutes)",
                    timeout_seconds,
                    timeout_seconds / 60
                )
                .into());
            }

            // Check health endpoint
            match client.get(&health_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Model server is healthy, checking if models are loaded...");

                        // Check models endpoint to verify the server is fully operational
                        match client.get(&models_url).send().await {
                            Ok(models_response) => {
                                if models_response.status().is_success() {
                                    println!("Model server is ready and models are accessible!");
                                    return Ok(());
                                } else {
                                    println!(
                                        "Model server is healthy but models not yet accessible (status: {})",
                                        models_response.status()
                                    );
                                }
                            }
                            Err(e) => {
                                println!("Models endpoint not accessible yet: {}", e);
                            }
                        }
                    } else {
                        println!(
                            "Model server health check failed with status: {}",
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    println!("Health check request failed: {}", e);
                }
            }

            // Wait before retrying
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn build_command_args(
        &self,
        model: &Model,
        port: u16,
    ) -> Result<Vec<String>, EngineError> {
        let mut args = Vec::new();

        // Get the model path
        let model_path_absolute = model.get_model_absolute_path();

        // Get llamacpp settings or use defaults
        let settings = model.engine_settings.as_ref().and_then(|s| s.llamacpp.as_ref());

        // Add basic server configuration
        args.extend(["--port".to_string(), port.to_string()]);
        args.extend(["--host".to_string(), "127.0.0.1".to_string()]);

        // Model file - Find the GGUF file in the model directory
        let gguf_file = find_gguf_file(&model_path_absolute)?;
        args.extend(["--model".to_string(), gguf_file]);

        // Logging - create a log file for this model
        let log_path = if let Some(_settings) = settings {
            // For future: settings.log_file.clone()
            None
        } else {
            None
        }
        .unwrap_or_else(|| {
            let log_dir = crate::get_app_data_dir().join("logs/models");
            if !log_dir.exists() {
                let _ = std::fs::create_dir_all(&log_dir);
            }
            log_dir
                .join(format!("{}_llamacpp.log", model.id))
                .to_string_lossy()
                .to_string()
        });
        args.extend(["--log-file".to_string(), log_path]);

        // Add settings-based arguments
        if let Some(settings) = settings {
            // Context & Memory Management
            if let Some(ctx_size) = settings.ctx_size {
                args.extend(["--ctx-size".to_string(), ctx_size.to_string()]);
            }

            if let Some(batch_size) = settings.batch_size {
                args.extend(["--batch-size".to_string(), batch_size.to_string()]);
            }

            if let Some(ubatch_size) = settings.ubatch_size {
                args.extend(["--ubatch-size".to_string(), ubatch_size.to_string()]);
            }

            if let Some(parallel) = settings.parallel {
                args.extend(["--parallel".to_string(), parallel.to_string()]);
            }

            if let Some(keep) = settings.keep {
                args.extend(["--keep".to_string(), keep.to_string()]);
            }

            if settings.mlock.unwrap_or(false) {
                args.push("--mlock".to_string());
            }

            if settings.no_mmap.unwrap_or(false) {
                args.push("--no-mmap".to_string());
            }

            // Threading & Performance
            if let Some(threads) = settings.threads {
                args.extend(["--threads".to_string(), threads.to_string()]);
            }

            if let Some(threads_batch) = settings.threads_batch {
                args.extend(["--threads-batch".to_string(), threads_batch.to_string()]);
            }

            if settings.cont_batching.unwrap_or(true) {
                args.push("--cont-batching".to_string());
            }

            if settings.flash_attn.unwrap_or(false) {
                args.push("--flash-attn".to_string());
            }

            if settings.no_kv_offload.unwrap_or(false) {
                args.push("--no-kv-offload".to_string());
            }

            // Device configuration with auto-detection
            let auto_detected_device_type = match settings.device_type {
                Some(crate::database::models::DeviceType::Cpu) => {
                    println!("Using CPU device (explicitly configured)");
                    "cpu"
                }
                Some(crate::database::models::DeviceType::Cuda) => {
                    println!("Using CUDA device (explicitly configured)");
                    "cuda"
                }
                Some(crate::database::models::DeviceType::Metal) => {
                    println!("Using Metal device (explicitly configured)");
                    "metal"
                }
                Some(crate::database::models::DeviceType::Rocm) => {
                    println!("Using ROCm device (explicitly configured)");
                    "rocm"
                }
                Some(crate::database::models::DeviceType::Vulkan) => {
                    println!("Using Vulkan device (explicitly configured)");
                    "vulkan"
                }
                Some(crate::database::models::DeviceType::Opencl) => {
                    println!("Using OpenCL device (explicitly configured)");
                    "opencl"
                }
                Some(crate::database::models::DeviceType::Auto) | None => {
                    // Auto-detect best available device type
                    let available_devices = crate::ai::device_detection::detect_available_devices();
                    println!(
                        "Auto-detecting device type. Available devices: {} (default: {})",
                        available_devices.devices.len(),
                        available_devices.default_device_type
                    );
                    match available_devices.default_device_type.as_str() {
                        "metal" => {
                            println!("Auto-selected Metal device (best available)");
                            "metal"
                        }
                        "cuda" => {
                            println!("Auto-selected CUDA device (best available)");
                            "cuda"
                        }
                        _ => {
                            println!("Auto-selected CPU device (fallback)");
                            "cpu"
                        }
                    }
                }
            };

            // Auto-select device IDs if not specified
            let final_device_ids: Option<Vec<i32>> = if settings.device_ids.is_none()
                || settings
                    .device_ids
                    .as_ref()
                    .map(|ids| ids.is_empty())
                    .unwrap_or(true)
            {
                // Get all available devices of the selected device type
                let available_devices = crate::ai::device_detection::detect_available_devices();
                let matching_devices: Vec<i32> = available_devices
                    .devices
                    .iter()
                    .filter(|device| {
                        device.device_type.as_str() == auto_detected_device_type && device.is_available
                    })
                    .map(|device| device.id)
                    .collect();

                if !matching_devices.is_empty() {
                    println!(
                        "Auto-selected all available {} devices: {:?}",
                        auto_detected_device_type, matching_devices
                    );
                    Some(matching_devices)
                } else {
                    println!(
                        "No available {} devices found, using default device selection",
                        auto_detected_device_type
                    );
                    None
                }
            } else {
                println!(
                    "Using explicitly configured device IDs: {:?}",
                    settings.device_ids
                );
                settings.device_ids.clone()
            };

            // Set device parameter for LlamaCpp
            if auto_detected_device_type == "cpu" {
                // For CPU, set device to none and ensure no GPU layers
                args.extend(["--device".to_string(), "none".to_string()]);
                args.extend(["--n-gpu-layers".to_string(), "0".to_string()]);
                println!("Using CPU device: none (0 GPU layers)");
            } else if auto_detected_device_type == "metal" {
                // For Metal, set device to metal
                args.extend(["--device".to_string(), "metal".to_string()]);
                println!("Using Metal device: metal");
            } else {
                // For other GPU devices (CUDA, OpenCL), use device IDs
                if let Some(device_ids) = &final_device_ids {
                    if !device_ids.is_empty() {
                        let device_list = device_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        args.extend(["--device".to_string(), device_list.clone()]);
                        println!("Using devices: {}", device_list);
                    }
                }
            }

            // GPU Configuration - only set for non-CPU devices
            if auto_detected_device_type != "cpu" {
                if let Some(n_gpu_layers) = settings.n_gpu_layers {
                    args.extend(["--n-gpu-layers".to_string(), n_gpu_layers.to_string()]);
                } else if final_device_ids.is_some() {
                    // Auto-configure GPU layers for non-CPU devices
                    args.extend(["--n-gpu-layers".to_string(), "999".to_string()]);
                    println!("Auto-configured GPU layers: 999 (offload all layers)");
                }

                if let Some(main_gpu) = settings.main_gpu {
                    args.extend(["--main-gpu".to_string(), main_gpu.to_string()]);
                } else if let Some(device_ids) = &final_device_ids {
                    if !device_ids.is_empty() {
                        args.extend(["--main-gpu".to_string(), device_ids[0].to_string()]);
                        println!("Auto-configured main GPU: {}", device_ids[0]);
                    }
                }

                if let Some(split_mode) = &settings.split_mode {
                    args.extend(["--split-mode".to_string(), split_mode.clone()]);
                } else if let Some(device_ids) = &final_device_ids {
                    if device_ids.len() > 1 {
                        args.extend(["--split-mode".to_string(), "layer".to_string()]);
                        println!("Auto-configured split mode: layer (multiple devices detected)");
                    }
                }

                if let Some(tensor_split) = &settings.tensor_split {
                    args.extend(["--tensor-split".to_string(), tensor_split.clone()]);
                } else if let Some(device_ids) = &final_device_ids {
                    if device_ids.len() > 1 {
                        // Create equal tensor split ratios for all devices
                        let ratio = format!("{}", 1.0 / device_ids.len() as f32);
                        let tensor_split_ratios = vec![ratio; device_ids.len()].join(",");
                        args.extend(["--tensor-split".to_string(), tensor_split_ratios.clone()]);
                        println!("Auto-configured tensor split: {}", tensor_split_ratios);
                    }
                }
            } else {
                println!("Skipping GPU configuration for CPU device");
            }

            // Model Configuration
            if let Some(rope_freq_base) = settings.rope_freq_base {
                args.extend(["--rope-freq-base".to_string(), rope_freq_base.to_string()]);
            }

            if let Some(rope_freq_scale) = settings.rope_freq_scale {
                args.extend(["--rope-freq-scale".to_string(), rope_freq_scale.to_string()]);
            }

            if let Some(rope_scaling) = &settings.rope_scaling {
                args.extend(["--rope-scaling".to_string(), rope_scaling.clone()]);
            }

            if let Some(cache_type_k) = &settings.cache_type_k {
                args.extend(["--cache-type-k".to_string(), cache_type_k.clone()]);
            }

            if let Some(cache_type_v) = &settings.cache_type_v {
                args.extend(["--cache-type-v".to_string(), cache_type_v.clone()]);
            }

            // Advanced Options
            if let Some(seed) = settings.seed {
                args.extend(["--seed".to_string(), seed.to_string()]);
            }

            if let Some(numa) = &settings.numa {
                args.extend(["--numa".to_string(), numa.clone()]);
            }
        }

        Ok(args)
    }
}

#[async_trait::async_trait]
impl LocalEngine for LlamaCppEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::LlamaCpp
    }

    fn name(&self) -> &'static str {
        "LlamaCpp"
    }

    fn version(&self) -> String {
        "dev".to_string()
    }

    async fn start(&self, model: &Model) -> Result<EngineInstance, EngineError> {
        let port = Self::get_available_port();
        let model_uuid = model.id.to_string();

        // Calculate model size for timeout determination
        let model_size = self.calculate_model_size(model).unwrap_or(0);
        let timeout_seconds = self.calculate_timeout_for_model_size(model_size);

        println!(
            "Starting LlamaCpp model {} with timeout {} seconds ({:.1} minutes)",
            model.alias,
            timeout_seconds,
            timeout_seconds as f64 / 60.0
        );

        // Build command arguments
        let args = self.build_command_args(model, port).await?;

        // Start the llama-server process
        let binary_path =
            ResourcePaths::find_executable_binary("llama-server").ok_or_else(|| {
                EngineError::StartupFailed("llama-server binary not found".to_string())
            })?;

        // Create stdout/stderr log file path
        let stdout_stderr_log_path = {
            let log_dir = crate::get_app_data_dir().join("logs/models");
            if !log_dir.exists() {
                std::fs::create_dir_all(&log_dir).map_err(|e| {
                    EngineError::StartupFailed(format!("Failed to create log directory: {}", e))
                })?;
            }
            log_dir
                .join(format!("{}_engine.log", model.id))
                .to_string_lossy()
                .to_string()
        };

        // Clear the stdout/stderr log file before starting
        if let Err(e) = std::fs::write(&stdout_stderr_log_path, "") {
            eprintln!(
                "Warning: Failed to clear stdout/stderr log file {}: {}",
                stdout_stderr_log_path, e
            );
        }

        // Create or open the stdout/stderr log file for writing
        let stdout_stderr_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&stdout_stderr_log_path)
            .map_err(|e| {
                EngineError::StartupFailed(format!("Failed to open stdout/stderr log file: {}", e))
            })?;

        // Clone the file handle for stderr
        let stderr_file = stdout_stderr_file.try_clone().map_err(|e| {
            EngineError::StartupFailed(format!("Failed to clone log file handle: {}", e))
        })?;

        let mut cmd = Command::new(binary_path);
        cmd.args(&args)
            .env("MODEL_UUID", model_uuid.clone()) // Add model UUID for process identification
            .stdout(Stdio::from(stdout_stderr_file))
            .stderr(Stdio::from(stderr_file));

        println!("Starting llama-server process: {:?}", cmd);
        println!(
            "Process output will be logged to: {}",
            stdout_stderr_log_path
        );

        let mut child = cmd
            .spawn()
            .map_err(|e| EngineError::StartupFailed(format!("Failed to spawn process: {}", e)))?;

        let pid = child.id();

        println!(
            "llama-server process spawned with PID: {}, port: {}",
            pid, port
        );

        // Wait a bit to ensure the process has started successfully
        sleep(Duration::from_millis(100)).await;

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has already exited
                eprintln!(
                    "llama-server process exited immediately with status: {}",
                    status
                );
                return Err(EngineError::StartupFailed(format!(
                    "llama-server process failed to start: {}",
                    status
                )));
            }
            Ok(None) => {
                // Process is still running, we'll store it properly in the registry later
                println!("llama-server process is running, waiting for health check...");
            }
            Err(e) => {
                eprintln!("Failed to check llama-server process status: {}", e);
                return Err(EngineError::StartupFailed(format!(
                    "Failed to check process status: {}",
                    e
                )));
            }
        }

        let instance = EngineInstance {
            model_uuid: model_uuid.clone(),
            port,
            pid: Some(pid),
        };

        // Wait for the server to be healthy with model size-based timeout
        self.wait_for_model_health(&instance, timeout_seconds)
            .await
            .map_err(|e| EngineError::StartupFailed(format!("Health check failed: {}", e)))?;

        Ok(instance)
    }

    async fn stop(&self, instance: &EngineInstance) -> Result<(), EngineError> {
        if let Some(pid) = instance.pid {
            #[cfg(unix)]
            {
                // Send SIGTERM to the process
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            #[cfg(windows)]
            {
                // Windows process termination
                use std::process::Command;
                Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output()
                    .map_err(|e| {
                        EngineError::StartupFailed(format!("Failed to stop process: {}", e))
                    })?;
            }
        }
        Ok(())
    }

    async fn health_check(&self, instance: &EngineInstance) -> Result<bool, EngineError> {
        self.check_model_server_health(instance.port)
            .await
            .map(|_| true)
            .map_err(|e| EngineError::HealthCheckFailed(format!("Health check failed: {}", e)))
    }

    async fn get_server_models(
        &self,
        instance: &EngineInstance,
    ) -> Result<Vec<super::ModelInfo>, EngineError> {
        let url = format!("http://localhost:{}/v1/models", instance.port);

        let response = reqwest::get(&url).await.map_err(|e| {
            EngineError::NetworkError(format!("Failed to get server models: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(EngineError::NetworkError(format!(
                "Server models request failed with status: {}",
                response.status()
            )));
        }

        let models_response: super::ModelsResponse = response.json().await.map_err(|e| {
            EngineError::NetworkError(format!("Failed to parse server models response: {}", e))
        })?;

        Ok(models_response.data)
    }
}

/// Find the GGUF file in the given model directory
fn find_gguf_file(model_path: &str) -> Result<String, EngineError> {
    let path = std::path::Path::new(model_path);

    // If the path is already a file and ends with .gguf, return it directly
    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("gguf") {
        return Ok(model_path.to_string());
    }

    // If it's a directory, search for .gguf files
    if path.is_dir() {
        let entries = std::fs::read_dir(path).map_err(|e| {
            EngineError::StartupFailed(format!(
                "Failed to read model directory {}: {}",
                model_path, e
            ))
        })?;

        let mut gguf_files = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                EngineError::StartupFailed(format!("Failed to read directory entry: {}", e))
            })?;
            let file_path = entry.path();

            if file_path.is_file() {
                if let Some(extension) = file_path.extension() {
                    if extension == "gguf" {
                        gguf_files.push(file_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        if gguf_files.is_empty() {
            return Err(EngineError::StartupFailed(format!(
                "No GGUF files found in directory: {}",
                model_path
            )));
        }

        if gguf_files.len() > 1 {
            // If multiple GGUF files, prefer the one with "model" in the name, or take the first one
            if let Some(model_file) = gguf_files
                .iter()
                .find(|f| f.to_lowercase().contains("model"))
            {
                return Ok(model_file.clone());
            }

            println!(
                "Warning: Multiple GGUF files found in {}, using the first one: {}",
                model_path, gguf_files[0]
            );
        }

        return Ok(gguf_files[0].clone());
    }

    // If the path doesn't exist or is neither a file nor directory
    Err(EngineError::StartupFailed(format!(
        "Invalid model path: {} (not a file or directory)",
        model_path
    )))
}
