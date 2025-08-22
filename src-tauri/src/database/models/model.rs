use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::download_instance::SourceInfo;
use crate::api::engines::EngineType;

/// Device types for ML model inference
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// CPU-only inference
    Cpu,
    /// NVIDIA CUDA GPU acceleration
    Cuda,
    /// Apple Metal GPU acceleration (macOS)
    Metal,
    /// AMD ROCm GPU acceleration
    Rocm,
    /// Vulkan GPU acceleration
    Vulkan,
    /// OpenCL GPU acceleration
    Opencl,
    /// Automatic device detection and selection
    Auto,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Cpu => "cpu",
            DeviceType::Cuda => "cuda",
            DeviceType::Metal => "metal",
            DeviceType::Rocm => "rocm",
            DeviceType::Vulkan => "vulkan",
            DeviceType::Opencl => "opencl",
            DeviceType::Auto => "auto",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cpu" => Some(DeviceType::Cpu),
            "cuda" => Some(DeviceType::Cuda),
            "metal" => Some(DeviceType::Metal),
            "rocm" => Some(DeviceType::Rocm),
            "vulkan" => Some(DeviceType::Vulkan),
            "opencl" => Some(DeviceType::Opencl),
            "auto" => Some(DeviceType::Auto),
            _ => None,
        }
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// MistralRS command types for different model formats and use cases
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MistralRsCommand {
    /// Plain model format (safetensors/pytorch)
    Plain,
    /// GGUF quantized model format
    Gguf,
    /// Auto-loader for various model formats
    Run,
    /// Vision-enabled plain models for multimodal capabilities
    VisionPlain,
    /// X-LoRA (Cross-Layer LoRA) models
    XLora,
    /// LoRA (Low-Rank Adaptation) models
    Lora,
    /// TOML configuration-based models
    Toml,
}

impl MistralRsCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            MistralRsCommand::Plain => "plain",
            MistralRsCommand::Gguf => "gguf",
            MistralRsCommand::Run => "run",
            MistralRsCommand::VisionPlain => "vision-plain",
            MistralRsCommand::XLora => "x-lora",
            MistralRsCommand::Lora => "lora",
            MistralRsCommand::Toml => "toml",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "plain" => Some(MistralRsCommand::Plain),
            "gguf" => Some(MistralRsCommand::Gguf),
            "run" => Some(MistralRsCommand::Run),
            "vision-plain" => Some(MistralRsCommand::VisionPlain),
            "x-lora" => Some(MistralRsCommand::XLora),
            "lora" => Some(MistralRsCommand::Lora),
            "toml" => Some(MistralRsCommand::Toml),
            _ => None,
        }
    }
}

impl std::fmt::Display for MistralRsCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// File format types for local models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Safetensors,
    Pytorch,
    Gguf,
}

impl FileFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileFormat::Safetensors => "safetensors",
            FileFormat::Pytorch => "pytorch",
            FileFormat::Gguf => "gguf",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "safetensors" => Some(FileFormat::Safetensors),
            "pytorch" => Some(FileFormat::Pytorch),
            "gguf" => Some(FileFormat::Gguf),
            _ => None,
        }
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Model capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ModelCapabilities {
    /// Vision capability - can process images
    pub vision: Option<bool>,
    /// Audio capability - can process audio
    pub audio: Option<bool>,
    /// Tools capability - can use function calling/tools
    pub tools: Option<bool>,
    /// Code interpreter capability
    pub code_interpreter: Option<bool>,
}

impl ModelCapabilities {
    /// Create new capabilities with all disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create capabilities for a vision model
    pub fn vision_enabled() -> Self {
        Self {
            vision: Some(true),
            audio: Some(false),
            tools: Some(false),
            code_interpreter: Some(false),
        }
    }

    /// Create capabilities for a code model
    pub fn code_enabled() -> Self {
        Self {
            vision: Some(false),
            audio: Some(false),
            tools: Some(true),
            code_interpreter: Some(true),
        }
    }
}

/// Model parameters for inference configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ModelParameters {
    // Context and generation parameters
    /// Context size for the model
    pub max_tokens: Option<u32>,

    // Sampling parameters
    /// Temperature for randomness (0.0-2.0)
    pub temperature: Option<f32>,
    /// Top-K sampling parameter
    pub top_k: Option<u32>,
    /// Top-P (nucleus) sampling parameter (0.0-1.0)
    pub top_p: Option<f32>,
    /// Min-P sampling parameter (0.0-1.0)
    pub min_p: Option<f32>,

    // Repetition control
    /// Number of last tokens to consider for repetition penalty
    pub repeat_last_n: Option<u32>,
    /// Repetition penalty (1.0 = no penalty)
    pub repeat_penalty: Option<f32>,
    /// Presence penalty for new tokens
    pub presence_penalty: Option<f32>,
    /// Frequency penalty for repeated tokens
    pub frequency_penalty: Option<f32>,

    // Generation control
    /// Random seed for reproducible outputs
    pub seed: Option<i32>,
    /// Stop sequences to terminate generation
    pub stop: Option<Vec<String>>,
}

impl ModelParameters {
    /// Create new parameters with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create parameters optimized for creative text generation
    pub fn creative() -> Self {
        Self {
            max_tokens: Some(4096),
            temperature: Some(0.8),
            top_k: Some(40),
            top_p: Some(0.95),
            min_p: Some(0.05),
            repeat_last_n: Some(64),
            repeat_penalty: Some(1.1),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            seed: None,
            stop: None,
        }
    }

    /// Create parameters optimized for precise/factual generation
    pub fn precise() -> Self {
        Self {
            max_tokens: Some(4096),
            temperature: Some(0.2),
            top_k: Some(20),
            top_p: Some(0.9),
            min_p: Some(0.1),
            repeat_last_n: Some(64),
            repeat_penalty: Some(1.05),
            presence_penalty: Some(0.1),
            frequency_penalty: Some(0.1),
            seed: None,
            stop: None,
        }
    }

    /// Validate the parameters and return errors if any
    pub fn validate(&self) -> Result<(), String> {
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err("temperature must be between 0.0 and 2.0".to_string());
            }
        }

        if let Some(top_p) = self.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err("top_p must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(min_p) = self.min_p {
            if min_p < 0.0 || min_p > 1.0 {
                return Err("min_p must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(repeat_penalty) = self.repeat_penalty {
            if repeat_penalty < 0.0 || repeat_penalty > 2.0 {
                return Err("repeat_penalty must be between 0.0 and 2.0".to_string());
            }
        }

        if let Some(presence_penalty) = self.presence_penalty {
            if presence_penalty < -2.0 || presence_penalty > 2.0 {
                return Err("presence_penalty must be between -2.0 and 2.0".to_string());
            }
        }

        if let Some(frequency_penalty) = self.frequency_penalty {
            if frequency_penalty < -2.0 || frequency_penalty > 2.0 {
                return Err("frequency_penalty must be between -2.0 and 2.0".to_string());
            }
        }

        if let Some(stop) = &self.stop {
            if stop.len() > 4 {
                return Err("stop sequences cannot exceed 4 items".to_string());
            }
            for stop_seq in stop {
                if stop_seq.is_empty() {
                    return Err("stop sequences cannot be empty".to_string());
                }
                if stop_seq.len() > 32 {
                    return Err("stop sequences cannot exceed 32 characters each".to_string());
                }
            }
        }

        Ok(())
    }
}

/// MistralRs-specific settings for individual model performance and batching configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct MistralRsSettings {
    // Core model configuration
    /// Model command type for MistralRS engine
    pub command: Option<MistralRsCommand>,
    /// Model ID name for --model-id in subcommands
    pub model_id_name: Option<String>,
    /// Path to tokenizer.json file
    pub tokenizer_json: Option<String>,
    /// Model architecture (for plain models)
    pub arch: Option<String>,

    // Quantization and weights
    /// GGUF filename pattern (for GGUF models)
    pub quantized_filename: Option<String>,
    /// Specific weight file
    pub weight_file: Option<String>,

    // Device configuration
    /// Device type (cpu, cuda, metal, etc.)
    pub device_type: Option<DeviceType>,
    /// Array of device IDs for multi-GPU
    pub device_ids: Option<Vec<i32>>,
    /// Per-device layer distribution
    pub num_device_layers: Option<Vec<String>>,
    /// Force CPU mode
    pub cpu: Option<bool>,

    // Sequence and memory management
    /// Maximum running sequences at any time (--max-seqs)
    pub max_seqs: Option<usize>,
    /// Maximum sequence length (--max-seq-len)
    pub max_seq_len: Option<usize>,
    /// Use no KV cache (--no-kv-cache)
    pub no_kv_cache: Option<bool>,
    /// Truncate sequences that exceed max length (--truncate-sequence)
    pub truncate_sequence: Option<bool>,

    // PagedAttention configuration
    /// GPU memory for KV cache in MBs (--pa-gpu-mem)
    pub paged_attn_gpu_mem: Option<usize>,
    /// GPU memory usage percentage 0-1 (--pa-gpu-mem-usage)
    pub paged_attn_gpu_mem_usage: Option<f32>,
    /// Total context length for KV cache (--pa-ctxt-len)
    pub paged_ctxt_len: Option<usize>,
    /// PagedAttention block size (--pa-blk-size)
    pub paged_attn_block_size: Option<usize>,
    /// Disable PagedAttention on CUDA (--no-paged-attn)
    pub no_paged_attn: Option<bool>,
    /// Enable PagedAttention on Metal (--paged-attn)
    pub paged_attn: Option<bool>,

    // Chat and templates
    /// Chat template string
    pub chat_template: Option<String>,
    /// Jinja template explicit definition
    pub jinja_explicit: Option<String>,

    // Performance optimization
    /// Number of prefix caches to hold (--prefix-cache-n)
    pub prefix_cache_n: Option<usize>,
    /// Prompt batching chunk size (--prompt-batchsize)
    pub prompt_chunksize: Option<usize>,

    // Model configuration
    /// Model data type: auto, f16, f32, bf16 (--dtype)
    pub dtype: Option<String>,
    /// In-situ quantization method (--isq)
    pub in_situ_quant: Option<String>,

    // Reproducibility
    /// Seed for reproducible generation (--seed)
    pub seed: Option<u64>,

    // Vision model parameters
    /// Maximum edge length for image resizing (--max-edge)
    pub max_edge: Option<usize>,
    /// Maximum number of images (--max-num-images)
    pub max_num_images: Option<usize>,
    /// Maximum image edge length (--max-image-length)
    pub max_image_length: Option<usize>,

    // Server configuration
    /// Server IP address to serve on
    pub serve_ip: Option<String>,
    /// Log file path
    pub log_file: Option<String>,

    // Search capabilities
    /// Enable search functionality
    pub enable_search: Option<bool>,
    /// BERT model for search
    pub search_bert_model: Option<String>,

    // Interactive and thinking
    /// Enable interactive mode
    pub interactive_mode: Option<bool>,
    /// Enable thinking capabilities
    pub enable_thinking: Option<bool>,

    // Token source for authentication
    pub token_source: Option<String>,
}

// Default value functions for MistralRsSettings - all fields are optional

/// LlamaCpp-specific settings for llama-server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct LlamaCppSettings {
    /// Device type (cpu, cuda, metal, etc.)
    pub device_type: Option<DeviceType>,
    /// Array of device IDs for multi-GPU
    pub device_ids: Option<Vec<i32>>,

    // Context & Memory Management (equivalent to MistralRs Sequence Management)
    /// Context size (--ctx-size, default: 4096)
    pub ctx_size: Option<i32>,
    /// Logical batch size (--batch-size, default: 2048)
    pub batch_size: Option<i32>,
    /// Physical batch size (--ubatch-size, default: 512)
    pub ubatch_size: Option<i32>,
    /// Number of parallel sequences (--parallel, default: 1)
    pub parallel: Option<i32>,
    /// Tokens to keep from initial prompt (--keep, default: 0)
    pub keep: Option<i32>,
    /// Force model to stay in RAM (--mlock, default: false)
    pub mlock: Option<bool>,
    /// Disable memory mapping (--no-mmap, default: false)
    pub no_mmap: Option<bool>,

    // Threading & Performance (equivalent to MistralRs Performance)
    /// Generation threads (--threads, default: -1)
    pub threads: Option<i32>,
    /// Batch processing threads (--threads-batch, default: same as threads)
    pub threads_batch: Option<i32>,
    /// Enable continuous batching (--cont-batching, default: true)
    pub cont_batching: Option<bool>,
    /// Enable Flash Attention (--flash-attn, default: false)
    pub flash_attn: Option<bool>,
    /// Disable KV cache offloading (--no-kv-offload, default: false)
    pub no_kv_offload: Option<bool>,

    // GPU Configuration (equivalent to MistralRs Device Config)
    /// Number of layers on GPU (--n-gpu-layers, default: 0)
    pub n_gpu_layers: Option<i32>,
    /// Primary GPU index (--main-gpu, default: 0)
    pub main_gpu: Option<i32>,
    /// How to split across GPUs: none/layer/row (--split-mode)
    pub split_mode: Option<String>,
    /// GPU memory distribution ratios (--tensor-split)
    pub tensor_split: Option<String>,

    // Model Configuration (equivalent to MistralRs Model Config)
    /// RoPE base frequency (--rope-freq-base)
    pub rope_freq_base: Option<f64>,
    /// RoPE frequency scaling (--rope-freq-scale)
    pub rope_freq_scale: Option<f64>,
    /// RoPE scaling method: none/linear/yarn (--rope-scaling)
    pub rope_scaling: Option<String>,
    /// KV cache data type for K (--cache-type-k)
    pub cache_type_k: Option<String>,
    /// KV cache data type for V (--cache-type-v)
    pub cache_type_v: Option<String>,

    // Advanced Options
    /// Random seed (--seed, default: -1)
    pub seed: Option<i64>,
    /// NUMA optimizations: distribute/isolate/numactl (--numa)
    pub numa: Option<String>,
}

impl LlamaCppSettings {
    /// Create a new LlamaCppSettings with all None values (auto-configuration)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create LlamaCppSettings optimized for high performance
    pub fn high_performance() -> Self {
        Self {
            device_type: None,
            device_ids: None,
            ctx_size: Some(8192),
            batch_size: Some(2048),
            ubatch_size: Some(512),
            parallel: Some(4),
            keep: Some(0),
            mlock: Some(true),
            no_mmap: Some(false),
            threads: Some(-1),   // Auto-detect
            threads_batch: None, // Use same as threads
            cont_batching: Some(true),
            flash_attn: Some(true),
            no_kv_offload: Some(false),
            n_gpu_layers: Some(99), // Offload all layers
            main_gpu: Some(0),
            split_mode: Some("layer".to_string()),
            tensor_split: None,
            rope_freq_base: None,
            rope_freq_scale: None,
            rope_scaling: None,
            cache_type_k: Some("f16".to_string()),
            cache_type_v: Some("f16".to_string()),
            seed: Some(-1),
            numa: Some("distribute".to_string()),
        }
    }

    /// Create LlamaCppSettings optimized for low memory usage
    pub fn low_memory() -> Self {
        Self {
            device_type: None,
            device_ids: None,
            ctx_size: Some(2048),
            batch_size: Some(512),
            ubatch_size: Some(256),
            parallel: Some(1),
            keep: Some(0),
            mlock: Some(false),
            no_mmap: Some(false),
            threads: Some(4),
            threads_batch: Some(2),
            cont_batching: Some(false),
            flash_attn: Some(false),
            no_kv_offload: Some(true),
            n_gpu_layers: Some(0), // CPU only
            main_gpu: Some(0),
            split_mode: Some("none".to_string()),
            tensor_split: None,
            rope_freq_base: None,
            rope_freq_scale: None,
            rope_scaling: None,
            cache_type_k: Some("q8_0".to_string()),
            cache_type_v: Some("q8_0".to_string()),
            seed: Some(-1),
            numa: None,
        }
    }

    /// Validate the settings and return errors if any
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ctx_size) = self.ctx_size {
            if ctx_size <= 0 {
                return Err("ctx_size must be greater than 0".to_string());
            }
            if ctx_size > 131072 {
                return Err("ctx_size should not exceed 131072 tokens".to_string());
            }
        }

        if let Some(batch_size) = self.batch_size {
            if batch_size <= 0 {
                return Err("batch_size must be greater than 0".to_string());
            }
        }

        if let Some(parallel) = self.parallel {
            if parallel <= 0 {
                return Err("parallel must be greater than 0".to_string());
            }
            if parallel > 64 {
                return Err("parallel should not exceed 64".to_string());
            }
        }

        if let Some(n_gpu_layers) = self.n_gpu_layers {
            if n_gpu_layers < 0 {
                return Err("n_gpu_layers must be non-negative".to_string());
            }
        }

        if let Some(main_gpu) = self.main_gpu {
            if main_gpu < 0 {
                return Err("main_gpu must be non-negative".to_string());
            }
        }

        if let Some(split_mode) = &self.split_mode {
            match split_mode.as_str() {
                "none" | "layer" | "row" => {}
                _ => return Err("split_mode must be 'none', 'layer', or 'row'".to_string()),
            }
        }

        if let Some(rope_scaling) = &self.rope_scaling {
            match rope_scaling.as_str() {
                "none" | "linear" | "yarn" => {}
                _ => return Err("rope_scaling must be 'none', 'linear', or 'yarn'".to_string()),
            }
        }

        if let Some(numa) = &self.numa {
            match numa.as_str() {
                "distribute" | "isolate" | "numactl" => {}
                _ => return Err("numa must be 'distribute', 'isolate', or 'numactl'".to_string()),
            }
        }

        Ok(())
    }
}

impl MistralRsSettings {
    /// Create a new MistralRsSettings with all None values (auto-load configuration)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create MistralRsSettings optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            command: Some(MistralRsCommand::Run),
            model_id_name: None,
            tokenizer_json: None,
            arch: None,
            quantized_filename: None,
            weight_file: None,
            device_type: None,
            device_ids: None,
            num_device_layers: None,
            cpu: Some(false),
            max_seqs: Some(64),
            max_seq_len: Some(8192),
            no_kv_cache: Some(false),
            truncate_sequence: Some(false),
            paged_attn_gpu_mem: Some(8192),
            paged_attn_gpu_mem_usage: Some(0.9),
            paged_ctxt_len: Some(8192),
            paged_attn_block_size: Some(64),
            no_paged_attn: Some(false),
            paged_attn: Some(true),
            chat_template: None,
            jinja_explicit: None,
            prefix_cache_n: Some(32),
            prompt_chunksize: Some(1024),
            dtype: Some("f16".to_string()),
            in_situ_quant: None,
            seed: None,
            max_edge: None,
            max_num_images: None,
            max_image_length: None,
            serve_ip: None,
            log_file: None,
            enable_search: Some(false),
            search_bert_model: None,
            interactive_mode: Some(false),
            enable_thinking: Some(false),
            token_source: None,
        }
    }

    /// Create MistralRsSettings optimized for low latency
    pub fn low_latency() -> Self {
        Self {
            command: Some(MistralRsCommand::Run),
            model_id_name: None,
            tokenizer_json: None,
            arch: None,
            quantized_filename: None,
            weight_file: None,
            device_type: Some(DeviceType::Metal),
            device_ids: None,
            num_device_layers: None,
            cpu: Some(false),
            max_seqs: Some(16),
            max_seq_len: Some(2048),
            no_kv_cache: Some(false),
            truncate_sequence: Some(true),
            paged_attn_gpu_mem: Some(2048),
            paged_attn_gpu_mem_usage: Some(0.7),
            paged_ctxt_len: Some(2048),
            paged_attn_block_size: Some(16),
            no_paged_attn: Some(false),
            paged_attn: Some(true),
            chat_template: None,
            jinja_explicit: None,
            prefix_cache_n: Some(8),
            prompt_chunksize: Some(512),
            dtype: Some("f16".to_string()),
            in_situ_quant: None,
            seed: None,
            max_edge: None,
            max_num_images: None,
            max_image_length: None,
            serve_ip: None,
            log_file: None,
            enable_search: Some(false),
            search_bert_model: None,
            interactive_mode: Some(false),
            enable_thinking: Some(false),
            token_source: None,
        }
    }

    /// Validate the settings and return errors if any
    pub fn validate(&self) -> Result<(), String> {
        // Only validate fields that are Some (configured)
        if let Some(max_seqs) = self.max_seqs {
            if max_seqs == 0 {
                return Err("max_seqs must be greater than 0".to_string());
            }
            if max_seqs > 2048 {
                return Err("max_seqs should not exceed 2048".to_string());
            }
        }

        if let Some(paged_attn_block_size) = self.paged_attn_block_size {
            if paged_attn_block_size == 0 {
                return Err("paged_attn_block_size must be greater than 0".to_string());
            }
            if paged_attn_block_size > 512 {
                return Err("paged_attn_block_size should not exceed 512".to_string());
            }
        }

        if let Some(gpu_mem) = self.paged_attn_gpu_mem {
            if gpu_mem == 0 {
                return Err("paged_attn_gpu_mem must be greater than 0".to_string());
            }
            if gpu_mem > 65536 {
                return Err("paged_attn_gpu_mem should not exceed 65536MB (64GB)".to_string());
            }
        }

        if let Some(usage) = self.paged_attn_gpu_mem_usage {
            if usage <= 0.0 || usage > 1.0 {
                return Err("paged_attn_gpu_mem_usage must be between 0 and 1".to_string());
            }
        }

        if let Some(prefix_cache_n) = self.prefix_cache_n {
            if prefix_cache_n == 0 {
                return Err("prefix_cache_n must be greater than 0".to_string());
            }
        }

        if let Some(max_seq_len) = self.max_seq_len {
            if max_seq_len > 131072 {
                return Err("max_seq_len should not exceed 131072 tokens".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Model {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_deprecated: bool,
    pub is_active: bool,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Additional fields for Candle models (None for other providers)
    pub file_size_bytes: Option<i64>,
    pub validation_status: Option<String>,
    pub validation_issues: Option<Vec<String>>,
    pub port: Option<i32>,   // Port number where the model server is running
    pub pid: Option<i32>,    // Process ID of the running model server
    pub engine_type: EngineType, // Engine type: Mistralrs | Llamacpp | None - REQUIRED
    pub engine_settings_mistralrs: Option<MistralRsSettings>, // MistralRs-specific settings
    pub engine_settings_llamacpp: Option<LlamaCppSettings>, // LlamaCpp-specific settings
    pub file_format: FileFormat, // Model file format: safetensors, gguf, pytorch, etc. - REQUIRED
    pub source: Option<SourceInfo>, // Source information for tracking download origin
    pub files: Option<Vec<ModelFileInfo>>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Model {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Parse capabilities JSON
        let capabilities_json: serde_json::Value = row.try_get("capabilities")?;
        let capabilities = if capabilities_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(capabilities_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "capabilities".into(),
                    source: Box::new(e),
                }
            })?)
        };

        // Parse parameters JSON
        let parameters_json: serde_json::Value = row.try_get("parameters")?;
        let parameters = if parameters_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(parameters_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "parameters".into(),
                    source: Box::new(e),
                }
            })?)
        };

        // Parse MistralRs engine settings
        let mistralrs_settings_json: Option<serde_json::Value> =
            row.try_get("engine_settings_mistralrs")?;
        let engine_settings_mistralrs = if let Some(json_val) = mistralrs_settings_json {
            if json_val.is_object() && json_val.as_object().unwrap().is_empty() {
                None
            } else {
                Some(serde_json::from_value(json_val).map_err(|e| {
                    sqlx::Error::ColumnDecode {
                        index: "engine_settings_mistralrs".into(),
                        source: Box::new(e),
                    }
                })?)
            }
        } else {
            None
        };

        // Parse LlamaCpp engine settings
        let llamacpp_settings_json: Option<serde_json::Value> = row.try_get("engine_settings_llamacpp")?;
        let engine_settings_llamacpp = if let Some(json_val) = llamacpp_settings_json {
            if json_val.is_object() && json_val.as_object().unwrap().is_empty() {
                None
            } else {
                Some(serde_json::from_value(json_val).map_err(|e| {
                    sqlx::Error::ColumnDecode {
                        index: "engine_settings_llamacpp".into(),
                        source: Box::new(e),
                    }
                })?)
            }
        } else {
            None
        };

        // Parse validation_issues JSON
        let validation_issues_json: Option<serde_json::Value> = row.try_get("validation_issues")?;
        let validation_issues =
            validation_issues_json.and_then(|v| serde_json::from_value::<Vec<String>>(v).ok());

        // Parse source JSON
        let source_json: Option<serde_json::Value> = row.try_get("source")?;
        let source = if let Some(source_val) = source_json {
            if source_val.is_null() {
                None
            } else {
                Some(
                    serde_json::from_value(source_val).map_err(|e| sqlx::Error::ColumnDecode {
                        index: "source".into(),
                        source: Box::new(e),
                    })?,
                )
            }
        } else {
            None
        };

        Ok(Model {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            name: row.try_get("name")?,
            alias: row.try_get("alias")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            is_deprecated: row.try_get("is_deprecated")?,
            is_active: row.try_get("is_active")?,
            capabilities,
            parameters,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            file_size_bytes: row.try_get("file_size_bytes")?,
            validation_status: row.try_get("validation_status")?,
            validation_issues,
            port: row.try_get("port")?,
            pid: row.try_get("pid")?,
            engine_type: {
                let engine_type_str: String = row.try_get("engine_type")?;
                EngineType::from_str(&engine_type_str)
                    .ok_or_else(|| sqlx::Error::ColumnDecode {
                        index: "engine_type".to_string(),
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid engine type: {}", engine_type_str),
                        )),
                    })?
            },
            engine_settings_mistralrs,
            engine_settings_llamacpp,
            file_format: {
                let file_format_str: String = row.try_get("file_format")?;
                FileFormat::from_str(&file_format_str)
                    .ok_or_else(|| sqlx::Error::ColumnDecode {
                        index: "file_format".to_string(),
                        source: format!("Invalid file format: {}", file_format_str).into(),
                    })?
            }, // Convert string to enum
            source,
            files: None, // Files need to be loaded separately
        })
    }
}

// Request/Response structures for models
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateModelRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub engine_type: EngineType, // Required field
    pub engine_settings_mistralrs: Option<MistralRsSettings>,
    pub engine_settings_llamacpp: Option<LlamaCppSettings>,
    pub file_format: FileFormat,        // Required field
    pub source: Option<SourceInfo>, // Source information for tracking download origin
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub is_active: Option<bool>,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub engine_type: Option<EngineType>,
    pub engine_settings_mistralrs: Option<MistralRsSettings>,
    pub engine_settings_llamacpp: Option<LlamaCppSettings>,
    pub file_format: Option<FileFormat>,
}

// Model file tracking for uploaded files
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FromRow)]
pub struct ModelFile {
    pub id: Uuid,
    pub model_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub upload_status: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelFileInfo {
    pub filename: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelUploadResponse {
    pub model_id: Uuid,
    pub upload_url: Option<String>,
    pub chunk_uploaded: bool,
    pub upload_complete: bool,
    pub next_chunk_index: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelListResponse {
    pub models: Vec<Model>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_storage_bytes: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelDetailsResponse {
    pub model: Model,
    pub files: Vec<ModelFileInfo>,
    pub storage_size_bytes: u64,
    pub validation_issues: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub required_files: Vec<String>,
    pub present_files: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelStorageInfo {
    pub provider_id: Uuid,
    pub total_models: i64,
    pub total_storage_bytes: u64,
    pub models_by_status: ModelStatusCounts,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ModelStatusCounts {
    pub active: i64,
    pub inactive: i64,
    pub deprecated: i64,
    pub enabled: i64,
    pub disabled: i64,
}

impl Model {
    /// Get the model path using the pattern {provider_id}/{id}
    pub fn get_model_path(&self) -> String {
        format!("models/{}/{}", self.provider_id, self.id)
    }

    pub fn get_model_absolute_path(&self) -> String {
        crate::get_app_data_dir()
            .join(self.get_model_path())
            .to_string_lossy()
            .to_string()
    }

    /// Set files from ModelFileDb structs
    pub fn with_files(mut self, files: Option<Vec<ModelFile>>) -> Self {
        self.files = files.map(|files| {
            files
                .into_iter()
                .map(|f| ModelFileInfo {
                    filename: f.filename,
                    file_size_bytes: f.file_size_bytes,
                    file_type: f.file_type,
                    uploaded_at: f.uploaded_at,
                })
                .collect()
        });
        self
    }

    /// Get the MistralRs settings, or return default settings if none are set
    pub fn get_mistralrs_settings(&self) -> MistralRsSettings {
        self.engine_settings_mistralrs.clone().unwrap_or_default()
    }

    /// Get the LlamaCpp settings, or return default settings if none are set
    pub fn get_llamacpp_settings(&self) -> LlamaCppSettings {
        self.engine_settings_llamacpp.clone().unwrap_or_default()
    }
}
