use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for Candle provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub models_directory: String,
    pub default_device: DeviceConfig,
    pub inference_settings: InferenceSettings,
}

/// Device configuration for Candle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub device_type: String, // "cpu", "cuda", "metal"
    pub device_id: Option<usize>, // For CUDA devices
}

/// Inference settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceSettings {
    pub max_tokens: u32,
    pub temperature: f64,
    pub top_p: f64,
    pub top_k: Option<u32>,
    pub repeat_penalty: f64,
    pub repeat_last_n: usize,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub architecture: String, // "llama", "mistral", "phi", etc.
    pub path: String,
    pub size_bytes: Option<u64>,
    pub parameters: Option<String>, // "7B", "13B", etc.
    pub quantization: Option<String>,
    pub enabled: bool,
    pub capabilities: ModelCapabilities,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub streaming: bool,
    pub max_context_length: Option<u32>,
}

impl Default for CandleProviderConfig {
    fn default() -> Self {
        Self {
            name: "Candle Local".to_string(),
            enabled: true,
            models_directory: "./models".to_string(),
            default_device: DeviceConfig::default(),
            inference_settings: InferenceSettings::default(),
        }
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_type: "cpu".to_string(),
            device_id: None,
        }
    }
}

impl Default for InferenceSettings {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: Some(50),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            chat: true,
            completion: true,
            streaming: true,
            max_context_length: Some(2048),
        }
    }
}

/// Model discovery and management
#[allow(dead_code)]
pub struct ModelDiscovery;

#[allow(dead_code)]
impl ModelDiscovery {
    /// Scan a directory for available models
    pub fn scan_models_directory(path: &str) -> Result<Vec<ModelConfig>, std::io::Error> {
        let mut models = Vec::new();
        let models_path = PathBuf::from(path);
        
        if !models_path.exists() {
            std::fs::create_dir_all(&models_path)?;
            return Ok(models);
        }

        for entry in std::fs::read_dir(&models_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(model) = Self::detect_model(&entry.path())? {
                    models.push(model);
                }
            }
        }

        Ok(models)
    }

    /// Detect model type and configuration from a directory
    fn detect_model(path: &PathBuf) -> Result<Option<ModelConfig>, std::io::Error> {
        let model_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Check for required files
        let config_file = path.join("config.json");
        let tokenizer_file = path.join("tokenizer.json");
        
        if !tokenizer_file.exists() {
            return Ok(None); // Not a valid model directory
        }

        // Try to determine architecture from config or directory name
        let architecture = Self::detect_architecture(&model_name, &config_file);
        
        // Calculate total size
        let size_bytes = Self::calculate_directory_size(path)?;

        Ok(Some(ModelConfig {
            id: format!("candle_{}", model_name),
            name: model_name.clone(),
            architecture,
            path: path.to_string_lossy().to_string(),
            size_bytes: Some(size_bytes),
            parameters: Self::extract_parameters(&model_name),
            quantization: Self::detect_quantization(&model_name),
            enabled: true,
            capabilities: ModelCapabilities::default(),
        }))
    }

    /// Detect model architecture from name or config
    fn detect_architecture(name: &str, config_path: &PathBuf) -> String {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("llama") {
            "llama".to_string()
        } else if name_lower.contains("mistral") {
            "mistral".to_string()
        } else if name_lower.contains("phi") {
            "phi".to_string()
        } else if name_lower.contains("gemma") {
            "gemma".to_string()
        } else {
            // Try to read from config.json if available
            if config_path.exists() {
                if let Ok(config_content) = std::fs::read_to_string(config_path) {
                    if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_content) {
                        if let Some(arch) = config_json.get("architectures")
                            .and_then(|a| a.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str()) {
                            return arch.to_lowercase();
                        }
                    }
                }
            }
            "unknown".to_string()
        }
    }

    /// Extract parameter count from model name
    fn extract_parameters(name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();
        
        for params in &["70b", "65b", "30b", "13b", "7b", "3b", "1b"] {
            if name_lower.contains(params) {
                return Some(params.to_uppercase());
            }
        }
        None
    }

    /// Detect quantization from model name
    fn detect_quantization(name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("q8_0") || name_lower.contains("q8") {
            Some("Q8_0".to_string())
        } else if name_lower.contains("q4_0") || name_lower.contains("q4") {
            Some("Q4_0".to_string())
        } else if name_lower.contains("q4_1") {
            Some("Q4_1".to_string())
        } else if name_lower.contains("gguf") {
            Some("GGUF".to_string())
        } else {
            None
        }
    }

    /// Calculate total size of directory
    fn calculate_directory_size(path: &PathBuf) -> Result<u64, std::io::Error> {
        let mut total_size = 0;
        
        fn visit_dir(dir: &PathBuf, total: &mut u64) -> Result<(), std::io::Error> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    visit_dir(&path, total)?;
                } else {
                    *total += entry.metadata()?.len();
                }
            }
            Ok(())
        }
        
        visit_dir(path, &mut total_size)?;
        Ok(total_size)
    }
}