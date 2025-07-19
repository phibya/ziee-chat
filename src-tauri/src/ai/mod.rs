//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.
//! It also includes local ML inference capabilities using the Candle framework.

pub mod core;
pub mod model_manager;
pub mod models;
pub mod providers;

// Define local types that were previously from candle_server
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

// Re-export specific items from candle_server to avoid conflicts
pub use core::device_detection;
// Re-export commonly used items for convenience
pub use core::{
  build_http_client, AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig,
  StreamingChunk, StreamingResponse, Usage,
};
pub use model_manager::{ModelStartResult, ModelStartParams, is_model_running, start_model, stop_model, check_and_cleanup_model};
pub use providers::*;
