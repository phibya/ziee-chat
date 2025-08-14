use reqwest;
use std::collections::HashMap;
use std::fs::{metadata, OpenOptions};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

// Import the engine abstraction
use super::engines::{EngineType, LocalEngine, EngineInstance};
use super::engines::mistralrs::MistralRsEngine;
use super::engines::llamacpp::LlamaCppEngine;

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
    Started { port: u16, pid: u32 },
    AlreadyRunning { port: u16, pid: u32 },
    Failed { error: String, stdout_stderr_log_path: String },
}

/// Check if port is already in use using system commands
fn is_port_in_use(port: u16) -> bool {
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
fn find_available_port(start_port: u16) -> Option<u16> {
    for port in start_port..start_port + 100 {
        if !is_port_in_use(port) {
            return Some(port);
        }
    }
    None
}

/// Calculate the total size of model files in bytes
/// Handles models with multiple weight files (sharded models)
fn calculate_model_size(model_path: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let model_dir = Path::new(model_path);
    let mut total_size = 0u64;
    let mut model_files = Vec::new();

    if model_dir.is_dir() {
        // If it's a directory, sum up all files recursively
        scan_model_files(model_dir, &mut total_size, &mut model_files)?;
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
                if is_model_file(&path, &file_name_str) {
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
            scan_model_files(&path, total_size, model_files)?;
        }
    }
    Ok(())
}

/// Check if a file is a model weight file
fn is_model_file(path: &Path, file_name_lower: &str) -> bool {
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

/// Read config.json and extract the number of layers
fn get_model_layer_count(model_path: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let model_dir = Path::new(model_path);
    let config_path = if model_dir.is_dir() {
        model_dir.join("config.json")
    } else {
        // If model_path is a file, look for config.json in the same directory
        model_dir.parent().unwrap_or(model_dir).join("config.json")
    };

    if !config_path.exists() {
        return Err("config.json not found in model directory".into());
    }

    let config_content = std::fs::read_to_string(&config_path)?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Try different common field names for layer count
    let layer_count = config
        .get("num_hidden_layers")
        .or_else(|| config.get("n_layers"))
        .or_else(|| config.get("num_layers"))
        .or_else(|| config.get("n_layer"))
        .and_then(|v| v.as_u64())
        .ok_or("No layer count field found in config.json")?;

    Ok(layer_count as usize)
}

/// Calculate timeout based on model size
/// Base timeout: 2 minutes (120 seconds)
/// Additional time: 30 seconds per GB
/// Maximum timeout: 30 minutes (1800 seconds)
/// Minimum timeout: 2 minutes (120 seconds)
fn calculate_timeout_for_model_size(model_size_bytes: u64) -> u64 {
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

/// Get the path to the mistralrs-server binary
fn get_model_server_binary_path(
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    // Get the current executable's directory
    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe.parent().ok_or("Cannot get parent directory")?;

    // Look for mistralrs-server binary in the same directory
    let model_server_path = current_dir.join("mistralrs-server");

    // Check if the binary exists
    if model_server_path.exists() {
        Ok(model_server_path)
    } else {
        // Try with .exe extension on Windows
        #[cfg(windows)]
        {
            let model_server_exe = current_dir.join("mistralrs-server.exe");
            if model_server_exe.exists() {
                return Ok(model_server_exe);
            }
        }

        // Fallback: look in mistralrs-server/target/debug or target/release
        let mistralrs_debug = current_dir.join("mistralrs-server/target/debug/mistralrs-server");
        if mistralrs_debug.exists() {
            return Ok(mistralrs_debug);
        }

        let mistralrs_release =
            current_dir.join("mistralrs-server/target/release/mistralrs-server");
        if mistralrs_release.exists() {
            return Ok(mistralrs_release);
        }

        // Also check in our target directory
        let target_debug = current_dir.join("../target/debug/mistralrs-server");
        if target_debug.exists() {
            return Ok(target_debug);
        }

        let target_release = current_dir.join("../target/release/mistralrs-server");
        if target_release.exists() {
            return Ok(target_release);
        }

        Err("mistralrs-server binary not found".into())
    }
}

/// Check if the model server is healthy (quick health check without timeout)
async fn check_model_server_health(
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
                Ok(())
            } else {
                Err(format!("Health check failed with status: {}", resp.status()).into())
            }
        }
        Ok(Err(e)) => Err(format!("Health check request failed: {}", e).into()),
        Err(_) => Err("Health check timed out".into()),
    }
}

/// Check if the model server is healthy and ready (with timeout for startup)
async fn wait_for_model_health(
    port: u16,
    timeout_seconds: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let health_url = format!("http://127.0.0.1:{}/health", port);
    let models_url = format!("http://127.0.0.1:{}/v1/models", port);

    let client = reqwest::Client::new();
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout_seconds);

    println!(
        "Waiting for mistralrs-server to be healthy at {} (timeout: {} minutes)",
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

/// Find any mistralrs-server process (for fallback when registry is out of sync)
async fn find_any_model_server_process() -> Option<(u32, u16)> {
    #[cfg(unix)]
    {
        // Use ps to get all processes with their command lines
        match Command::new("ps")
            .arg("-eo")
            .arg("pid,command")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                for line in output_str.lines().skip(1) {
                    // Skip header line
                    if line.contains("mistralrs-server") {
                        // Parse the line to get PID and check if it contains our model ID
                        if let Some(space_idx) = line.find(' ') {
                            if let Ok(pid) = line[..space_idx].trim().parse::<u32>() {
                                let command_line = &line[space_idx..];

                                // Extract port from command line (look for --port argument)
                                if let Some(port) = extract_port_from_command_line(command_line) {
                                    // For now, we'll use a simpler approach - store running models in memory
                                    // and match by port. The environment variable is still useful for debugging.
                                    return Some((pid, port));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => return None,
        }
    }

    #[cfg(windows)]
    {
        // Use wmic to get process command lines on Windows
        match Command::new("wmic")
            .arg("process")
            .arg("get")
            .arg("processid,commandline")
            .arg("/format:csv")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                for line in output_str.lines().skip(1) {
                    // Skip header line
                    if line.contains("mistralrs-server") {
                        // Parse CSV format: Node,CommandLine,ProcessId
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 3 {
                            if let Ok(pid) = parts[2].trim().parse::<u32>() {
                                let command_line = parts[1];

                                // Extract port from command line
                                if let Some(port) = extract_port_from_command_line(command_line) {
                                    return Some((pid, port));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => return None,
        }
    }

    None
}

/// Extract port number from mistralrs-server command line
fn extract_port_from_command_line(command_line: &str) -> Option<u16> {
    // Look for --port argument
    if let Some(port_idx) = command_line.find("--port") {
        let after_port = &command_line[port_idx + 6..]; // Skip "--port"

        // Find the next argument (port number)
        for word in after_port.split_whitespace() {
            if let Ok(port) = word.parse::<u16>() {
                return Some(port);
            }
        }
    }
    None
}

/// Verify that the server at the given port is running the expected model
async fn verify_model_uuid_match(
    port: u16, 
    expected_model_id: &Uuid
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let server_info_url = format!("http://127.0.0.1:{}/server-info", port);
    
    // Short timeout for this check
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        client.get(&server_info_url).send(),
    ).await??;

    if !response.status().is_success() {
        return Err(format!("Server info request failed with status: {}", response.status()).into());
    }

    let server_info: serde_json::Value = response.json().await?;
    
    // Extract model_uuid from server response
    let server_model_uuid = server_info
        .get("model_uuid")
        .and_then(|v| v.as_str())
        .ok_or("No model_uuid in server response")?;

    // Compare with expected model ID
    Ok(server_model_uuid == expected_model_id.to_string())
}

/// Check if a process is actually a mistral server by examining its command line
fn is_model_server_process(pid: u32) -> bool {
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
                // Check if the process name contains "mistralrs-server"
                output_str.contains("mistralrs-server")
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
                // Check if the process name contains "mistralrs-server"
                output_str.contains("mistralrs-server")
            }
            Err(_) => {
                // If tasklist fails, assume it's valid (fallback to less strict validation)
                true
            }
        }
    }
}

/// Robust multi-stage verification that a model server is running correctly
pub async fn verify_model_server_running(model_id: &Uuid) -> Option<(u32, u16)> {
    // Stage 1: Get PID and port from database
    let (pid, port) = match crate::database::queries::models::get_model_runtime_info(model_id).await {
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
            model_id, None, None, false
        ).await;
        return None;
    }

    // Stage 3: Request /server-info to verify model UUID match
    match verify_model_uuid_match(port, model_id).await {
        Ok(true) => {
            println!("Model {} verified running on PID {} port {} with correct UUID", 
                     model_id, pid, port);
            Some((pid, port))
        }
        Ok(false) => {
            println!("Model {} PID {} port {} is running different model UUID", 
                     model_id, pid, port);
            // Clean up incorrect database entry
            let _ = crate::database::queries::models::update_model_runtime_info(
                model_id, None, None, false
            ).await;
            None
        }
        Err(e) => {
            println!("Failed to verify model {} at port {}: {}", model_id, port, e);
            // Server might be starting up or unhealthy, don't clean database yet
            None
        }
    }
}

/// Check if a model is running by model ID by examining the registry and process list
/// Returns (pid, port) if the model is running and healthy, None otherwise
pub async fn is_model_running(model_id: &Uuid) -> Option<(u32, u16)> {
    // First check our registry
    let registry_entry = {
        if let Ok(registry) = MODEL_REGISTRY.read() {
            registry.get(model_id).map(|p| (p.pid, p.port))
        } else {
            None
        }
    };

    if let Some((pid, port)) = registry_entry {
        // Verify the process is actually running and healthy
        if is_process_running(pid) && is_model_server_process(pid) {
            // Check server health
            match check_model_server_health(port).await {
                Ok(()) => {
                    println!(
                        "Model {} is running and healthy on PID {} port {} (from registry)",
                        model_id, pid, port
                    );
                    return Some((pid, port));
                }
                Err(e) => {
                    println!("Model {} health check failed: {}", model_id, e);
                    // Remove from registry if health check fails
                    if let Ok(mut registry) = MODEL_REGISTRY.write() {
                        registry.remove(model_id);
                    }
                    return None;
                }
            }
        } else {
            println!("Process {} for model {} is not responding", pid, model_id);
            // Remove from registry if process is not running
            if let Ok(mut registry) = MODEL_REGISTRY.write() {
                registry.remove(model_id);
            }
        }
    }

    // Fallback: scan all mistralrs-server processes
    // This is less reliable but handles cases where registry is out of sync
    if let Some((pid, port)) = find_any_model_server_process().await {
        // Verify the process is actually running and healthy
        if is_process_running(pid) && is_model_server_process(pid) {
            // Check server health
            match check_model_server_health(port).await {
                Ok(()) => {
                    println!(
                        "Found running model server on PID {} port {} (process scan)",
                        pid, port
                    );
                    // Note: We can't add to registry here since we don't have the child process handle
                    // This is just a fallback for orphaned processes
                    Some((pid, port))
                }
                Err(e) => {
                    println!("Model server health check failed: {}", e);
                    None
                }
            }
        } else {
            println!("Process {} is not responding", pid);
            None
        }
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ModelStartParams {
    // Core model configuration
    pub model_path: String,
    pub command: String, // "plain", "gguf", "run", "vision-plain", etc.
    pub model_id_name: Option<String>, // For --model-id in subcommands
    pub tokenizer_json: Option<String>,
    pub arch: Option<String>,

    // Quantization and weights
    pub quantized_filename: Option<String>, // For GGUF models
    pub weight_file: Option<String>,
    pub dtype: Option<String>,
    pub in_situ_quant: Option<String>, // --isq parameter

    // Device and performance
    pub device_type: crate::ai::DeviceType,
    pub device_ids: Option<Vec<i32>>,
    pub num_device_layers: Option<Vec<String>>, // Per-device layer distribution
    pub cpu: bool,

    // Sequence and memory management
    pub max_seqs: Option<usize>,
    pub max_seq_len: Option<usize>,
    pub no_kv_cache: bool,
    pub truncate_sequence: bool,

    // PagedAttention configuration
    pub paged_attn_gpu_mem: Option<usize>,
    pub paged_attn_gpu_mem_usage: Option<f32>,
    pub paged_ctxt_len: Option<usize>,
    pub paged_attn_block_size: Option<usize>,
    pub no_paged_attn: bool,
    pub paged_attn: bool,

    // Chat and templates
    pub chat_template: Option<String>,
    pub jinja_explicit: Option<String>,

    // Performance and optimization
    pub prompt_chunksize: Option<usize>,
    pub prefix_cache_n: Option<usize>,

    // Vision model parameters
    pub max_edge: Option<usize>,
    pub max_num_images: Option<usize>,
    pub max_image_length: Option<usize>,

    // Server configuration
    pub serve_ip: Option<String>,
    pub seed: Option<u64>,
    pub log_file: Option<String>,

    // Search capabilities
    pub enable_search: bool,
    pub search_bert_model: Option<String>,

    // Interactive and thinking
    pub interactive_mode: bool,
    pub enable_thinking: bool,

    // Token source for authentication
    pub token_source: Option<String>,
}

impl Default for ModelStartParams {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            command: "run".to_string(), // Use auto-loader by default
            model_id_name: None,
            tokenizer_json: None,
            arch: None,
            quantized_filename: None,
            weight_file: None,
            dtype: None,
            in_situ_quant: None,
            device_type: crate::ai::DeviceType::Cpu,
            device_ids: None,
            num_device_layers: None,
            cpu: false,
            max_seqs: None,    // Will use mistralrs default
            max_seq_len: None, // Will use model default
            no_kv_cache: false,
            truncate_sequence: false,
            paged_attn_gpu_mem: None,
            paged_attn_gpu_mem_usage: None,
            paged_ctxt_len: None,
            paged_attn_block_size: None,
            no_paged_attn: false,
            paged_attn: false,
            chat_template: None,
            jinja_explicit: None,
            prompt_chunksize: None,
            prefix_cache_n: None,
            max_edge: None,
            max_num_images: None,
            max_image_length: None,
            serve_ip: None,
            seed: None,
            log_file: None,
            enable_search: false,
            search_bert_model: None,
            interactive_mode: false,
            enable_thinking: false,
            token_source: None,
        }
    }
}

pub async fn start_model_with_engine(
    model_id: &Uuid,
    model: &crate::database::models::model::Model,
) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
    // Create the appropriate engine based on model's engine_type
    let engine: Box<dyn LocalEngine> = match model.engine_type.as_str() {
        "mistralrs" => Box::new(MistralRsEngine::new()),
        "llamacpp" => Box::new(LlamaCppEngine::new()),
        _ => {
            println!("Unknown engine type '{}', defaulting to mistralrs", model.engine_type);
            Box::new(MistralRsEngine::new())
        }
    };

    // Start the engine with the model
    match engine.start(model).await {
        Ok(instance) => {
            let port = instance.port;
            let pid = instance.pid.unwrap_or(0);
            
            // Register the instance in our registry
            if let Ok(mut registry) = MODEL_REGISTRY.write() {
                // For now, we'll still track the Child process handle for backward compatibility
                // This will be refactored once we fully migrate to the engine system
                println!("Engine {} started model {} on PID {} port {}", 
                        engine.name(), model_id, pid, port);
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
                    .join(format!("{}_engine_error.log", model_id))
                    .to_string_lossy()
                    .to_string()
            };
            Ok(ModelStartResult::Failed { 
                error: error_msg, 
                stdout_stderr_log_path 
            })
        }
    }
}

// Main function - uses engine abstraction
pub async fn start_model(
    model_id: &Uuid,
    _params: ModelStartParams,
) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
    // Check if already running using process inspection
    if let Some((pid, port)) = is_model_running(model_id).await {
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
                    println!("Successfully killed child process with model_id: {}", model_id);
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
                    Err(_) => {}, // Process likely already terminated
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
                    eprintln!("Failed to kill process {}: {}", pid, String::from_utf8_lossy(&output.stderr));
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
    if let Some((pid, port)) = is_model_running(model_id).await {
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
                    eprintln!("Error checking process status for model {}: {}", model_id, e);
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