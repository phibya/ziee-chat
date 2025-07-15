use candle_core::{Device, Tensor};
use candle_transformers::models::llama::{Cache, Llama, Config as LlamaConfig, LlamaEosToks};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;
use std::path::Path;
use serde_json;
use serde::Deserialize;

use super::candle::{CandleError, CandleModel};

/// Check if two devices are the same by comparing their debug representation
/// This is a workaround since Device doesn't implement PartialEq
fn device_matches(device1: &Device, device2: &Device) -> bool {
    format!("{:?}", device1) == format!("{:?}", device2)
}

#[derive(Debug, Deserialize)]
struct ConfigJson {
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: Option<usize>,
    max_position_embeddings: usize,
    rms_norm_eps: f64,
    rope_theta: f32,
    bos_token_id: Option<u32>,
    eos_token_id: Option<u32>,
    tie_word_embeddings: Option<bool>,
}

impl ConfigJson {
    fn to_candle_config(&self) -> LlamaConfig {
        LlamaConfig {
            vocab_size: self.vocab_size,
            hidden_size: self.hidden_size,
            intermediate_size: self.intermediate_size,
            num_hidden_layers: self.num_hidden_layers,
            num_attention_heads: self.num_attention_heads,
            num_key_value_heads: self.num_key_value_heads.unwrap_or(self.num_attention_heads),
            max_position_embeddings: self.max_position_embeddings,
            rms_norm_eps: self.rms_norm_eps,
            rope_theta: self.rope_theta,
            bos_token_id: Some(self.bos_token_id.unwrap_or(1)),
            eos_token_id: Some(LlamaEosToks::Single(self.eos_token_id.unwrap_or(2))),
            rope_scaling: None,
            tie_word_embeddings: self.tie_word_embeddings.unwrap_or(false),
            use_flash_attn: false,
        }
    }
}

/// Real Llama model implementation using Candle
#[derive(Debug)]
pub struct LlamaModelWrapper {
    model: Llama,
    device: Device,
    cache: Cache,
    config: LlamaConfig,
}

impl LlamaModelWrapper {
    pub fn load(model_path: &str, device: &Device) -> Result<Self, CandleError> {
        println!("Loading real Llama model from: {}", model_path);
        
        // Load configuration
        let config_path = Path::new(model_path).join("config.json");
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to read config: {}", e)))?;
        let config_json: ConfigJson = serde_json::from_str(&config_str)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to parse config: {}", e)))?;
        let config = config_json.to_candle_config();
        
        println!("Model config loaded: vocab_size={}, hidden_size={}", config.vocab_size, config.hidden_size);
        
        // Load model weights - try with F16 first as it's more common for Llama models
        let weights_path = Path::new(model_path).join("model.safetensors");
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F16, device)? };
        
        // Create model
        let model = Llama::load(vb, &config)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to load model: {}", e)))?;
        
        // Initialize cache with F16 to match model
        let cache = Cache::new(true, candle_core::DType::F16, &config, device)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to create cache: {}", e)))?;
        
        println!("Model loaded successfully!");
        
        Ok(Self {
            model,
            device: device.clone(),
            cache,
            config,
        })
    }

    pub fn load_tokenizer(model_path: &str) -> Result<Tokenizer, CandleError> {
        let tokenizer_path = Path::new(model_path).join("tokenizer.json");
        println!("Loading tokenizer from: {}", tokenizer_path.display());
        Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| CandleError::TokenizerError(format!("Failed to load tokenizer: {}", e)))
    }

    pub fn load_with_files(
        model_path: &str,
        device: &Device,
        config_file: Option<&str>,
        weight_file: Option<&str>,
        _additional_weight_files: Option<&str>,
    ) -> Result<Self, CandleError> {
        println!("Loading Llama model from: {} with specific files", model_path);
        
        // Use specific config file if provided
        let config_path = if let Some(config_file) = config_file {
            Path::new(model_path).join(config_file)
        } else {
            Path::new(model_path).join("config.json")
        };
        
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to read config: {}", e)))?;
        let config_json: ConfigJson = serde_json::from_str(&config_str)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to parse config: {}", e)))?;
        let config = config_json.to_candle_config();
        
        // Use specific weight file if provided
        let weights_path = if let Some(weight_file) = weight_file {
            Path::new(model_path).join(weight_file)
        } else {
            Path::new(model_path).join("model.safetensors")
        };
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F16, device)? };
        
        let model = Llama::load(vb, &config)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to load model: {}", e)))?;
        
        let cache = Cache::new(true, candle_core::DType::F16, &config, device)
            .map_err(|e| CandleError::ModelLoadError(format!("Failed to create cache: {}", e)))?;
        
        Ok(Self {
            model,
            device: device.clone(),
            cache,
            config,
        })
    }

    pub fn load_gguf(
        model_path: &str,
        device: &Device,
        weight_file: Option<&str>,
    ) -> Result<Self, CandleError> {
        // GGUF loading would require different implementation
        // For now, fall back to regular loading
        Self::load(model_path, device)
    }

    pub fn load_tokenizer_with_file(
        model_path: &str,
        tokenizer_file: Option<&str>,
    ) -> Result<Tokenizer, CandleError> {
        let absolute_path = crate::APP_DATA_DIR.join(model_path);
        
        let tokenizer_path = if let Some(tokenizer_file) = tokenizer_file {
            absolute_path.join(tokenizer_file)
        } else {
            absolute_path.join("tokenizer.json")
        };
        
        println!("Loading tokenizer from: {}", tokenizer_path.display());
        
        Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| CandleError::TokenizerError(format!("Failed to load tokenizer: {}", e)))
    }
}

impl CandleModel for LlamaModelWrapper {
    fn forward(&mut self, input_ids: &Tensor, start_pos: usize) -> candle_core::Result<Tensor> {
        // Run the actual model forward pass
        println!("Running model forward pass with input shape: {:?}, start_pos: {}", input_ids.dims(), start_pos);
        
        // Ensure input tensor is on the same device as the model
        let input_ids = if !device_matches(input_ids.device(), &self.device) {
            println!("Moving input tensor from {:?} to {:?}", input_ids.device(), self.device);
            input_ids.to_device(&self.device)?
        } else {
            input_ids.clone()
        };
        
        let real_logits = self.model.forward(&input_ids, start_pos, &mut self.cache)?;
        
        println!("Model output logits shape: {:?}", real_logits.dims());
        
        // DEBUG: Check what the real logits look like
        if let Ok(logits_vec) = real_logits.to_vec2::<f32>() {
            if !logits_vec.is_empty() && !logits_vec[0].is_empty() {
                // Get the last token's logits
                let last_token_logits = &logits_vec[0];
                
                // Find top 10 tokens by probability
                let mut indexed_logits: Vec<(usize, f32)> = last_token_logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
                indexed_logits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                println!("Real model top 10 logits: {:?}", &indexed_logits[..10.min(indexed_logits.len())]);
                
                // Check if the model is outputting reasonable distributions
                let max_logit = indexed_logits[0].1;
                let min_logit = indexed_logits.last().map(|(_, v)| *v).unwrap_or(max_logit);
                println!("Logit range: {} to {}", min_logit, max_logit);
                
                // Check if UNK token is consistently highest
                if indexed_logits[0].0 == 0 {
                    println!("WARNING: Model real logits show UNK token (0) as highest: {}", indexed_logits[0].1);
                    
                    // Try to find what tokens should be high
                    let mut non_unk_tokens = Vec::new();
                    for (i, logit) in indexed_logits.iter().take(20) {
                        if *i != 0 && *i != 1 && *i != 2 { // Skip UNK, BOS, EOS
                            non_unk_tokens.push((*i, *logit));
                        }
                    }
                    println!("Top non-special tokens: {:?}", &non_unk_tokens[..5.min(non_unk_tokens.len())]);
                }
            }
        }
        
        // Use the actual model logits now
        Ok(real_logits)
    }

    fn clear_cache(&mut self) {
        // Reset the cache
        if let Ok(new_cache) = Cache::new(true, candle_core::DType::F16, &self.config, &self.device) {
            self.cache = new_cache;
        }
    }
}

/// Model factory for creating different model types
pub struct ModelFactory;

impl ModelFactory {
    pub fn create_model(
        model_type: &str,
        model_path: &str,
        device: &Device,
    ) -> Result<Box<dyn CandleModel + Send + Sync>, CandleError> {
        match model_type.to_lowercase().as_str() {
            "llama" => {
                let model = LlamaModelWrapper::load(model_path, device)?;
                Ok(Box::new(model))
            }
            _ => Err(CandleError::UnsupportedModel(model_type.to_string())),
        }
    }

    pub fn create_model_with_files(
        model_type: &str,
        model_path: &str,
        device: &Device,
        config_file: Option<&str>,
        weight_file: Option<&str>,
        additional_weight_files: Option<&str>,
    ) -> Result<Box<dyn CandleModel + Send + Sync>, CandleError> {
        match model_type.to_lowercase().as_str() {
            "llama" => {
                let model = LlamaModelWrapper::load_with_files(
                    model_path, 
                    device, 
                    config_file, 
                    weight_file,
                    additional_weight_files
                )?;
                Ok(Box::new(model))
            }
            "gguf" => {
                let model = LlamaModelWrapper::load_gguf(model_path, device, weight_file)?;
                Ok(Box::new(model))
            }
            _ => Err(CandleError::UnsupportedModel(model_type.to_string())),
        }
    }

    pub fn load_tokenizer(model_type: &str, model_path: &str) -> Result<Tokenizer, CandleError> {
        match model_type.to_lowercase().as_str() {
            "llama" => LlamaModelWrapper::load_tokenizer(model_path),
            _ => Err(CandleError::UnsupportedModel(model_type.to_string())),
        }
    }

    pub fn load_tokenizer_with_file(
        model_type: &str,
        model_path: &str,
        tokenizer_file: Option<&str>,
    ) -> Result<Tokenizer, CandleError> {
        match model_type.to_lowercase().as_str() {
            "llama" | "gguf" => LlamaModelWrapper::load_tokenizer_with_file(model_path, tokenizer_file),
            _ => Err(CandleError::UnsupportedModel(model_type.to_string())),
        }
    }
}

/// Model format specifications for different model types
#[derive(Debug, Clone)]
pub struct ModelFormat {
    pub name: String,
    pub required_files: Vec<String>,  // Required filenames
    pub optional_files: Vec<String>,  // Optional filenames 
    pub weight_file_patterns: Vec<String>, // Patterns for weight files (can be multiple)
}

/// Model file paths for a specific model instance
#[derive(Debug, Clone)]
pub struct ModelFilePaths {
    pub config_file: Option<String>,
    pub tokenizer_file: Option<String>,
    pub tokenizer_config_file: Option<String>,
    pub vocab_file: Option<String>,
    pub special_tokens_file: Option<String>,
    pub weight_files: Vec<String>, // Can be multiple weight files
}

/// Utility functions for model management
pub struct ModelUtils;

impl ModelUtils {
    /// Get model format specifications for different architectures
    pub fn get_model_format(architecture: &str) -> ModelFormat {
        match architecture.to_lowercase().as_str() {
            "llama" => ModelFormat {
                name: "Llama".to_string(),
                required_files: vec![
                    "tokenizer.json".to_string(),
                    "config.json".to_string(),
                ],
                optional_files: vec![
                    "tokenizer_config.json".to_string(),
                    "special_tokens_map.json".to_string(),
                    "generation_config.json".to_string(),
                ],
                weight_file_patterns: vec![
                    "model.safetensors".to_string(),
                    "pytorch_model.bin".to_string(),
                    "model-*.safetensors".to_string(),
                    "pytorch_model-*.bin".to_string(),
                ],
            },
            "mistral" => ModelFormat {
                name: "Mistral".to_string(),
                required_files: vec![
                    "tokenizer.json".to_string(),
                    "config.json".to_string(),
                ],
                optional_files: vec![
                    "tokenizer_config.json".to_string(),
                    "special_tokens_map.json".to_string(),
                    "generation_config.json".to_string(),
                ],
                weight_file_patterns: vec![
                    "model.safetensors".to_string(),
                    "pytorch_model.bin".to_string(),
                    "model-*.safetensors".to_string(),
                    "pytorch_model-*.bin".to_string(),
                ],
            },
            "gguf" => ModelFormat {
                name: "GGUF".to_string(),
                required_files: vec![],  // GGUF models are self-contained
                optional_files: vec![
                    "tokenizer.json".to_string(),
                    "config.json".to_string(),
                ],
                weight_file_patterns: vec![
                    "*.gguf".to_string(),
                ],
            },
            _ => ModelFormat {
                name: "Generic".to_string(),
                required_files: vec![
                    "tokenizer.json".to_string(),
                    "config.json".to_string(),
                ],
                optional_files: vec![
                    "tokenizer_config.json".to_string(),
                    "special_tokens_map.json".to_string(),
                ],
                weight_file_patterns: vec![
                    "model.safetensors".to_string(),
                    "pytorch_model.bin".to_string(),
                    "*.safetensors".to_string(),
                    "*.bin".to_string(),
                ],
            }
        }
    }

    /// Detect the model format based on available files in the directory
    pub fn detect_model_format(model_path: &str) -> Result<String, std::io::Error> {
        let absolute_path = crate::APP_DATA_DIR.join(model_path);
        let entries = std::fs::read_dir(&absolute_path)?;
        
        let mut has_gguf = false;
        let mut has_safetensors = false;
        let mut has_pytorch_bin = false;
        let mut has_config = false;
        let mut has_tokenizer = false;
        
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_lowercase();
            
            if filename.ends_with(".gguf") {
                has_gguf = true;
            } else if filename.ends_with(".safetensors") {
                has_safetensors = true;
            } else if filename.ends_with(".bin") && filename.contains("pytorch") {
                has_pytorch_bin = true;
            } else if filename == "config.json" {
                has_config = true;
            } else if filename == "tokenizer.json" {
                has_tokenizer = true;
            }
        }
        
        // Determine format based on file patterns
        if has_gguf {
            Ok("gguf".to_string())
        } else if has_config && has_tokenizer {
            if has_safetensors {
                Ok("llama".to_string()) // Default to llama for safetensors
            } else if has_pytorch_bin {
                Ok("llama".to_string()) // Default to llama for pytorch bins  
            } else {
                Ok("llama".to_string()) // Default format
            }
        } else {
            Ok("llama".to_string()) // Default fallback
        }
    }

    /// Get specific file paths for a model
    pub fn get_model_file_paths(model_path: &str, architecture: &str) -> Result<ModelFilePaths, std::io::Error> {
        let absolute_path = crate::APP_DATA_DIR.join(model_path);
        let format = Self::get_model_format(architecture);
        
        let mut file_paths = ModelFilePaths {
            config_file: None,
            tokenizer_file: None,
            tokenizer_config_file: None,
            vocab_file: None,
            special_tokens_file: None,
            weight_files: Vec::new(),
        };
        
        let entries = std::fs::read_dir(&absolute_path)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();
            
            match filename.as_str() {
                "config.json" => file_paths.config_file = Some(file_path.to_string_lossy().to_string()),
                "tokenizer.json" => file_paths.tokenizer_file = Some(file_path.to_string_lossy().to_string()),
                "tokenizer_config.json" => file_paths.tokenizer_config_file = Some(file_path.to_string_lossy().to_string()),
                "special_tokens_map.json" => file_paths.special_tokens_file = Some(file_path.to_string_lossy().to_string()),
                "vocab.json" | "vocab.txt" => file_paths.vocab_file = Some(file_path.to_string_lossy().to_string()),
                _ => {
                    // Check if this is a weight file
                    for pattern in &format.weight_file_patterns {
                        if Self::matches_pattern(&filename, pattern) {
                            file_paths.weight_files.push(file_path.to_string_lossy().to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(file_paths)
    }
    
    /// Simple pattern matching for filenames
    fn matches_pattern(filename: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                filename.starts_with(prefix) && filename.ends_with(suffix)
            } else {
                false
            }
        } else {
            filename == pattern
        }
    }

    /// Check if a model exists at the given path (relative to APP_DATA_DIR)
    pub fn model_exists(model_path: &str) -> bool {
        // Convert relative path to absolute path based on APP_DATA_DIR
        let absolute_path = crate::APP_DATA_DIR.join(model_path);
        println!("Checking model path: {}", absolute_path.display());
        
        if !absolute_path.exists() || !absolute_path.is_dir() {
            return false;
        }
        
        // Try to detect the model format and check for required files
        match Self::detect_model_format(model_path) {
            Ok(architecture) => {
                let format = Self::get_model_format(&architecture);
                
                // Check for required files
                for required_file in &format.required_files {
                    if !absolute_path.join(required_file).exists() {
                        println!("Missing required file: {}", required_file);
                        return false;
                    }
                }
                
                // Check for at least one weight file
                if let Ok(file_paths) = Self::get_model_file_paths(model_path, &architecture) {
                    if file_paths.weight_files.is_empty() && architecture != "gguf" {
                        println!("No weight files found");
                        return false;
                    }
                }
                
                true
            }
            Err(e) => {
                println!("Error detecting model format: {}", e);
                false
            }
        }
    }

    /// Get model size in bytes (path relative to APP_DATA_DIR)
    pub fn get_model_size(model_path: &str) -> Result<u64, std::io::Error> {
        // Convert relative path to absolute path based on APP_DATA_DIR
        let absolute_path = crate::APP_DATA_DIR.join(model_path);
        let mut total_size = 0;
        for entry in std::fs::read_dir(&absolute_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                total_size += entry.metadata()?.len();
            }
        }
        Ok(total_size)
    }

    /// List available models in a directory (path relative to APP_DATA_DIR)
    pub fn list_models(models_dir: &str) -> Result<Vec<String>, std::io::Error> {
        // Convert relative path to absolute path based on APP_DATA_DIR
        let absolute_path = crate::APP_DATA_DIR.join(models_dir);
        let mut models = Vec::new();
        for entry in std::fs::read_dir(&absolute_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Pass relative path to model_exists (models_dir/name)
                    let relative_path = format!("{}/{}", models_dir, name);
                    if Self::model_exists(&relative_path) {
                        models.push(name.to_string());
                    }
                }
            }
        }
        Ok(models)
    }
}
