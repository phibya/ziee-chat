use std::collections::HashMap;
use std::path::PathBuf;

use crate::llamacpp::backend::BackendConfig;
use crate::llamacpp::platform::PlatformConfig;

/// Complete build configuration for llama.cpp
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub source_dir: PathBuf,
    pub target_dir: PathBuf,
    pub target: String,
    pub platform: PlatformConfig,
    pub backends: Vec<BackendConfig>,
}

/// Common CMAKE flags used across configurations
pub struct CommonFlags;

impl CommonFlags {
    /// Base flags for all llama.cpp builds
    pub fn base() -> HashMap<String, String> {
        let mut flags = HashMap::new();
        flags.insert("CMAKE_BUILD_TYPE".to_string(), "Release".to_string());
        flags.insert("LLAMA_BUILD_TESTS".to_string(), "OFF".to_string());
        flags.insert("LLAMA_BUILD_EXAMPLES".to_string(), "OFF".to_string());
        flags.insert("GGML_BACKEND_DL".to_string(), "ON".to_string());
        flags.insert("GGML_CPU_ALL_VARIANTS".to_string(), "ON".to_string());
        flags.insert("LLAMA_BUILD_SERVER".to_string(), "ON".to_string());
        flags
    }

    /// Flags for CPU-only builds
    pub fn cpu_optimized() -> HashMap<String, String> {
        let mut flags = Self::base();
        flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());
        flags
    }

    /// Flags for GPU-accelerated builds
    pub fn gpu_base() -> HashMap<String, String> {
        let mut flags = Self::base();
        flags.insert("GGML_NATIVE".to_string(), "OFF".to_string());
        flags
    }

    /// Flags for debugging builds
    pub fn debug() -> HashMap<String, String> {
        let mut flags = HashMap::new();
        flags.insert("CMAKE_BUILD_TYPE".to_string(), "Debug".to_string());
        flags.insert("LLAMA_FATAL_WARNINGS".to_string(), "ON".to_string());
        flags
    }
}
