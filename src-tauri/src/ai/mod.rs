//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers like OpenAI and Anthropic
//! with support for streaming responses and proxy configurations.

pub mod openai;
pub mod anthropic;
pub mod providers;

pub use providers::{AIProvider, ChatRequest, ChatResponse, StreamingResponse};