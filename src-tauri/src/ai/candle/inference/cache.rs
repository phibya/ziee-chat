use super::super::candle::CandleError;
use candle_core::{Device, DType};
use candle_transformers::models::llama::Cache;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "metal")]
use std::sync::OnceLock;

// Global Metal synchronization lock to prevent concurrent Metal operations
#[cfg(feature = "metal")]
static METAL_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

#[cfg(feature = "metal")]
fn get_metal_lock() -> &'static Arc<Mutex<()>> {
    METAL_LOCK.get_or_init(|| Arc::new(Mutex::new(())))
}

/// Cache pool for managing reusable cache instances
pub struct CachePool {
    available_caches: Arc<Mutex<VecDeque<Cache>>>,
    config: candle_transformers::models::llama::Config,
    device: Device,
    max_size: usize,
    created_count: Arc<Mutex<usize>>,
    kv_cache_type: String,
    model_path: String,
    architecture: String,
}

impl CachePool {
    pub fn new(
        config: candle_transformers::models::llama::Config,
        device: Device,
        max_size: usize,
        kv_cache_type: String,
        model_path: String,
        architecture: String,
    ) -> Self {
        Self {
            available_caches: Arc::new(Mutex::new(VecDeque::new())),
            config,
            device,
            max_size,
            created_count: Arc::new(Mutex::new(0)),
            kv_cache_type,
            model_path,
            architecture,
        }
    }

    pub async fn acquire(&self) -> Result<Cache, CandleError> {
        loop {
            let mut available = self.available_caches.lock().await;

            if let Some(_cache) = available.pop_front() {
                // Always create a fresh cache for each request to avoid corruption
                drop(available);
                return self.create_cache_safely().await;
            } else {
                // Create new cache if under limit
                let mut created = self.created_count.lock().await;
                if *created < self.max_size {
                    *created += 1;
                    drop(created);
                    drop(available);

                    return self.create_cache_safely().await;
                } else {
                    drop(available);
                    drop(created);
                    // Wait for a cache to become available
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    // Continue loop instead of recursive call
                }
            }
        }
    }

    fn get_cache_dtype(&self) -> DType {
        match self.kv_cache_type.to_lowercase().as_str() {
            "f32" => DType::F32,
            "f16" => DType::F16,
            "bf16" => DType::BF16,
            "i8" => DType::U8, // Use U8 as closest equivalent to I8
            "auto" => {
                // Auto-detect based on model configuration
                self.auto_detect_cache_dtype()
            }
            _ => {
                println!(
                    "Warning: Unknown KV cache type '{}', defaulting to F16",
                    self.kv_cache_type
                );
                DType::F16
            }
        }
    }

    fn auto_detect_cache_dtype(&self) -> DType {
        // Try to detect model dtype from model config first
        if let Some(dtype) = self.parse_model_config_dtype() {
            println!(
                "Auto-detected KV cache dtype: {:?} (matching model precision from config.json)",
                dtype
            );
            return dtype;
        }

        // Fallback to architecture-based detection
        let model_dtype = self.detect_model_dtype();

        match model_dtype {
            Some(dtype) => {
                println!(
                    "Auto-detected KV cache dtype: {:?} (architecture-based detection)",
                    dtype
                );
                dtype
            }
            None => {
                println!("Auto-detecting KV cache dtype: Could not detect model dtype, using F16 (balanced precision)");
                DType::F16
            }
        }
    }

    fn detect_model_dtype(&self) -> Option<DType> {
        // Try to detect from model architecture and common patterns
        match self.architecture.as_str() {
            "llama" => {
                // Most Llama models use F16 by default, but some use BF16
                // We'll try to detect from the model directory structure
                self.detect_dtype_from_model_files()
            }
            _ => {
                // Default to F16 for other architectures
                Some(DType::F16)
            }
        }
    }

    fn detect_dtype_from_model_files(&self) -> Option<DType> {
        // Try to parse config.json if it exists
        if let Some(dtype) = self.parse_model_config_dtype() {
            return Some(dtype);
        }

        // Fallback to F16 as a safe default
        Some(DType::F16)
    }

    fn parse_model_config_dtype(&self) -> Option<DType> {
        // Try to read config.json from the model directory
        let config_path = std::path::Path::new(&self.model_path).join("config.json");

        if !config_path.exists() {
            println!("Model config.json not found at: {}", config_path.display());
            return None;
        }

        // Read and parse the config.json file
        match std::fs::read_to_string(&config_path) {
            Ok(config_content) => {
                match serde_json::from_str::<serde_json::Value>(&config_content) {
                    Ok(config) => {
                        // Extract torch_dtype from the config
                        if let Some(torch_dtype) =
                            config.get("torch_dtype").and_then(|v| v.as_str())
                        {
                            let detected_dtype = self.convert_torch_dtype_to_candle(torch_dtype);
                            println!(
                                "Detected model dtype from config.json: {} -> {:?}",
                                torch_dtype, detected_dtype
                            );
                            return detected_dtype;
                        } else {
                            println!("No torch_dtype found in config.json");
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse config.json: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to read config.json: {}", e);
            }
        }

        None
    }

    fn convert_torch_dtype_to_candle(&self, torch_dtype: &str) -> Option<DType> {
        match torch_dtype.to_lowercase().as_str() {
            "float32" | "torch.float32" | "f32" => Some(DType::F32),
            "float16" | "torch.float16" | "f16" => Some(DType::F16),
            "bfloat16" | "torch.bfloat16" | "bf16" => Some(DType::BF16),
            "int8" | "torch.int8" | "i8" => Some(DType::U8), // Use U8 as closest equivalent
            _ => {
                println!(
                    "Warning: Unknown torch_dtype '{}', cannot convert to candle DType",
                    torch_dtype
                );
                None
            }
        }
    }

    fn validate_dtype_compatibility(
        &self,
        model_dtype: DType,
        cache_dtype: DType,
    ) -> Result<(), String> {
        // Check if the model and cache dtypes are compatible
        match (model_dtype, cache_dtype) {
            // Perfect matches
            (DType::F32, DType::F32) |
            (DType::F16, DType::F16) |
            (DType::BF16, DType::BF16) |
            (DType::U8, DType::U8) => {
                println!("✓ Perfect dtype compatibility: Model={:?}, Cache={:?}", model_dtype, cache_dtype);
                Ok(())
            }

            // Potentially compatible combinations (with warnings)
            (DType::F32, DType::F16) |
            (DType::F16, DType::F32) => {
                Err(format!("Potential dtype mismatch: Model={:?}, Cache={:?}. This may cause inference errors.", model_dtype, cache_dtype))
            }

            (DType::BF16, DType::F16) |
            (DType::F16, DType::BF16) => {
                Err(format!("Potential dtype mismatch: Model={:?}, Cache={:?}. This may cause inference errors.", model_dtype, cache_dtype))
            }

            // Likely incompatible combinations
            _ => {
                Err(format!("Likely dtype incompatibility: Model={:?}, Cache={:?}. This will probably cause inference errors.", model_dtype, cache_dtype))
            }
        }
    }

    async fn create_cache_safely(&self) -> Result<Cache, CandleError> {
        // For Metal devices, serialize cache creation to prevent command buffer conflicts
        let cache_dtype = self.get_cache_dtype();

        // Validate dtype compatibility if we can detect the model dtype
        if let Some(model_dtype) = self.parse_model_config_dtype() {
            if let Err(warning) = self.validate_dtype_compatibility(model_dtype, cache_dtype) {
                println!("Warning: {}", warning);
            }
        }

        #[cfg(feature = "metal")]
        {
            if matches!(self.device, Device::Metal(_)) {
                let _lock = get_metal_lock().lock().await;
                return Cache::new(true, cache_dtype, &self.config, &self.device).map_err(|e| {
                    CandleError::ModelLoadError(format!("Failed to create cache: {}", e))
                });
            }
        }

        // For non-Metal devices, create cache without serialization
        Cache::new(true, cache_dtype, &self.config, &self.device)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to create cache: {}", e)))
    }

    pub async fn release(&self, cache: Cache) {
        let mut available = self.available_caches.lock().await;
        if available.len() < self.max_size {
            available.push_back(cache);
        }
        // If at capacity, just drop the cache (it will be garbage collected)
    }
}