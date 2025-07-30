use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

/// Model capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Typed settings for individual model performance and batching configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelSettings {
    // Device configuration (kept as requested)
    /// Device type (cpu, cuda, metal, etc.)
    pub device_type: Option<String>,
    /// Array of device IDs for multi-GPU
    pub device_ids: Option<Vec<i32>>,

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
}

// Default value functions for ModelSettings - removed since all fields are now optional

impl ModelSettings {
    /// Create a new ModelSettings with all None values (auto-load configuration)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create ModelSettings optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            device_type: Some("cuda".to_string()),
            device_ids: None,
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
            prefix_cache_n: Some(32),
            prompt_chunksize: Some(1024),
            dtype: Some("f16".to_string()),
            in_situ_quant: None,
            seed: None,
            max_edge: None,
            max_num_images: None,
            max_image_length: None,
        }
    }

    /// Create ModelSettings optimized for low latency
    pub fn low_latency() -> Self {
        Self {
            device_type: Some("metal".to_string()),
            device_ids: None,
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
            prefix_cache_n: Some(8),
            prompt_chunksize: Some(512),
            dtype: Some("f16".to_string()),
            in_situ_quant: None,
            seed: None,
            max_edge: None,
            max_num_images: None,
            max_image_length: None,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub port: Option<i32>, // Port number where the model server is running
    pub pid: Option<i32>,  // Process ID of the running model server
    pub settings: Option<ModelSettings>, // Model-specific performance settings
    pub files: Option<Vec<ModelFileInfo>>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Model {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Parse capabilities JSON
        let capabilities_json: serde_json::Value = row.try_get("capabilities")?;
        let capabilities = if capabilities_json.is_null() {
            None
        } else {
            Some(
                serde_json::from_value(capabilities_json).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "capabilities".into(),
                    source: Box::new(e),
                })?,
            )
        };

        // Parse parameters JSON
        let parameters_json: serde_json::Value = row.try_get("parameters")?;
        let parameters = if parameters_json.is_null() {
            None
        } else {
            Some(
                serde_json::from_value(parameters_json).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "parameters".into(),
                    source: Box::new(e),
                })?,
            )
        };

        // Parse settings JSON
        let settings_json: serde_json::Value = row.try_get("settings")?;
        let settings = if settings_json.is_null() {
            None
        } else {
            Some(
                serde_json::from_value(settings_json).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "settings".into(),
                    source: Box::new(e),
                })?,
            )
        };

        // Parse validation_issues JSON
        let validation_issues_json: Option<serde_json::Value> = row.try_get("validation_issues")?;
        let validation_issues =
            validation_issues_json.and_then(|v| serde_json::from_value::<Vec<String>>(v).ok());

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
            settings,
            files: None, // Files need to be loaded separately
        })
    }
}

// Request/Response structures for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub settings: Option<ModelSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub is_active: Option<bool>,
    pub capabilities: Option<ModelCapabilities>,
    pub parameters: Option<ModelParameters>,
    pub settings: Option<ModelSettings>,
}

// Model file tracking for uploaded files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFileInfo {
    pub filename: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ModelUploadResponse {
    pub model_id: Uuid,
    pub upload_url: Option<String>,
    pub chunk_uploaded: bool,
    pub upload_complete: bool,
    pub next_chunk_index: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub models: Vec<Model>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_storage_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelDetailsResponse {
    pub model: Model,
    pub files: Vec<ModelFileInfo>,
    pub storage_size_bytes: u64,
    pub validation_issues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub required_files: Vec<String>,
    pub present_files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelStorageInfo {
    pub provider_id: Uuid,
    pub total_models: i64,
    pub total_storage_bytes: u64,
    pub models_by_status: ModelStatusCounts,
}

#[derive(Debug, Serialize)]
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

    /// Get the model settings, or return default settings if none are set
    pub fn get_settings(&self) -> ModelSettings {
        self.settings.clone().unwrap_or_default()
    }
}
