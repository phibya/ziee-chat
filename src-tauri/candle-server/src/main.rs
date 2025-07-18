use axum::{
    http::{self, Method, StatusCode},
    response::Json as AxumJson,
    routing::{get, post},
    Router,
};
use candle_core::{DType, Device};
use clap::Parser;
use serde_json::json;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{path::PathBuf, sync::Arc};
use tokio::signal;
use tokio::sync::Notify;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{error, info, warn};
use candle_server::management::get_model_manager;
use candle_server::openai::models::Config;
use candle_server::openai::openai_server::chat_completions;
use candle_server::openai::pipelines::llm_engine::LLMEngine;
use candle_server::openai::pipelines::pipeline::DefaultModelPaths;
use candle_server::openai::responses::APIError;
use candle_server::openai::OpenAIServerData;
use candle_server::scheduler::cache_engine::{CacheConfig, CacheEngine};
use candle_server::scheduler::SchedulerConfig;
use candle_server::{get_model_loader, hub_load_local_safetensors, ModelSelected};

const SIZE_IN_MB: usize = 1024 * 1024;

// Global shutdown flag
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Check if a model is already running in the given path using ModelManager
async fn is_model_already_running(
    model_path: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let manager = get_model_manager();
    manager.is_model_already_running(model_path).await
}

/// Create a lock file for the model using ModelManager
async fn create_lock_file(
    model_path: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manager = get_model_manager();
    manager.create_lock_file(model_path, port).await
}

/// Remove the lock file for the model using ModelManager
async fn remove_lock_file(
    model_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manager = get_model_manager();
    manager.remove_lock_file_public(model_path).await
}

/// Setup graceful shutdown handler
async fn setup_shutdown_handler(model_path: Option<String>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received terminate signal, initiating graceful shutdown...");
        }
    }

    // Set shutdown flag
    SHUTDOWN.store(true, Ordering::Relaxed);

    // Clean up lock file if we have a model path
    if let Some(model_path) = model_path {
        info!("Cleaning up lock file...");
        if let Err(e) = remove_lock_file(&model_path).await {
            error!("Failed to remove lock file: {}", e);
        } else {
            info!("Lock file cleaned up successfully");
        }
    }

    info!("Graceful shutdown complete");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "candle_server-server")]
#[command(about = "Candle-vLLM model server with OpenAI-compatible API")]
struct Args {
    /// Port to serve on (localhost:port)
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Set verbose mode (print all requests)
    #[arg(long)]
    verbose: bool,

    #[clap(subcommand)]
    command: ModelSelected,

    /// Maximum number of sequences to allow
    #[arg(long, default_value_t = 256)]
    max_num_seqs: usize,

    /// Size of a block
    #[arg(long, default_value_t = 32)]
    block_size: usize,

    /// Model identifier
    #[arg(long)]
    model_id: Option<String>,

    /// The folder name that contains safetensor weights and json files
    /// (same structure as huggingface online), path must include last "/"
    #[arg(long)]
    weight_path: Option<String>,

    /// The quantized weight file name (for gguf/ggml file)
    #[arg(long)]
    weight_file: Option<String>,

    #[arg(long)]
    dtype: Option<String>,

    #[arg(long, default_value_t = false)]
    cpu: bool,

    /// Available GPU memory for kvcache (MB)
    #[arg(long, default_value_t = 4096)]
    kvcache_mem_gpu: usize,

    /// Available CPU memory for kvcache (MB)
    #[arg(long, default_value_t = 128)]
    kvcache_mem_cpu: usize,

    /// Record conversation (default false, the client need to record chat history)
    #[arg(long)]
    record_conversation: bool,

    #[arg(long, value_delimiter = ',')]
    device_ids: Option<Vec<usize>>,

    /// Maximum waiting time for processing parallel requests (in milliseconds).
    /// A larger value means the engine can hold more requests and process them in a single generation call.
    #[arg(long, default_value_t = 500)]
    holding_time: usize,

    /// Whether the program running in multiprocess or multithread model for parallel inference
    #[arg(long, default_value_t = false)]
    multi_process: bool,

    #[arg(long, default_value_t = false)]
    log: bool,
}

fn get_cache_config(
    kvcache_mem_gpu: usize,
    kvcache_mem_cpu: usize,
    block_size: usize,
    config: &Config,
    num_shards: usize,
) -> CacheConfig {
    let dsize = config.kv_cache_dtype.size_in_bytes();
    let num_gpu_blocks = kvcache_mem_gpu * SIZE_IN_MB
        / dsize
        / block_size
        / (config.num_key_value_heads / num_shards)
        / config.k_head_dim()
        / config.num_hidden_layers
        / 2;
    let num_cpu_blocks = kvcache_mem_cpu * SIZE_IN_MB
        / dsize
        / block_size
        / (config.num_key_value_heads / num_shards)
        / config.k_head_dim()
        / config.num_hidden_layers
        / 2;
    CacheConfig {
        block_size,
        num_gpu_blocks: Some(num_gpu_blocks),
        num_cpu_blocks: Some(num_cpu_blocks),
        fully_init: true,
        dtype: config.kv_cache_dtype,
    }
}

/// Health check endpoint to verify if the model server is ready
async fn health_check() -> Result<AxumJson<serde_json::Value>, StatusCode> {
    Ok(AxumJson(json!({
        "status": "healthy",
        "message": "Model server is ready",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check endpoint - returns 200 when model is loaded and ready
async fn readiness_check() -> Result<AxumJson<serde_json::Value>, StatusCode> {
    // For now, we'll assume the model is ready if the server is running
    // In a more sophisticated implementation, we could check the actual model state
    Ok(AxumJson(json!({
        "status": "ready",
        "message": "Model is loaded and ready to serve requests",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[tokio::main]
async fn main() -> Result<(), APIError> {
    let args = Args::parse();

    // Set up panic hook to ensure graceful shutdown on panic
    let model_path_for_panic = args.weight_path.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("PANIC occurred: {}", panic_info);
        
        // Set shutdown flag
        SHUTDOWN.store(true, Ordering::Relaxed);
        
        // Try to clean up lock file if we have a model path
        if let Some(ref model_path) = model_path_for_panic {
            if let Err(e) = std::fs::remove_file(std::path::Path::new(model_path).join(".model.lock")) {
                error!("Failed to clean up lock file after panic: {}", e);
            } else {
                error!("Lock file cleaned up after panic");
            }
        }
        
        error!("Server shutting down due to panic...");
        std::process::exit(1);
    }));

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting candle_server-vLLM model server...");
    info!("Port: {}", args.port);

    // Check if model is already running and create lock file early
    if let Some(model_path) = &args.weight_path {
        match is_model_already_running(model_path).await {
            Ok(true) => {
                error!("Model is already running at path: {}", model_path);
                return Err(APIError::new(format!(
                    "Model is already running at path: {}",
                    model_path
                )));
            }
            Ok(false) => {
                // Create lock file immediately to prevent other instances
                if let Err(e) = create_lock_file(model_path, args.port).await {
                    error!("Failed to create lock file: {}", e);
                    return Err(APIError::new(format!("Failed to create lock file: {}", e)));
                }
            }
            Err(e) => {
                error!("Failed to check if model is already running: {}", e);
                return Err(APIError::new(format!(
                    "Failed to check if model is already running: {}",
                    e
                )));
            }
        }
    }

    // Bind to port immediately to prevent conflicts
    info!("Binding to port {} to prevent conflicts...", args.port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .map_err(|e| APIError::new(format!("Failed to bind to port {}: {}", args.port, e)))?;

    info!(
        "Port {} successfully bound, starting model loading...",
        args.port
    );

    // Setup shutdown handler
    let shutdown_model_path = args.weight_path.clone();
    tokio::spawn(async move {
        setup_shutdown_handler(shutdown_model_path).await;
    });

    let (loader, model_id, quant) = get_model_loader(args.command, args.model_id.clone());
    if args.model_id.is_none() && args.weight_path.is_none() && args.weight_file.is_none() {
        info!("No model id specified, using the default model_id or specified in the weight_path to retrieve config files!");
    }

    let paths = match (&args.weight_path, &args.weight_file) {
        // Model in a folder (safetensor format, huggingface folder structure)
        (Some(path), None) => DefaultModelPaths {
            tokenizer_filename: Path::new(path).join("tokenizer.json"),
            tokenizer_config_filename: Path::new(path).join("tokenizer_config.json"),
            config_filename: Path::new(path).join("config.json"),
            filenames: if Path::new(path)
                .join("model.safetensors.index.json")
                .exists()
            {
                hub_load_local_safetensors(path, "model.safetensors.index.json").unwrap()
            } else {
                // A single weight file case
                let mut safetensors_files = Vec::<std::path::PathBuf>::new();
                safetensors_files.insert(0, Path::new(path).join("model.safetensors"));
                safetensors_files
            },
        },
        // Model in a quantized file (gguf/ggml format)
        (path, Some(file)) => DefaultModelPaths {
            tokenizer_filename: PathBuf::new(),
            tokenizer_config_filename: PathBuf::new(),
            config_filename: PathBuf::new(),
            filenames: {
                let path = path.clone().unwrap_or("".to_string());
                if Path::new(&path).join(file).exists() {
                    vec![Path::new(&path).join(file)]
                } else {
                    panic!("Model file not found {file}");
                }
            },
        },
        _ => {
            // Try download model
            loader.download_model(
                model_id.clone(),
                args.weight_file.clone(),
                quant.clone(),
                None,
                None, // No HF token for now
                None, // No HF token path for now
            )?
        }
    };

    let dtype = match args.dtype.as_deref() {
        Some("f16") => DType::F16,
        Some("bf16") => DType::BF16,
        Some("f32") => DType::F32,
        Some(dtype) => panic!("Unsupported dtype {dtype}"),
        None => DType::BF16,
    };

    let device_ids: Vec<usize> = match args.device_ids {
        Some(ids) => ids,
        _ => vec![0usize],
    };
    let num_shards = device_ids.len();

    // For simplicity, we'll use single-threaded mode for now
    let (pipelines, _global_rank) = (
        loader
            .load_model(paths, dtype, &quant, device_ids, None, None)
            .await,
        0,
    );

    let (default_pipelines, pipeline_config) = match pipelines {
        Err(e) => panic!("{e:?}"),
        Ok((p, c)) => (p, c),
    };

    let mut config: Option<Config> = None;
    let mut cache_config: Option<CacheConfig> = None;

    let pipelines = default_pipelines
        .into_iter()
        .map(|pipeline| {
            let cfg = pipeline.get_model_config();
            let cache_cfg = get_cache_config(
                args.kvcache_mem_gpu,
                args.kvcache_mem_cpu,
                args.block_size,
                &cfg,
                num_shards,
            );
            let cache_engine = CacheEngine::new(
                &cfg,
                &cache_cfg,
                cache_cfg.dtype,
                pipeline.device(),
                num_shards,
            )
            .unwrap();
            if config.is_none() {
                config = Some(cfg.clone());
            }
            if cache_config.is_none() {
                cache_config = Some(cache_cfg.clone());
            }
            (pipeline.rank(), (pipeline, cache_engine))
        })
        .collect();

    let cache_config = cache_config.as_ref().unwrap().clone();
    let config = config.as_ref().unwrap().clone();
    info!("Cache config {:?}", cache_config);

    let llm_engine = LLMEngine::new(
        pipelines,
        SchedulerConfig {
            max_num_seqs: args.max_num_seqs,
        },
        &cache_config,
        &config,
        Arc::new(Notify::new()),
        args.holding_time,
        num_shards,
        args.multi_process,
    )?;

    let max_model_len = pipeline_config.max_model_len;
    let kvcached_tokens = cache_config.num_gpu_blocks.unwrap() * cache_config.block_size;
    let server_data = OpenAIServerData {
        pipeline_config,
        model: llm_engine,
        record_conversation: args.record_conversation,
        device: Device::Cpu,
    };

    warn!("Maximum Model Length (affected by `--kvcache-mem-gpu` and the number of ranks):");
    for batch in [1, 8] {
        println!(
            "-> Batch {}: {}",
            batch,
            std::cmp::min(kvcached_tokens / batch, max_model_len)
        );
    }
    warn!("Server started at http://0.0.0.0:{}.", args.port);

    let allow_origin = AllowOrigin::any();
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([http::header::CONTENT_TYPE])
        .allow_origin(allow_origin);

    let app = Router::new()
        .layer(cors_layer)
        .route("/v1/chat/completions", post(chat_completions))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(Arc::new(server_data));

    // Use the pre-bound listener
    info!("Starting HTTP server with pre-bound listener...");

    // Serve with graceful shutdown
    let server = axum::serve(listener, app).with_graceful_shutdown(async {
        // Wait for shutdown signal
        while !SHUTDOWN.load(Ordering::Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        info!("Shutdown signal received, stopping server...");
    });

    // Run the server
    let result = server.await;

    match result {
        Ok(()) => {
            info!("Server stopped gracefully");
        }
        Err(e) => {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
    }

    info!("Model server shutdown complete");
    std::process::exit(0);
}
