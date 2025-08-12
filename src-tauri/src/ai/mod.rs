//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.
//! It also includes local ML inference capabilities using the Candle framework.

pub mod auto_unload;
pub mod core;
pub mod file_helpers;
pub mod model_manager;
pub mod models;
pub mod providers;

// Define local types that were previously from local_server
#[derive(Debug, Clone)]
pub enum DeviceType {
    Cpu,
    Cuda,
    Metal,
}

#[derive(Debug, Clone)]
pub enum ModelSelected {
    Llama,
    Mistral,
    Gemma,
    Qwen,
    Phi,
}

// Re-export specific items from local_server to avoid conflicts
pub use core::device_detection;
// Re-export commonly used items for convenience
pub use core::{
  AIProvider, ChatMessage, ChatRequest, ChatResponse, ContentPart, FileReference, MessageContent, ProviderFileContent, ProxyConfig,
  StreamingChunk, StreamingResponse, Usage, build_http_client,
};
pub use auto_unload::{register_model_access, start_auto_unload_task, AutoUnloadConfig};
pub use model_manager::{
  acquire_global_start_mutex, check_and_cleanup_model, is_model_running, verify_model_server_running, start_model, stop_model, ModelStartParams,
  ModelStartResult,
};
pub use providers::*;
