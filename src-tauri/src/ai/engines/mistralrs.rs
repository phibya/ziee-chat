use super::{EngineError, EngineInstance, EngineType, LocalEngine};
use crate::database::models::model::Model;
use crate::utils::resource_paths::ResourcePaths;
use serde_json::Value;
use std::fs::{metadata, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

pub struct MistralRsEngine;

impl MistralRsEngine {
    pub fn new() -> Self {
        MistralRsEngine
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

        // Log details of large models or models with multiple files
        if model_files.len() > 1 || total_size > 1_000_000_000 {
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

    /// Read config.json and extract the number of layers
    fn get_model_layer_count(
        &self,
        model_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
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
        let config: Value = serde_json::from_str(&config_content)?;

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

    /// Find GGUF files in the given model directory
    fn find_gguf_files(
        &self,
        model_path: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let model_dir = Path::new(model_path);
        let mut gguf_files = Vec::new();

        if model_dir.is_file() && model_path.to_lowercase().ends_with(".gguf") {
            // If the path is directly a GGUF file, use just the filename
            if let Some(filename) = model_dir.file_name() {
                gguf_files.push(filename.to_string_lossy().to_string());
            }
        } else if model_dir.is_dir() {
            // Search directory for GGUF files
            for entry in std::fs::read_dir(model_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        if filename_str.to_lowercase().ends_with(".gguf") {
                            gguf_files.push(filename_str.to_string());
                        }
                    }
                }
            }
        }

        // Sort filenames for consistent ordering
        gguf_files.sort();

        println!(
            "Searching for GGUF files in {}: found {} files",
            model_path,
            gguf_files.len()
        );
        if !gguf_files.is_empty() {
            println!("GGUF files found: {}", gguf_files.join(", "));
        }

        Ok(gguf_files)
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

        // Check for weight files with numeric suffixes
        if file_name_lower.starts_with("model.")
            && file_name_lower.chars().any(|c| c.is_ascii_digit())
            && (file_name_lower.ends_with(".safetensors") || file_name_lower.ends_with(".bin"))
        {
            return true;
        }

        false
    }

    /// Calculate timeout based on model size
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
                    Ok(())
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

    async fn build_command_args(
        &self,
        model: &Model,
        port: u16,
    ) -> Result<Vec<String>, EngineError> {
        let mut args = Vec::new();

        // Get the model path
        let model_path_absolute = model.get_model_absolute_path();

        // Get mistralrs settings or use defaults
        let settings = model
            .engine_settings
            .as_ref()
            .and_then(|s| s.mistralrs.as_ref());

        // Add global arguments first

        // Server configuration
        args.extend(["--port".to_string(), port.to_string()]);

        if let Some(settings) = settings {
            if let Some(serve_ip) = &settings.serve_ip {
                args.extend(["--serve-ip".to_string(), serve_ip.clone()]);
            }
        }

        // Logging - create a log file for this model
        let log_path = if let Some(settings) = settings {
            settings.log_file.clone()
        } else {
            None
        }
        .unwrap_or_else(|| {
            let log_dir = crate::get_app_data_dir().join("logs/models");
            if !log_dir.exists() {
                let _ = std::fs::create_dir_all(&log_dir);
            }
            log_dir
                .join(format!("{}_mistralrs.log", model.id))
                .to_string_lossy()
                .to_string()
        });
        args.extend(["--log".to_string(), log_path]);

        // Add settings-based global arguments
        if let Some(settings) = settings {
            // Sequence management
            if settings.truncate_sequence.unwrap_or(false) {
                args.push("--truncate-sequence".to_string());
            }

            if let Some(max_seqs) = settings.max_seqs {
                args.extend(["--max-seqs".to_string(), max_seqs.to_string()]);
            }

            if settings.no_kv_cache.unwrap_or(false) {
                args.push("--no-kv-cache".to_string());
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
                    // Auto-detect best available device: CUDA > Metal > CPU
                    let available_devices = crate::ai::device_detection::detect_available_devices();
                    println!(
                        "Auto-detecting device type. Available devices: {} (default: {})",
                        available_devices.devices.len(),
                        available_devices.default_device_type
                    );

                    match available_devices.default_device_type.as_str() {
                        "cuda" => {
                            println!("Auto-selected CUDA device for optimal performance");
                            "cuda"
                        }
                        "metal" => {
                            println!("Auto-selected Metal device for optimal performance");
                            "metal"
                        }
                        _ => {
                            println!("Auto-selected CPU device (fallback)");
                            "cpu"
                        }
                    }
                }
            };

            // Force CPU mode if explicitly requested
            if settings.cpu.unwrap_or(false) || auto_detected_device_type == "cpu" {
                args.push("--cpu".to_string());
            }

            // Auto-select all available devices of the chosen type if device_ids not specified
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
                        device.device_type.as_str() == auto_detected_device_type
                            && device.is_available
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

            // Device layers configuration - use explicit num_device_layers or generate from final_device_ids
            if let Some(layers) = &settings.num_device_layers {
                args.extend(["--num-device-layers".to_string(), layers.join(";")]);
                println!(
                    "Using explicit device layer distribution: {}",
                    layers.join(", ")
                );
            } else if let Some(device_ids) = &final_device_ids {
                if !device_ids.is_empty() {
                    let is_cpu_mode = auto_detected_device_type == "cpu";
                    let is_cpu_forced = settings.cpu.unwrap_or(false);

                    if !is_cpu_mode && !is_cpu_forced {
                        // Only add --num-device-layers if there are multiple devices
                        if device_ids.len() > 1 {
                            // Try to get the actual layer count from the model
                            let layers_per_device = match self
                                .get_model_layer_count(&model_path_absolute)
                            {
                                Ok(total_layers) => {
                                    // Distribute layers evenly across devices
                                    let devices_count = device_ids.len();
                                    let base_layers = total_layers / devices_count;
                                    let remainder = total_layers % devices_count;

                                    println!(
                                        "Distributing {} layers across {} devices",
                                        total_layers, devices_count
                                    );

                                    device_ids
                                        .iter()
                                        .enumerate()
                                        .map(|(i, id)| {
                                            // Give extra layers to first devices if there's a remainder
                                            let layers = if i < remainder {
                                                base_layers + 1
                                            } else {
                                                base_layers
                                            };
                                            format!("{}:{}", id, layers)
                                        })
                                        .collect::<Vec<_>>()
                                }
                                Err(e) => {
                                    println!("Could not read layer count from config.json ({}), using default distribution", e);
                                    // Fallback to 32 layers per device
                                    device_ids
                                        .iter()
                                        .map(|id| format!("{}:32", id))
                                        .collect::<Vec<_>>()
                                }
                            };

                            let device_layers_str = layers_per_device.join(";");
                            args.extend(["--num-device-layers".to_string(), device_layers_str]);
                            println!(
                                "Device layer distribution: {}",
                                layers_per_device.join(", ")
                            );
                        }
                        // For single device, don't add --num-device-layers parameter at all
                    }
                }
            }

            // In-situ quantization
            if let Some(isq) = &settings.in_situ_quant {
                args.extend(["--isq".to_string(), isq.clone()]);
            }

            // PagedAttention configuration
            if let Some(gpu_mem) = settings.paged_attn_gpu_mem {
                args.extend(["--pa-gpu-mem".to_string(), gpu_mem.to_string()]);
            }

            if let Some(gpu_mem_usage) = settings.paged_attn_gpu_mem_usage {
                args.extend(["--pa-gpu-mem-usage".to_string(), gpu_mem_usage.to_string()]);
            }

            if let Some(ctxt_len) = settings.paged_ctxt_len {
                args.extend(["--pa-ctxt-len".to_string(), ctxt_len.to_string()]);
            }

            if let Some(block_size) = settings.paged_attn_block_size {
                args.extend(["--pa-blk-size".to_string(), block_size.to_string()]);
            }

            if settings.no_paged_attn.unwrap_or(false) {
                args.push("--no-paged-attn".to_string());
            }

            if settings.paged_attn.unwrap_or(false) {
                args.push("--paged-attn".to_string());
            }

            // Performance optimization
            if let Some(prefix_cache) = settings.prefix_cache_n {
                args.extend(["--prefix-cache-n".to_string(), prefix_cache.to_string()]);
            }

            if let Some(prompt_chunk) = settings.prompt_chunksize {
                args.extend(["--prompt-batchsize".to_string(), prompt_chunk.to_string()]);
            }

            // Seed for reproducibility
            if let Some(seed) = settings.seed {
                args.extend(["--seed".to_string(), seed.to_string()]);
            }

            // Chat templates
            if let Some(chat_template) = &settings.chat_template {
                args.extend(["--chat-template".to_string(), chat_template.clone()]);
            }

            if let Some(jinja) = &settings.jinja_explicit {
                args.extend(["--jinja-explicit".to_string(), jinja.clone()]);
            }

            // Token source
            if let Some(token_source) = &settings.token_source {
                args.extend(["--token-source".to_string(), token_source.clone()]);
            }

            // Interactive mode and thinking
            if settings.interactive_mode.unwrap_or(false) {
                args.push("--interactive-mode".to_string());
            }

            // Search capabilities
            if settings.enable_search.unwrap_or(false) {
                args.push("--enable-search".to_string());
            }

            if let Some(bert_model) = &settings.search_bert_model {
                args.extend(["--search-bert-model".to_string(), bert_model.clone()]);
            }

            if settings.enable_thinking.unwrap_or(false) {
                args.push("--enable-thinking".to_string());
            }
        }

        // Add the model subcommand based on model type
        let command = if model.file_format == crate::database::models::FileFormat::Gguf {
            // If file format is GGUF, force the command to be "gguf"
            crate::database::models::MistralRsCommand::Gguf
        } else {
            // Otherwise use the configured command or default to "run"
            settings
                .and_then(|s| s.command)
                .unwrap_or(crate::database::models::MistralRsCommand::Run) // Default to "run" (auto-loader)
        };

        match command {
            crate::database::models::MistralRsCommand::Plain => {
                args.push("plain".to_string());
                args.push("--model-id".to_string());
                if let Some(model_id_name) = settings.and_then(|s| s.model_id_name.as_ref()) {
                    args.push(model_id_name.clone());
                } else {
                    args.push(model_path_absolute.clone());
                }

                // Add plain-specific parameters
                if let Some(settings) = settings {
                    if let Some(tokenizer) = &settings.tokenizer_json {
                        args.extend(["--tokenizer-json".to_string(), tokenizer.clone()]);
                    }
                    if let Some(arch) = &settings.arch {
                        args.extend(["--arch".to_string(), arch.clone()]);
                    }
                    if let Some(dtype) = &settings.dtype {
                        args.extend(["--dtype".to_string(), dtype.clone()]);
                    }
                    if let Some(max_seq_len) = settings.max_seq_len {
                        args.extend(["--max-seq-len".to_string(), max_seq_len.to_string()]);
                    }
                }
            }
            crate::database::models::MistralRsCommand::Gguf => {
                args.push("gguf".to_string());
                args.extend([
                    "--quantized-model-id".to_string(),
                    model_path_absolute.clone(),
                ]);

                // Handle quantized filename
                let filename_specified = settings
                    .and_then(|s| s.quantized_filename.as_ref())
                    .cloned();

                if let Some(filename) = filename_specified {
                    args.extend(["--quantized-filename".to_string(), filename]);
                } else {
                    // Find GGUF files in the model directory
                    match self.find_gguf_files(&model_path_absolute) {
                        Ok(gguf_files) if !gguf_files.is_empty() => {
                            // Use all found GGUF files, space-separated
                            let filenames = gguf_files.join(" ");
                            args.extend(["--quantized-filename".to_string(), filenames]);
                            println!("Found GGUF files: {}", gguf_files.join(", "));
                        }
                        Ok(_) => {
                            // No GGUF files found, fallback to wildcard
                            args.extend(["--quantized-filename".to_string(), "*.gguf".to_string()]);
                            println!("No GGUF files found, using wildcard pattern");
                        }
                        Err(e) => {
                            println!("Error finding GGUF files: {}, using wildcard pattern", e);
                            args.extend(["--quantized-filename".to_string(), "*.gguf".to_string()]);
                        }
                    }
                }

                // Add GGUF-specific parameters
                if let Some(settings) = settings {
                    if let Some(dtype) = &settings.dtype {
                        args.extend(["--dtype".to_string(), dtype.clone()]);
                    }
                    if let Some(max_seq_len) = settings.max_seq_len {
                        args.extend(["--max-seq-len".to_string(), max_seq_len.to_string()]);
                    }
                }
            }
            crate::database::models::MistralRsCommand::VisionPlain => {
                args.push("vision-plain".to_string());
                args.push("--model-id".to_string());
                if let Some(model_id_name) = settings.and_then(|s| s.model_id_name.as_ref()) {
                    args.push(model_id_name.clone());
                } else {
                    args.push(model_path_absolute.clone());
                }

                // Add vision-specific parameters
                if let Some(settings) = settings {
                    if let Some(max_edge) = settings.max_edge {
                        args.extend(["--max-edge".to_string(), max_edge.to_string()]);
                    }
                    if let Some(max_images) = settings.max_num_images {
                        args.extend(["--max-num-images".to_string(), max_images.to_string()]);
                    }
                    if let Some(max_image_len) = settings.max_image_length {
                        args.extend(["--max-image-length".to_string(), max_image_len.to_string()]);
                    }
                    if let Some(dtype) = &settings.dtype {
                        args.extend(["--dtype".to_string(), dtype.clone()]);
                    }
                    if let Some(max_seq_len) = settings.max_seq_len {
                        args.extend(["--max-seq-len".to_string(), max_seq_len.to_string()]);
                    }
                }
            }
            crate::database::models::MistralRsCommand::XLora => {
                args.push("x-lora".to_string());
                args.push("--model-id".to_string());
                if let Some(model_id_name) = settings.and_then(|s| s.model_id_name.as_ref()) {
                    args.push(model_id_name.clone());
                } else {
                    args.push(model_path_absolute.clone());
                }
                // X-LoRA specific parameters would go here
            }
            crate::database::models::MistralRsCommand::Lora => {
                args.push("lora".to_string());
                args.push("--model-id".to_string());
                if let Some(model_id_name) = settings.and_then(|s| s.model_id_name.as_ref()) {
                    args.push(model_id_name.clone());
                } else {
                    args.push(model_path_absolute.clone());
                }
                // LoRA specific parameters would go here
            }
            crate::database::models::MistralRsCommand::Toml => {
                args.push("toml".to_string());
                args.push("--toml-path".to_string());
                args.push(model_path_absolute.clone());
            }
            crate::database::models::MistralRsCommand::Run => {
                // Default to run (auto-loader) for unknown model types
                args.push("run".to_string());
                args.push("--model-id".to_string());
                if let Some(model_id_name) = settings.and_then(|s| s.model_id_name.as_ref()) {
                    args.push(model_id_name.clone());
                } else {
                    args.push(model_path_absolute.clone());
                }

                // Add run-specific parameters (auto-loader)
                if let Some(settings) = settings {
                    if let Some(dtype) = &settings.dtype {
                        args.extend(["--dtype".to_string(), dtype.clone()]);
                    }
                    if let Some(max_seq_len) = settings.max_seq_len {
                        args.extend(["--max-seq-len".to_string(), max_seq_len.to_string()]);
                    }
                }
            }
        }

        Ok(args)
    }
}

#[async_trait::async_trait]
impl LocalEngine for MistralRsEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::MistralRs
    }

    fn name(&self) -> &'static str {
        "MistralRs"
    }

    fn version(&self) -> String {
        "0.3.3".to_string()
    }

    async fn start(&self, model: &Model) -> Result<EngineInstance, EngineError> {
        let port = Self::get_available_port();
        let model_uuid = model.id.to_string();

        // Calculate model size for timeout determination
        let model_size = self.calculate_model_size(model).unwrap_or(0);
        let timeout_seconds = self.calculate_timeout_for_model_size(model_size);

        println!(
          "Starting MistralRS model {} with timeout {} seconds ({:.1} minutes)",
          model.display_name,
          timeout_seconds,
          timeout_seconds as f64 / 60.0
        );

        // Build command arguments
        let args = self.build_command_args(model, port).await?;

        // Start the mistralrs-server process
        let binary_path =
            ResourcePaths::find_executable_binary("mistralrs-server").ok_or_else(|| {
                EngineError::StartupFailed("mistralrs-server binary not found".to_string())
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

        println!("Starting mistralrs-server process: {:?}", cmd);
        println!(
            "Process output will be logged to: {}",
            stdout_stderr_log_path
        );

        let mut child = cmd
            .spawn()
            .map_err(|e| EngineError::StartupFailed(format!("Failed to spawn process: {}", e)))?;

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
                return Err(EngineError::StartupFailed(format!(
                    "mistralrs-server process failed to start: {}",
                    status
                )));
            }
            Ok(None) => {
                // Process is still running, we'll store it properly in the registry later
                println!("mistralrs-server process is running, waiting for health check...");
            }
            Err(e) => {
                eprintln!("Failed to check mistralrs-server process status: {}", e);
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
