//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.
//! It also includes local ML inference capabilities using the Candle framework.

pub mod core;
pub mod providers;
pub mod candle;

// Re-export commonly used items for convenience
pub use core::{AIProvider, ChatMessage, ChatRequest, ChatResponse, ProxyConfig, StreamingChunk, StreamingResponse, Usage, build_http_client};
pub use providers::*;
// Re-export specific items from candle to avoid conflicts
pub use candle::{CandleProvider, CandleConfig, DeviceType, QuantizationType, ModelStatus, ModelManager};
