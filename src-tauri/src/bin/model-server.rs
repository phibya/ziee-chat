use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use tokio::signal;

// Import necessary modules from the main crate
use react_test_lib::ai::model_server::{create_model_server_router, ModelServerState};
use react_test_lib::APP_DATA_DIR;

#[derive(Parser, Debug)]
#[command(name = "model-server")]
#[command(about = "Standalone Candle model server with OpenAI-compatible API")]
struct Args {
    /// Model path relative to APP_DATA_DIR
    #[arg(long)]
    model_path: String,

    /// Model architecture (e.g., llama, mistral, gguf)
    #[arg(long)]
    architecture: String,

    /// Port to listen on
    #[arg(long)]
    port: u16,

    /// Model ID
    #[arg(long)]
    model_id: String,

    /// Model name (optional, defaults to model_id)
    #[arg(long)]
    model_name: Option<String>,

    /// APP_DATA_DIR override (for testing)
    #[arg(long)]
    app_data_dir: Option<String>,

    /// Config file path (relative to model_path)
    #[arg(long)]
    config_file: Option<String>,

    /// Tokenizer file path (relative to model_path)
    #[arg(long)]
    tokenizer_file: Option<String>,

    /// Primary weight file path (relative to model_path)
    #[arg(long)]
    weight_file: Option<String>,

    /// Additional weight files (comma-separated, relative to model_path)
    #[arg(long)]
    additional_weight_files: Option<String>,

    /// Vocab file path (relative to model_path)
    #[arg(long)]
    vocab_file: Option<String>,

    /// Special tokens file path (relative to model_path)
    #[arg(long)]
    special_tokens_file: Option<String>,

    /// Device type for inference (cpu, cuda, metal)
    #[arg(long)]
    device_type: Option<String>,

    /// Device IDs for inference (comma-separated, e.g. "0,1" for CUDA GPUs or "GPU-uuid1,GPU-uuid2")
    #[arg(long)]
    device_ids: Option<String>,

    /// Enable context shift to handle long prompts by shifting the context window
    #[arg(long)]
    enable_context_shift: bool,

    /// Enable continuous batching for improved throughput
    #[arg(long)]
    enable_continuous_batching: bool,

    /// Number of threads for batch processing (default: 4)
    #[arg(long, default_value = "4")]
    batch_threads: usize,

    /// Batch size for continuous batching (default: 4)
    #[arg(long, default_value = "4")]
    batch_size: usize,

    /// Batch timeout in milliseconds (default: 10)
    #[arg(long, default_value = "10")]
    batch_timeout_ms: u64,

    /// Maximum number of prompts that can be processed simultaneously (default: 8)
    #[arg(long, default_value = "8")]
    max_concurrent_prompts: usize,

    /// Number of CPU threads to use for inference when device type is cpu (default: 4)
    #[arg(long, default_value = "4")]
    cpu_threads: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Set up logging
    env_logger::init();

    println!("Starting model server...");
    println!("Model ID: {}", args.model_id);
    println!("Architecture: {}", args.architecture);
    println!("Port: {}", args.port);
    println!("Model Path: {}", args.model_path);
    
    // Print device configuration if provided
    if let Some(device_type) = &args.device_type {
        println!("Device Type: {}", device_type);
    }
    if let Some(device_ids) = &args.device_ids {
        println!("Device IDs: {}", device_ids);
    }
    println!("Context Shift: {}", args.enable_context_shift);
    println!("Continuous Batching: {}", args.enable_continuous_batching);
    println!("Batch Threads: {}", args.batch_threads);
    println!("Batch Size: {}", args.batch_size);
    println!("Batch Timeout: {}ms", args.batch_timeout_ms);
    println!("Max Concurrent Prompts: {}", args.max_concurrent_prompts);
    println!("CPU Threads: {}", args.cpu_threads);

    // Check if model_path is already absolute, otherwise join with base directory
    let full_model_path = if PathBuf::from(&args.model_path).is_absolute() {
        PathBuf::from(&args.model_path)
    } else {
        // Override APP_DATA_DIR if provided (for testing)
        let base_dir = if let Some(override_dir) = args.app_data_dir {
            PathBuf::from(override_dir)
        } else {
            APP_DATA_DIR.clone()
        };
        base_dir.join(&args.model_path)
    };

    println!("Full model path: {}", full_model_path.display());

    // Validate model path exists
    if !full_model_path.exists() {
        eprintln!(
            "Error: Model path does not exist: {}",
            full_model_path.display()
        );
        process::exit(1);
    }

    // Create PID file
    let pid_file = full_model_path.join(".model.pid");
    if let Err(e) = std::fs::write(&pid_file, process::id().to_string()) {
        eprintln!("Warning: Could not write PID file: {}", e);
    }

    // Create port file
    let port_file = full_model_path.join(".model.port");
    if let Err(e) = std::fs::write(&port_file, args.port.to_string()) {
        eprintln!("Warning: Could not write port file: {}", e);
    }

    // Create lock file
    let lock_file = full_model_path.join(".model.lock");
    if lock_file.exists() {
        eprintln!("Error: Model is already running (lock file exists)");
        process::exit(1);
    }
    if let Err(e) = std::fs::write(&lock_file, format!("{}:{}", process::id(), args.port)) {
        eprintln!("Error: Could not create lock file: {}", e);
        process::exit(1);
    }

    // Initialize model state
    let model_name = args.model_name.unwrap_or_else(|| args.model_id.clone());

    let model_state = if args.config_file.is_some() || args.tokenizer_file.is_some() || args.weight_file.is_some() {
        // Use specific file paths if provided
        match ModelServerState::new_with_specific_files_and_device(
            full_model_path.to_str().unwrap(),
            &args.architecture,
            &args.model_id,
            &model_name,
            args.config_file.as_deref(),
            args.tokenizer_file.as_deref(),
            args.weight_file.as_deref(),
            args.additional_weight_files.as_deref(),
            args.vocab_file.as_deref(),
            args.special_tokens_file.as_deref(),
            args.device_type.as_deref(),
            args.device_ids.as_deref(),
            args.enable_context_shift,
            args.enable_continuous_batching,
            args.batch_threads,
            args.batch_size,
            args.batch_timeout_ms,
            args.max_concurrent_prompts,
            args.cpu_threads,
        )
        .await
        {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Error: Failed to initialize model with specific files: {}", e);
                cleanup_files(&full_model_path);
                process::exit(1);
            }
        }
    } else {
        // Use auto-detection
        match ModelServerState::new_with_device_config(
            full_model_path.to_str().unwrap(),
            &args.architecture,
            &args.model_id,
            &model_name,
            args.device_type.as_deref(),
            args.device_ids.as_deref(),
            args.enable_context_shift,
            args.enable_continuous_batching,
            args.batch_threads,
            args.batch_size,
            args.batch_timeout_ms,
            args.max_concurrent_prompts,
            args.cpu_threads,
        )
        .await
        {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Error: Failed to initialize model: {}", e);
                cleanup_files(&full_model_path);
                process::exit(1);
            }
        }
    };

    println!("Model loaded successfully!");

    // Create router
    let app = create_model_server_router(model_state);

    // Setup graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error: Failed to bind to port {}: {}", args.port, e);
            cleanup_files(&full_model_path);
            process::exit(1);
        }
    };

    println!("Model server listening on http://{}", addr);
    println!("Health check: http://{}/health", addr);
    println!("OpenAI API: http://{}/v1/chat/completions", addr);

    // Setup signal handling for graceful shutdown
    let model_path_for_cleanup = full_model_path.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        println!("Shutdown signal received, cleaning up...");
        cleanup_files(&model_path_for_cleanup);
        process::exit(0);
    });

    // Start server
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Error: Server error: {}", e);
        cleanup_files(&full_model_path);
        process::exit(1);
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn cleanup_files(model_path: &PathBuf) {
    let files_to_remove = [".model.lock", ".model.pid", ".model.port"];

    for file in &files_to_remove {
        let file_path = model_path.join(file);
        if file_path.exists() {
            if let Err(e) = std::fs::remove_file(&file_path) {
                eprintln!("Warning: Could not remove {}: {}", file_path.display(), e);
            }
        }
    }

    println!("Cleanup completed");
}
