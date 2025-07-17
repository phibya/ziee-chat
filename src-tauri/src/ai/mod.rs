//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.
//! It also includes local ML inference capabilities using the Candle framework.

pub mod candle_server;

pub mod core;
pub mod providers;

// Re-export specific items from candle_server to avoid conflicts
pub use candle_server::{
  get_model_loader, hub_load_local_safetensors, DeviceType, ModelSelected, SpecificConfig,
};

// Re-export specific items from candle_server to avoid conflicts
pub use core::device_detection;
// Re-export commonly used items for convenience
pub use core::{
  build_http_client, AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig,
  StreamingChunk, StreamingResponse, Usage,
};
pub use providers::*;
