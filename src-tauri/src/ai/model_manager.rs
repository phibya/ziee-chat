use reqwest;
use std::collections::HashMap;
use std::fs::{metadata, OpenOptions};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

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

pub async fn start_model(
    model_id: &Uuid,
    params: ModelStartParams,
) -> Result<ModelStartResult, Box<dyn std::error::Error + Send + Sync>> {
    // Check if already running using process inspection
    if let Some((pid, port)) = is_model_running(model_id).await {
        return Ok(ModelStartResult::AlreadyRunning { port, pid });
    }

    // Find an available port
    let port = find_available_port(8080).ok_or("No available port found")?;

    // Get the mistralrs-server binary path
    let binary_path = get_model_server_binary_path()?;

    // Build the command arguments for mistralrs-server
    let mut command = Command::new(&binary_path);

    // Add global arguments first

    // Server configuration
    command.arg("--port").arg(port.to_string());

    if let Some(ip) = &params.serve_ip {
        command.arg("--serve-ip").arg(ip);
    }

    // Seed for reproducibility
    if let Some(seed) = params.seed {
        command.arg("--seed").arg(seed.to_string());
    }

    // Logging
    let log_path = if let Some(log_file) = &params.log_file {
        log_file.clone()
    } else {
        let log_dir = crate::get_app_data_dir().join("logs/models");
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }
        log_dir
            .join(format!("{}.log", model_id))
            .to_string_lossy()
            .to_string()
    };
    command.arg("--log").arg(&log_path);

    // Create stdout/stderr log file path
    let stdout_stderr_log_path = {
        let log_dir = crate::get_app_data_dir().join("logs/models");
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir)?;
        }
        log_dir
            .join(format!("{}_stdout_stderr.log", model_id))
            .to_string_lossy()
            .to_string()
    };

    // Clear the stdout/stderr log file before starting
    if let Err(e) = std::fs::write(&stdout_stderr_log_path, "") {
        eprintln!("Warning: Failed to clear stdout/stderr log file {}: {}", stdout_stderr_log_path, e);
    }

    // Sequence management
    if params.truncate_sequence {
        command.arg("--truncate-sequence");
    }

    if let Some(max_seqs) = params.max_seqs {
        command.arg("--max-seqs").arg(max_seqs.to_string());
    }

    if params.no_kv_cache {
        command.arg("--no-kv-cache");
    }

    // Device configuration
    if params.cpu || matches!(params.device_type, crate::ai::DeviceType::Cpu) {
        command.arg("--cpu");
    }

    // Device layers configuration - use explicit num_device_layers or generate from device_ids
    if let Some(layers) = &params.num_device_layers {
        command.arg("--num-device-layers").arg(layers.join(";"));
    } else if let Some(ids) = &params.device_ids {
        if !ids.is_empty()
            && !params.cpu
            && !matches!(params.device_type, crate::ai::DeviceType::Cpu)
        {
            // Only add --num-device-layers if there are multiple devices
            if ids.len() > 1 {
                // Try to read layer count from config.json and distribute evenly
                match get_model_layer_count(&params.model_path) {
                    Ok(total_layers) => {
                        let layers_per_device = total_layers / ids.len();
                        let remainder = total_layers % ids.len();
                        
                        let device_layers_str = ids
                            .iter()
                            .enumerate()
                            .map(|(i, id)| {
                                // Distribute remainder to first devices
                                let layers = if i < remainder {
                                    layers_per_device + 1
                                } else {
                                    layers_per_device
                                };
                                format!("{}:{}", id, layers)
                            })
                            .collect::<Vec<_>>()
                            .join(";");
                        
                        println!("Distributing {} layers across {} devices: {}", 
                               total_layers, ids.len(), device_layers_str);
                        command.arg("--num-device-layers").arg(device_layers_str);
                    }
                    Err(e) => {
                        println!("Could not read layer count from config.json ({}), using default distribution", e);
                        let device_layers_str = ids
                            .iter()
                            .enumerate()
                            .map(|(_i, id)| format!("{}:32", id)) // 32 layers per device as fallback
                            .collect::<Vec<_>>()
                            .join(";");
                        command.arg("--num-device-layers").arg(device_layers_str);
                    }
                }
            }
            // For single device, don't add --num-device-layers parameter at all
        }
    }

    // In-situ quantization
    if let Some(isq) = &params.in_situ_quant {
        command.arg("--isq").arg(isq);
    }

    // PagedAttention configuration
    if let Some(gpu_mem) = params.paged_attn_gpu_mem {
        command.arg("--pa-gpu-mem").arg(gpu_mem.to_string());
    }

    if let Some(gpu_mem_usage) = params.paged_attn_gpu_mem_usage {
        command
            .arg("--pa-gpu-mem-usage")
            .arg(gpu_mem_usage.to_string());
    }

    if let Some(ctxt_len) = params.paged_ctxt_len {
        command.arg("--pa-ctxt-len").arg(ctxt_len.to_string());
    }

    if let Some(block_size) = params.paged_attn_block_size {
        command.arg("--pa-blk-size").arg(block_size.to_string());
    }

    if params.no_paged_attn {
        command.arg("--no-paged-attn");
    }

    if params.paged_attn {
        command.arg("--paged-attn");
    }

    // Performance optimization
    if let Some(prefix_cache) = params.prefix_cache_n {
        command
            .arg("--prefix-cache-n")
            .arg(prefix_cache.to_string());
    }

    if let Some(prompt_chunk) = params.prompt_chunksize {
        command
            .arg("--prompt-batchsize")
            .arg(prompt_chunk.to_string());
    }

    // Chat templates
    if let Some(chat_template) = &params.chat_template {
        command.arg("--chat-template").arg(chat_template);
    }

    if let Some(jinja) = &params.jinja_explicit {
        command.arg("--jinja-explicit").arg(jinja);
    }

    // Token source
    if let Some(token_source) = &params.token_source {
        command.arg("--token-source").arg(token_source);
    }

    // Interactive mode and thinking
    if params.interactive_mode {
        command.arg("--interactive-mode");
    }

    // Search capabilities
    if params.enable_search {
        command.arg("--enable-search");
    }

    if let Some(bert_model) = &params.search_bert_model {
        command.arg("--search-bert-model").arg(bert_model);
    }

    if params.enable_thinking {
        command.arg("--enable-thinking");
    }

    // Add the model subcommand based on model type
    let model_path_absolute = ModelUtils::get_model_absolute_path(&params.model_path);

    match params.command.to_lowercase().as_str() {
        "plain" => {
            command.arg("plain");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }

            // Add plain-specific parameters
            if let Some(tokenizer) = &params.tokenizer_json {
                command.arg("--tokenizer-json").arg(tokenizer);
            }
            if let Some(arch) = &params.arch {
                command.arg("--arch").arg(arch);
            }
            if let Some(dtype) = &params.dtype {
                command.arg("--dtype").arg(dtype);
            }
            if let Some(max_seq_len) = params.max_seq_len {
                command.arg("--max-seq-len").arg(max_seq_len.to_string());
            }
        }
        "gguf" => {
            command.arg("gguf");
            command
                .arg("--quantized-model-id")
                .arg(&model_path_absolute);

            if let Some(filename) = &params.quantized_filename {
                command.arg("--quantized-filename").arg(filename);
            } else {
                // Default GGUF filename patterns
                command.arg("--quantized-filename").arg("*.gguf");
            }

            // Add GGUF-specific parameters
            if let Some(dtype) = &params.dtype {
                command.arg("--dtype").arg(dtype);
            }
            if let Some(max_seq_len) = params.max_seq_len {
                command.arg("--max-seq-len").arg(max_seq_len.to_string());
            }
        }
        "run" => {
            command.arg("run");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }

            // Add run-specific parameters (auto-loader)
            if let Some(dtype) = &params.dtype {
                command.arg("--dtype").arg(dtype);
            }
            if let Some(max_seq_len) = params.max_seq_len {
                command.arg("--max-seq-len").arg(max_seq_len.to_string());
            }
        }
        "vision-plain" => {
            command.arg("vision-plain");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }

            // Add vision-specific parameters
            if let Some(max_edge) = params.max_edge {
                command.arg("--max-edge").arg(max_edge.to_string());
            }
            if let Some(max_images) = params.max_num_images {
                command.arg("--max-num-images").arg(max_images.to_string());
            }
            if let Some(max_image_len) = params.max_image_length {
                command
                    .arg("--max-image-length")
                    .arg(max_image_len.to_string());
            }
            if let Some(dtype) = &params.dtype {
                command.arg("--dtype").arg(dtype);
            }
            if let Some(max_seq_len) = params.max_seq_len {
                command.arg("--max-seq-len").arg(max_seq_len.to_string());
            }
        }
        "x-lora" => {
            command.arg("x-lora");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }
            // X-LoRA specific parameters would go here
        }
        "lora" => {
            command.arg("lora");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }
            // LoRA specific parameters would go here
        }
        "toml" => {
            command.arg("toml");
            command.arg("--toml-path").arg(&model_path_absolute);
        }
        _ => {
            // Default to run (auto-loader) for unknown model types
            command.arg("run");
            command.arg("--model-id");
            if let Some(model_id_name) = &params.model_id_name {
                command.arg(model_id_name);
            } else {
                command.arg(&model_path_absolute);
            }
        }
    }

    // Add our internal model UUID as an environment variable for process identification
    // This helps us identify which process belongs to which model
    command.env("MODEL_UUID", model_id.to_string());

    // Create or open the stdout/stderr log file for writing
    let stdout_stderr_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&stdout_stderr_log_path)?;

    // Clone the file handle for stderr
    let stderr_file = stdout_stderr_file.try_clone()?;

    // Redirect stdout and stderr to the log file
    command
        .stdout(Stdio::from(stdout_stderr_file))
        .stderr(Stdio::from(stderr_file));

    println!("Starting mistralrs-server process: {:?}", command);
    println!("Process output will be logged to: {}", stdout_stderr_log_path);

    // Spawn the process
    let mut child = command.spawn()?;
    let pid = child.id();

    println!(
        "mistralrs-server process spawned with PID: {}, port: {}",
        pid, port
    );

    // Wait a bit to ensure the process has started successfully
    sleep(Duration::from_millis(100)).await;

    // Check if the process is still running
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process has already exited
            eprintln!(
                "mistralrs-server process exited immediately with status: {}",
                status
            );
            let error_msg = format!("mistralrs-server process failed to start: {}", status);
            return Ok(ModelStartResult::Failed { 
                error: error_msg, 
                stdout_stderr_log_path 
            });
        }
        Ok(None) => {
            // Process is still running, we'll store it properly in the registry later
            println!("mistralrs-server process is running, waiting for health check...");
        }
        Err(e) => {
            eprintln!("Failed to check mistralrs-server process status: {}", e);
            let error_msg = format!("Failed to check process status: {}", e);
            return Ok(ModelStartResult::Failed { 
                error: error_msg, 
                stdout_stderr_log_path 
            });
        }
    }

    // Calculate timeout based on model size
    let timeout_seconds = match calculate_model_size(&params.model_path) {
        Ok(size) => calculate_timeout_for_model_size(size),
        Err(e) => {
            eprintln!(
                "Failed to calculate model size: {}, using default timeout of 20 minutes",
                e
            );
            1200 // Default to 20 minutes if we can't calculate size
        }
    };

    // Wait for the model server to be healthy and ready
    if let Err(e) = wait_for_model_health(port, timeout_seconds).await {
        eprintln!("Model server health check failed: {}", e);
        // Try to stop the process if health check fails
        let _ = stop_model(model_id, pid, port).await;
        let error_msg = format!("Model server failed to become healthy: {}", e);
        return Ok(ModelStartResult::Failed { 
            error: error_msg, 
            stdout_stderr_log_path 
        });
    }

    println!("Model server is healthy and ready on port {}", port);

    // Register the process in our registry
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        let model_process = ModelProcess { child, pid, port };
        registry.insert(*model_id, model_process);
        println!(
            "Registered model {} with PID {} on port {}",
            model_id, pid, port
        );
    }

    Ok(ModelStartResult::Started { port, pid })
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
            println!("Found child process in registry, terminating gracefully...");
            
            // Try to kill the child process gracefully first
            match model_process.child.kill() {
                Ok(()) => {
                    println!("Sent kill signal to child process");
                    
                    // Wait for the process to exit and collect its status to prevent zombies
                    match model_process.child.wait() {
                        Ok(status) => {
                            println!("Child process exited with status: {}", status);
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("Error waiting for child process: {}", e);
                            // Continue with system-level termination as fallback
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error killing child process: {}", e);
                    // Continue with system-level termination as fallback
                }
            }
        }
    }

    // Fallback: system-level process termination for orphaned processes
    // Verify that the process is actually running and is a mistralrs server
    if !is_process_running(pid) {
        println!("Process {} is not running", pid);
        return Ok(());
    }

    // Verify that the process is actually a mistralrs server
    if !is_model_server_process(pid) {
        println!("Process {} is not a mistralrs server", pid);
        return Err("Process is not a mistralrs server".into());
    }

    // Verify that the port is actually in use (additional validation)
    if !is_port_in_use(port) {
        println!(
            "Port {} is not in use, process {} may be unresponsive",
            port, pid
        );
    }

    println!("Sending graceful termination signal to process {}", pid);

    // Try to terminate the process gracefully first
    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let pid_nix = Pid::from_raw(pid as i32);

        // Send SIGTERM first for graceful shutdown
        if let Err(e) = signal::kill(pid_nix, Signal::SIGTERM) {
            eprintln!("Failed to send SIGTERM to process {}: {}", pid, e);
            return Err(format!("Failed to send termination signal: {}", e).into());
        }

        // Wait for graceful shutdown with timeout
        let graceful_timeout_ms = 20000; // 20 seconds
        let check_interval_ms = 500; // Check every 200ms
        let max_checks = graceful_timeout_ms / check_interval_ms;

        for check in 0..max_checks {
            sleep(Duration::from_millis(check_interval_ms)).await;

            if !is_process_running(pid) {
                println!(
                    "Process {} terminated gracefully after {}ms",
                    pid,
                    (check + 1) * check_interval_ms
                );
                break;
            }

            // If this is the last check, force kill
            if check == max_checks - 1 {
                println!(
                    "Process {} did not respond to SIGTERM after {}ms, sending SIGKILL",
                    pid, graceful_timeout_ms
                );

                if let Err(e) = signal::kill(pid_nix, Signal::SIGKILL) {
                    eprintln!("Failed to send SIGKILL to process {}: {}", pid, e);
                    return Err(format!("Failed to force kill process: {}", e).into());
                }

                // Wait a bit more for force kill to take effect
                sleep(Duration::from_millis(1000)).await;

                if is_process_running(pid) {
                    eprintln!("Process {} is still running after SIGKILL", pid);
                    return Err("Process could not be terminated".into());
                } else {
                    println!("Process {} force killed successfully", pid);
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // On Windows, try graceful termination first
        let graceful_result = Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .output();

        match graceful_result {
            Ok(result) => {
                if result.status.success() {
                    // Wait for graceful shutdown
                    let mut graceful_success = false;
                    for _ in 0..25 {
                        // Check for 5 seconds (25 * 200ms)
                        sleep(Duration::from_millis(200)).await;
                        if !is_process_running(pid) {
                            graceful_success = true;
                            println!("Process {} terminated gracefully", pid);
                            break;
                        }
                    }

                    if !graceful_success {
                        println!(
                            "Process {} did not terminate gracefully, force killing",
                            pid
                        );
                        // Force terminate
                        let force_result = Command::new("taskkill")
                            .arg("/PID")
                            .arg(pid.to_string())
                            .arg("/F") // Force terminate
                            .output();

                        match force_result {
                            Ok(force_res) => {
                                if !force_res.status.success() {
                                    let stderr = String::from_utf8_lossy(&force_res.stderr);
                                    return Err(format!(
                                        "Failed to force terminate process {}: {}",
                                        pid, stderr
                                    )
                                    .into());
                                }
                            }
                            Err(e) => {
                                return Err(
                                    format!("Failed to execute force taskkill: {}", e).into()
                                );
                            }
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    return Err(format!("Failed to terminate process {}: {}", pid, stderr).into());
                }
            }
            Err(e) => {
                return Err(format!("Failed to execute taskkill: {}", e).into());
            }
        }
    }

    // Final verification that process is stopped
    if is_process_running(pid) {
        return Err("Process is still running after termination attempts".into());
    }

    println!("Process {} terminated successfully", pid);

    // Clean up any remaining registry entry (in case it wasn't removed earlier)
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        if registry.remove(model_id).is_some() {
            println!("Cleaned up remaining registry entry for model {}", model_id);
        }
    }

    Ok(())
}

/// Stop and cleanup a specific model by ID
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
                Ok(Some(status)) => {
                    println!("Process {} for model {} has exited with status: {}", 
                            model_process.pid, model_id, status);
                    dead_processes.push(*model_id);
                }
                Ok(None) => {
                    // Process is still running, check if it's actually alive via system call
                    if !is_process_running(model_process.pid) {
                        println!("Process {} for model {} appears dead but child handle didn't detect it", 
                                model_process.pid, model_id);
                        dead_processes.push(*model_id);
                    }
                }
                Err(e) => {
                    eprintln!("Error checking process {} for model {}: {}", 
                             model_process.pid, model_id, e);
                    dead_processes.push(*model_id);
                }
            }
        }
        
        // Remove dead processes from registry and wait on them to prevent zombies
        for model_id in dead_processes {
            if let Some(mut model_process) = registry.remove(&model_id) {
                println!("Cleaning up dead process {} for model {}", model_process.pid, model_id);
                // Wait on the child process to collect its exit status and prevent zombies
                let _ = model_process.child.wait();
            }
        }
    }
    
    Ok(())
}

/// Start a background task that periodically cleans up dead processes
/// This should be called once when the application starts
pub fn start_process_cleanup_task() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_dead_processes().await {
                eprintln!("Error during process cleanup: {}", e);
            }
        }
    });
    println!("Started background process cleanup task");
}

/// Cleanup all running model processes on application shutdown
/// This should be called when the application is shutting down to prevent orphaned processes
pub async fn cleanup_all_processes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Cleaning up all running model processes...");
    
    if let Ok(mut registry) = MODEL_REGISTRY.write() {
        let model_ids: Vec<Uuid> = registry.keys().cloned().collect();
        
        for model_id in model_ids {
            if let Some(mut model_process) = registry.remove(&model_id) {
                println!("Terminating process {} for model {}", model_process.pid, model_id);
                
                // Try to kill the process gracefully
                if let Err(e) = model_process.child.kill() {
                    eprintln!("Error killing process {} for model {}: {}", 
                             model_process.pid, model_id, e);
                }
                
                // Wait for the process to exit and collect its status
                match model_process.child.wait() {
                    Ok(status) => {
                        println!("Process {} for model {} exited with status: {}", 
                                model_process.pid, model_id, status);
                    }
                    Err(e) => {
                        eprintln!("Error waiting for process {} for model {}: {}", 
                                 model_process.pid, model_id, e);
                    }
                }
            }
        }
        
        registry.clear();
        println!("All model processes cleaned up");
    }
    
    Ok(())
}

use crate::ai::models::ModelUtils;
