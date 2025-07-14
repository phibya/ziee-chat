//! AI provider integrations for chat functionality
//!
//! This module provides integrations with various AI providers including OpenAI, Anthropic,
//! Groq, Gemini, Mistral, and Custom providers with support for streaming responses and proxy configurations.

pub mod anthropic;
pub mod candle;
pub mod candle_config;
pub mod candle_models;
pub mod custom;
pub mod gemini;
pub mod groq;
pub mod mistral;
pub mod openai;
pub mod openai_compatible;
pub mod provider_base;
pub mod providers;
