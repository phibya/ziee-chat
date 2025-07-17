use super::super::candle::CandleModel;
use super::super::inference::{CachePool, InferenceRequest};
use super::config::{ModelConfig, TokenizerConfig};
use candle_core::Device;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex, Semaphore};

pub struct ModelServerState {
    pub model: Arc<Mutex<Box<dyn CandleModel + Send + Sync>>>,
    pub tokenizer: Arc<tokenizers::Tokenizer>,
    pub model_id: String,
    pub model_name: String,
    pub architecture: String,
    pub started_at: i64,
    pub tokenizer_config: TokenizerConfig,
    pub model_config: ModelConfig,
    pub device: Device,
    pub cache_pool: Arc<CachePool>,
    pub inference_tx: mpsc::UnboundedSender<InferenceRequest>,
    pub enable_context_shift: bool,
    pub enable_continuous_batching: bool,
    pub batch_threads: usize,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub max_concurrent_prompts: usize,
    pub concurrent_request_semaphore: Arc<Semaphore>,
    pub cpu_threads: usize,
    pub flash_attention: bool,
    pub kv_cache_type: String,
    pub paged_attention: bool,
    pub mmap: bool,
    pub auto_unload_model: bool,
    pub auto_unload_minutes: u64,
    pub model_loaded: Arc<Mutex<bool>>,
    pub last_request_time: Arc<Mutex<Instant>>,
}

impl ModelServerState {
    /// Configure CPU thread limit for CPU inference
    pub fn configure_cpu_threads(device: &Device, cpu_threads: usize) {
        // Only configure CPU threads if the device is CPU
        if matches!(device, Device::Cpu) {
            println!("Configuring CPU threads for inference: {}", cpu_threads);

            // Set the number of threads for CPU inference
            std::env::set_var("RAYON_NUM_THREADS", cpu_threads.to_string());
            std::env::set_var("OMP_NUM_THREADS", cpu_threads.to_string());
            std::env::set_var("MKL_NUM_THREADS", cpu_threads.to_string());
        }
    }

    /// Configure flash attention for optimized memory usage and faster inference
    pub fn configure_flash_attention(flash_attention: bool) {
        if flash_attention {
            println!("Enabling flash attention for optimized memory usage and faster inference");
            std::env::set_var("ENABLE_FLASH_ATTENTION", "1");
            std::env::set_var("FLASH_ATTENTION_ENABLED", "true");
            std::env::set_var("PYTORCH_FLASH_ATTENTION", "1");
            std::env::set_var("TRANSFORMERS_FLASH_ATTENTION", "1");
            std::env::set_var("HF_FLASH_ATTENTION", "1");
            std::env::set_var("FLASH_ATTENTION_MEMORY_EFFICIENT", "1");
        } else {
            println!("Flash attention is disabled");
            std::env::set_var("ENABLE_FLASH_ATTENTION", "0");
            std::env::set_var("FLASH_ATTENTION_ENABLED", "false");
            std::env::set_var("PYTORCH_FLASH_ATTENTION", "0");
            std::env::set_var("TRANSFORMERS_FLASH_ATTENTION", "0");
            std::env::set_var("HF_FLASH_ATTENTION", "0");
            std::env::set_var("FLASH_ATTENTION_MEMORY_EFFICIENT", "0");
        }
    }

    /// Configure KV Cache Type for controlling memory usage and precision trade-off
    pub fn configure_kv_cache_type(kv_cache_type: &str) {
        println!("Configuring KV Cache Type: {}", kv_cache_type);

        std::env::set_var("KV_CACHE_TYPE", kv_cache_type);
        std::env::set_var("CANDLE_KV_CACHE_TYPE", kv_cache_type);
        std::env::set_var("PYTORCH_KV_CACHE_TYPE", kv_cache_type);
        std::env::set_var("TRANSFORMERS_KV_CACHE_TYPE", kv_cache_type);

        match kv_cache_type.to_lowercase().as_str() {
            "f32" => {
                std::env::set_var("KV_CACHE_PRECISION", "f32");
                std::env::set_var("FORCE_F32_PRECISION", "1");
                println!("KV Cache configured for f32 precision (high memory usage, highest precision)");
            }
            "f16" => {
                std::env::set_var("KV_CACHE_PRECISION", "f16");
                std::env::set_var("FORCE_F16_PRECISION", "1");
                println!("KV Cache configured for f16 precision (balanced memory usage and precision)");
            }
            "bf16" => {
                std::env::set_var("KV_CACHE_PRECISION", "bf16");
                std::env::set_var("FORCE_BF16_PRECISION", "1");
                println!("KV Cache configured for bf16 precision (memory efficient, good precision)");
            }
            "i8" => {
                std::env::set_var("KV_CACHE_PRECISION", "i8");
                std::env::set_var("FORCE_INT8_PRECISION", "1");
                println!("KV Cache configured for i8 precision (very memory efficient, quantized)");
            }
            "auto" => {
                std::env::set_var("KV_CACHE_PRECISION", "auto");
                println!("KV Cache configured for auto precision (automatically determined)");
            }
            _ => {
                println!("Warning: Unknown KV cache type '{}', using default f16", kv_cache_type);
                std::env::set_var("KV_CACHE_PRECISION", "f16");
                std::env::set_var("FORCE_F16_PRECISION", "1");
            }
        }
    }

    /// Configure Paged Attention for efficient memory usage and better batching performance
    pub fn configure_paged_attention(paged_attention: bool) {
        if paged_attention {
            println!("Enabling Paged Attention for efficient memory usage and better batching performance");
            std::env::set_var("ENABLE_PAGED_ATTENTION", "1");
            std::env::set_var("PAGED_ATTENTION_ENABLED", "true");
            std::env::set_var("USE_PAGED_ATTENTION", "1");
            std::env::set_var("TRANSFORMERS_PAGED_ATTENTION", "1");
        } else {
            println!("Paged Attention is disabled");
            std::env::set_var("ENABLE_PAGED_ATTENTION", "0");
            std::env::set_var("PAGED_ATTENTION_ENABLED", "false");
            std::env::set_var("USE_PAGED_ATTENTION", "0");
            std::env::set_var("TRANSFORMERS_PAGED_ATTENTION", "0");
        }
    }

    /// Configure Memory Mapping (mmap) for efficient model file loading
    pub fn configure_mmap(mmap: bool) {
        if mmap {
            println!("Enabling Memory Mapping (mmap) for efficient model file loading and reduced RAM usage");
            std::env::set_var("ENABLE_MMAP", "1");
            std::env::set_var("USE_MEMORY_MAPPING", "true");
            std::env::set_var("TORCH_USE_MMAP", "1");
            std::env::set_var("TRANSFORMERS_USE_MMAP", "1");
        } else {
            println!("Memory Mapping (mmap) is disabled - using traditional file loading");
            std::env::set_var("ENABLE_MMAP", "0");
            std::env::set_var("USE_MEMORY_MAPPING", "false");
            std::env::set_var("TORCH_USE_MMAP", "0");
            std::env::set_var("TRANSFORMERS_USE_MMAP", "0");
        }
    }

    /// Configure auto model unloading
    pub fn configure_auto_unload(auto_unload: bool, auto_unload_minutes: u64) {
        if auto_unload {
            println!("Enabling auto unload - model will unload after {} minutes of inactivity", auto_unload_minutes);
            std::env::set_var("AUTO_UNLOAD_ENABLED", "1");
            std::env::set_var("AUTO_UNLOAD_MINUTES", auto_unload_minutes.to_string());
        } else {
            println!("Auto unload is disabled - model will remain loaded");
            std::env::set_var("AUTO_UNLOAD_ENABLED", "0");
        }
    }
}