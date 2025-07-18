use chrono;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::metadata;
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLockInfo {
    pub pid: u32,
    pub port: u16,
    pub model_path: String,
    pub started_at: String,
}

#[derive(Debug, Clone)]
pub enum ModelStartResult {
    Started(u16),
    AlreadyRunning(u16),
}

#[derive(Debug, Clone)]
pub struct ModelManager {}

impl ModelManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Get the lock file path for a model
    fn get_lock_file_path(&self, model_path: &str) -> std::path::PathBuf {
        Path::new(model_path).join(".model.lock")
    }

    /// Check if port is already in use using system commands
    fn is_port_in_use(&self, port: u16) -> bool {
        #[cfg(unix)]
        {
            // Use lsof to check if port is in use
            match Command::new("lsof")
                .arg("-i")
                .arg(format!(":{}", port))
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) => !output.stdout.is_empty(),
                Err(_) => {
                    // Fallback to netstat if lsof is not available
                    match Command::new("netstat")
                        .arg("-an")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .output()
                    {
                        Ok(output) => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            output_str.lines().any(|line| {
                                line.contains(&format!(":{}", port))
                                    && (line.contains("LISTEN") || line.contains("ESTABLISHED"))
                            })
                        }
                        Err(_) => false, // If both commands fail, assume port is free
                    }
                }
            }
        }
        #[cfg(windows)]
        {
            // Use netstat on Windows
            match Command::new("netstat")
                .arg("-an")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.lines().any(|line| {
                        line.contains(&format!(":{}", port))
                            && (line.contains("LISTENING") || line.contains("ESTABLISHED"))
                    })
                }
                Err(_) => false, // If command fails, assume port is free
            }
        }
    }

    /// Find an available port starting from a given port
    fn find_available_port(&self, start_port: u16) -> Option<u16> {
        for port in start_port..start_port + 100 {
            if !self.is_port_in_use(port) {
                return Some(port);
            }
        }
        None
    }

    /// Calculate the total size of model files in bytes
    /// Handles models with multiple weight files (sharded models)
    fn calculate_model_size(
        &self,
        model_path: &str,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let model_dir = Path::new(model_path);
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

        // Log summary of found model files
        println!(
            "Found {} model file(s) with total size: {} bytes ({:.2} GB)",
            model_files.len(),
            total_size,
            total_size as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        // Log details of large models or models with multiple files
        if model_files.len() > 1 || total_size > 1_000_000_000 {
            // > 1GB
            println!("Model file breakdown:");
            for (path, size) in &model_files {
                let size_gb = *size as f64 / (1024.0 * 1024.0 * 1024.0);
                println!("  - {} ({:.2} GB)", path.display(), size_gb);
            }
        }

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

                    // Check for model files by extension and common patterns
                    if self.is_model_file(&path, &file_name_str) {
                        let file_size = metadata(&path)?.len();
                        *total_size += file_size;
                        model_files.push((path.clone(), file_size));

                        // Log individual files for debugging
                        let size_mb = file_size as f64 / (1024.0 * 1024.0);
                        if size_mb > 100.0 {
                            // Log files > 100MB
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

    /// Check if a file is a model weight file
    fn is_model_file(&self, path: &Path, file_name_lower: &str) -> bool {
        // Check by extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if ext == "safetensors"
                || ext == "bin"
                || ext == "gguf"
                || ext == "pt"
                || ext == "pth"
                || ext == "onnx"
                || ext == "tflite"
            {
                return true;
            }
        }

        // Check for common sharded model patterns
        // Examples: model-00001-of-00002.safetensors, pytorch_model-00001-of-00002.bin
        if file_name_lower.contains("model")
            && (file_name_lower.contains("-of-") || file_name_lower.contains("shard"))
            && (file_name_lower.ends_with(".safetensors")
                || file_name_lower.ends_with(".bin")
                || file_name_lower.ends_with(".pt"))
        {
            return true;
        }

        // Check for specific model file patterns
        if file_name_lower == "pytorch_model.bin"
            || file_name_lower == "model.safetensors"
            || file_name_lower == "consolidated.00.pth"
        {
            return true;
        }

        // Skip config files
        if file_name_lower == "params.json" || file_name_lower == "config.json" {
            return false;
        }

        // Check for weight files with numeric suffixes (e.g., model.00.safetensors)
        if file_name_lower.starts_with("model.")
            && file_name_lower.chars().any(|c| c.is_ascii_digit())
            && (file_name_lower.ends_with(".safetensors") || file_name_lower.ends_with(".bin"))
        {
            return true;
        }

        false
    }

    /// Calculate timeout based on model size
    /// Base timeout: 2 minutes (120 seconds)
    /// Additional time: 30 seconds per GB
    /// Maximum timeout: 30 minutes (1800 seconds)
    /// Minimum timeout: 2 minutes (120 seconds)
    fn calculate_timeout_for_model_size(&self, model_size_bytes: u64) -> u64 {
        const BASE_TIMEOUT: u64 = 120; // 2 minutes
        const SECONDS_PER_GB: u64 = 30; // 30 seconds per GB
        const MAX_TIMEOUT: u64 = 1800; // 30 minutes
        const MIN_TIMEOUT: u64 = 120; // 2 minutes

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

    /// Get the path to the candle-server binary
    fn get_model_server_binary_path(
        &self,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        // Get the current executable's directory
        let current_exe = std::env::current_exe()?;
        let current_dir = current_exe.parent().ok_or("Cannot get parent directory")?;

        // Look for candle-server binary in the same directory
        let model_server_path = current_dir.join("candle-server");

        // Check if the binary exists
        if model_server_path.exists() {
            Ok(model_server_path)
        } else {
            // Try with .exe extension on Windows
            #[cfg(windows)]
            {
                let model_server_exe = current_dir.join("candle-server.exe");
                if model_server_exe.exists() {
                    return Ok(model_server_exe);
                }
            }

            // Fallback: look in target/debug or target/release
            let target_debug = current_dir.join("../target/debug/candle-server");
            if target_debug.exists() {
                return Ok(target_debug);
            }

            let target_release = current_dir.join("../target/release/candle-server");
            if target_release.exists() {
                return Ok(target_release);
            }

            Err("candle-server binary not found".into())
        }
    }

    /// Check if the model server is healthy and ready
    async fn wait_for_model_health(
        &self,
        port: u16,
        timeout_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let health_url = format!("http://127.0.0.1:{}/health", port);
        let ready_url = format!("http://127.0.0.1:{}/ready", port);

        let client = reqwest::Client::new();
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_seconds);

        println!(
            "Waiting for model server to be healthy at {} (timeout: {} minutes)",
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
                        println!("Model server is healthy, checking ready status...");

                        // Check ready endpoint
                        match client.get(&ready_url).send().await {
                            Ok(ready_response) => {
                                if ready_response.status().is_success() {
                                    println!("Model server is ready!");
                                    return Ok(());
                                } else {
                                    println!(
                                        "Model server is healthy but not ready yet (status: {})",
                                        ready_response.status()
                                    );
                                }
                            }
                            Err(e) => {
                                println!("Ready endpoint not accessible yet: {}", e);
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

    /// Check if a process is running by PID (enhanced version)
    fn is_process_running(&self, pid: u32) -> bool {
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

    /// Check if a process is actually a model server by examining its command line
    fn is_model_server_process(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            // Try to get the process command line using ps
            match Command::new("ps")
                .arg("-p")
                .arg(pid.to_string())
                .arg("-o")
                .arg("comm=")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Check if the process name contains "candle-server"
                    output_str.contains("candle-server")
                }
                Err(_) => {
                    // If ps fails, assume it's valid (fallback to less strict validation)
                    true
                }
            }
        }
        #[cfg(windows)]
        {
            // Try to get the process image name using tasklist
            match Command::new("tasklist")
                .arg("/FI")
                .arg(format!("PID eq {}", pid))
                .arg("/FO")
                .arg("CSV")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Check if the process name contains "candle-server"
                    output_str.contains("candle-server")
                }
                Err(_) => {
                    // If tasklist fails, assume it's valid (fallback to less strict validation)
                    true
                }
            }
        }
    }

    /// Read lock file information
    async fn read_lock_file(
        &self,
        model_path: &str,
    ) -> Result<ModelLockInfo, Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        let content = fs::read_to_string(&lock_file_path)?;
        let lock_info: ModelLockInfo = serde_json::from_str(&content)?;
        Ok(lock_info)
    }

    /// Write lock file information
    async fn write_lock_file(
        &self,
        model_path: &str,
        lock_info: &ModelLockInfo,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        let content = serde_json::to_string_pretty(lock_info)?;
        fs::write(&lock_file_path, content)?;
        println!("Created lock file at: {}", lock_file_path.display());
        Ok(())
    }

    /// Create a lock file for the model (public version)
    pub async fn create_lock_file(
        &self,
        model_path: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_info = ModelLockInfo {
            pid: std::process::id(),
            port,
            model_path: model_path.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
        };

        self.write_lock_file(model_path, &lock_info).await
    }

    /// Remove lock file
    async fn remove_lock_file(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);
        if lock_file_path.exists() {
            fs::remove_file(&lock_file_path)?;
            println!("Removed lock file at: {}", lock_file_path.display());
        }
        Ok(())
    }

    /// Remove lock file (public version)
    pub async fn remove_lock_file_public(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.remove_lock_file(model_path).await
    }

    pub async fn is_model_running(&self, model_path: &str) -> bool {
        // Check lock file with enhanced validation
        match self.is_model_already_running(model_path).await {
            Ok(true) => true,
            Ok(false) => false,
            Err(_) => false,
        }
    }

    /// Enhanced check if a model is already running with comprehensive validation
    pub async fn is_model_already_running(
        &self,
        model_path: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let lock_file_path = self.get_lock_file_path(model_path);

        if !lock_file_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&lock_file_path)?;
        let lock_info: ModelLockInfo = serde_json::from_str(&content)?;

        // First check if the process is still running
        let process_exists = self.is_process_running(lock_info.pid);

        if !process_exists {
            // Process is dead, remove stale lock file
            println!(
                "Removing stale lock file for dead process {}",
                lock_info.pid
            );
            let _ = fs::remove_file(&lock_file_path);
            return Ok(false);
        }

        // Process exists, now check if it's actually using the expected port
        let port_in_use = self.is_port_in_use(lock_info.port);

        if !port_in_use {
            // Process exists but port is not in use, likely crashed or not a model server
            println!(
                "Process {} exists but port {} is not in use, removing stale lock file",
                lock_info.pid, lock_info.port
            );
            let _ = fs::remove_file(&lock_file_path);
            return Ok(false);
        }

        // Additional validation: check if the process is actually our model server
        if !self.is_model_server_process(lock_info.pid) {
            println!(
                "Process {} is not a model server, removing stale lock file",
                lock_info.pid
            );
            let _ = fs::remove_file(&lock_file_path);
            return Ok(false);
        }

        println!(
            "Model already running with PID {} on port {}",
            lock_info.pid, lock_info.port
        );
        Ok(true)
    }

    pub async fn start_model(
        &self,
        model_id: &str,
        model_path: String,
        model_type: String,
        device_type: crate::ai::DeviceType,
        device_ids: Option<Vec<i32>>,
        verbose: Option<bool>,
        max_num_seqs: Option<usize>,
        block_size: Option<usize>,
        weight_file: Option<String>,
        dtype: Option<String>,
        kvcache_mem_gpu: Option<usize>,
        kvcache_mem_cpu: Option<usize>,
        record_conversation: Option<bool>,
        holding_time: Option<usize>,
        multi_process: Option<bool>,
        log: Option<bool>,
    ) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
        // Check if already running using enhanced lock file validation
        if self.is_model_running(&model_path).await {
            // Model is running, get the port from lock file
            if let Ok(lock_info) = self.read_lock_file(&model_path).await {
                return Ok(ModelStartResult::AlreadyRunning(lock_info.port));
            }
        }

        // Find an available port
        let port = self
            .find_available_port(8080)
            .ok_or("No available port found")?;

        // Get the candle-server binary path
        let binary_path = self.get_model_server_binary_path()?;

        // Build the command arguments
        let mut command = Command::new(&binary_path);
        command
            .arg("--port")
            .arg(port.to_string())
            .arg("--model-id")
            .arg(model_id)
            .arg("--weight-path")
            .arg(ModelUtils::get_model_absolute_path(&model_path));

        // Add verbose flag if specified
        if verbose.unwrap_or(false) {
            command.arg("--verbose");
        }

        // Add max-num-seqs if specified (default: 256)
        if let Some(max_seqs) = max_num_seqs {
            command.arg("--max-num-seqs").arg(max_seqs.to_string());
        }

        // Add block-size if specified (default: 32)
        if let Some(block_sz) = block_size {
            command.arg("--block-size").arg(block_sz.to_string());
        }

        // Add weight-file if specified (for quantized models)
        if let Some(weight_f) = weight_file {
            command.arg("--weight-file").arg(weight_f);
        }

        // Add dtype if specified
        if let Some(dt) = dtype {
            command.arg("--dtype").arg(dt);
        }

        // Add CPU flag if device type is CPU
        if matches!(device_type, crate::ai::DeviceType::Cpu) {
            command.arg("--cpu");
        }

        // Add kvcache-mem-gpu if specified (default: 4096)
        if let Some(kvcache_gpu) = kvcache_mem_gpu {
            command
                .arg("--kvcache-mem-gpu")
                .arg(kvcache_gpu.to_string());
        }

        // Add kvcache-mem-cpu if specified (default: 128)
        if let Some(kvcache_cpu) = kvcache_mem_cpu {
            command
                .arg("--kvcache-mem-cpu")
                .arg(kvcache_cpu.to_string());
        }

        // Add record-conversation flag if specified
        if record_conversation.unwrap_or(false) {
            command.arg("--record-conversation");
        }

        // Add device IDs if provided (for GPU usage)
        if let Some(ids) = device_ids {
            if !ids.is_empty() {
                let device_ids_str = ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                command.arg("--device-ids").arg(device_ids_str);
            }
        }

        // Add holding-time if specified (default: 500)
        if let Some(hold_time) = holding_time {
            command.arg("--holding-time").arg(hold_time.to_string());
        }

        // Add multi-process flag if specified
        if multi_process.unwrap_or(false) {
            command.arg("--multi-process");
        }

        // Add log flag if specified
        if log.unwrap_or(false) {
            command.arg("--log");
        }

        // Add the model type as a subcommand (e.g., "llama")
        command.arg(&model_type.to_lowercase());

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        println!("Starting candle-server process: {:?}", command);

        // Spawn the process
        let mut child = command.spawn()?;
        let pid = child.id();

        println!(
            "candle-server process spawned with PID: {}, port: {}",
            pid, port
        );

        // Wait a bit to ensure the process has started successfully
        sleep(Duration::from_millis(100)).await;

        // Check if the process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has already exited
                eprintln!(
                    "candle-server process exited immediately with status: {}",
                    status
                );
                let _ = self.remove_lock_file(&model_path).await;
                return Err(format!("candle-server process failed to start: {}", status).into());
            }
            Ok(None) => {
                // Process is still running, detach it
                println!("candle-server process is running, waiting for health check...");
                // We don't want to wait for the child process, so we detach it
                // The process will continue running independently
                std::mem::forget(child);
            }
            Err(e) => {
                eprintln!("Failed to check candle-server process status: {}", e);
                let _ = self.remove_lock_file(&model_path).await;
                return Err(format!("Failed to check process status: {}", e).into());
            }
        }

        // Calculate timeout based on model size
        let timeout_seconds = match self.calculate_model_size(&model_path) {
            Ok(size) => self.calculate_timeout_for_model_size(size),
            Err(e) => {
                eprintln!(
                    "Failed to calculate model size: {}, using default timeout of 20 minutes",
                    e
                );
                1200 // Default to 20 minutes if we can't calculate size
            }
        };

        // Wait for the model server to be healthy and ready
        if let Err(e) = self.wait_for_model_health(port, timeout_seconds).await {
            eprintln!("Model server health check failed: {}", e);
            // Try to stop the process if health check fails
            let _ = self.stop_model(&model_path).await;
            return Err(format!("Model server failed to become healthy: {}", e).into());
        }

        println!("Model server is healthy and ready on port {}", port);
        Ok(ModelStartResult::Started(port))
    }

    pub async fn stop_model(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Read lock file to get process information
        let lock_info = match self.read_lock_file(model_path).await {
            Ok(info) => info,
            Err(_) => {
                // No lock file found, model is not running
                return Ok(());
            }
        };

        // Check if the process is still running
        if self.is_process_running(lock_info.pid) {
            println!(
                "Terminating candle-server process with PID: {}",
                lock_info.pid
            );

            // Try to terminate the process gracefully first
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                // Send SIGTERM first for graceful shutdown
                let pid = Pid::from_raw(lock_info.pid as i32);
                if let Err(e) = signal::kill(pid, Signal::SIGTERM) {
                    eprintln!("Failed to send SIGTERM to process {}: {}", lock_info.pid, e);
                }

                // Wait a bit for graceful shutdown
                sleep(Duration::from_millis(1000)).await;

                // Check if process is still running
                if self.is_process_running(lock_info.pid) {
                    eprintln!(
                        "Process {} did not respond to SIGTERM, sending SIGKILL",
                        lock_info.pid
                    );
                    if let Err(e) = signal::kill(pid, Signal::SIGKILL) {
                        eprintln!("Failed to send SIGKILL to process {}: {}", lock_info.pid, e);
                    }
                }
            }

            #[cfg(windows)]
            {
                // On Windows, use taskkill command
                let output = Command::new("taskkill")
                    .arg("/PID")
                    .arg(lock_info.pid.to_string())
                    .arg("/F") // Force terminate
                    .output();

                match output {
                    Ok(result) => {
                        if !result.status.success() {
                            let stderr = String::from_utf8_lossy(&result.stderr);
                            eprintln!("Failed to terminate process {}: {}", lock_info.pid, stderr);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to execute taskkill for process {}: {}",
                            lock_info.pid, e
                        );
                    }
                }
            }

            // Wait a bit more to ensure process is terminated
            sleep(Duration::from_millis(500)).await;

            if self.is_process_running(lock_info.pid) {
                eprintln!(
                    "Process {} is still running after termination attempt",
                    lock_info.pid
                );
            } else {
                println!("Process {} terminated successfully", lock_info.pid);
            }
        }

        // Remove lock file
        self.remove_lock_file(model_path).await?;

        Ok(())
    }

    pub async fn get_model_port(&self, model_path: &str) -> Option<u16> {
        // Get port from lock file
        if let Ok(lock_info) = self.read_lock_file(model_path).await {
            Some(lock_info.port)
        } else {
            None
        }
    }

    pub async fn check_and_cleanup_model(
        &self,
        model_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if model is running using enhanced lock file validation
        if self.is_model_running(model_path).await {
            // Model is running, stop it
            self.stop_model(model_path).await?;
        } else {
            // Model is not running, but clean up any stale lock files
            let _ = self.remove_lock_file(model_path).await;
        }
        Ok(())
    }
}

use crate::ai::models::ModelUtils;
use std::sync::OnceLock;

static MODEL_MANAGER: OnceLock<ModelManager> = OnceLock::new();

pub fn get_model_manager() -> &'static ModelManager {
    MODEL_MANAGER.get_or_init(|| ModelManager::new())
}
