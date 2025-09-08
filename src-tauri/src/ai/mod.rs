//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.
//! It also includes local ML inference capabilities using the local framework.

pub mod api_proxy_server;
pub mod core;
pub mod engines;
pub mod file_helpers;
pub mod model_manager;
pub mod providers;
pub mod rag;
pub mod utils;

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
    build_http_client, AIProvider, ChatMessage, ChatRequest, ChatResponse, ContentPart,
    FileReference, MessageContent, ProviderFileContent, ProxyConfig, StreamingChunk,
    StreamingResponse, Usage,
};
pub use model_manager::{
    acquire_global_start_mutex,
    check_and_cleanup_model,
    // NEW EXPORTS for lifecycle management:
    reconcile_model_states,
    shutdown_all_models,
    start_model,
    // Core model starting functions:
    start_model_core_protected,
    stop_model,
    verify_model_server_running,
    ModelStartResult,
};
pub use model_manager::{register_model_access, start_auto_unload_task, AutoUnloadConfig};
pub use providers::*;
