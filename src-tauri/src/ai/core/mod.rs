//! Core AI provider types, traits, and utilities
//!
//! This module contains the fundamental types and traits used across all AI providers,
//! including common HTTP client builders and proxy configuration.
//! It also includes the new AIModel abstraction for simplified model usage.

pub mod ai_model;
pub mod device_detection;
pub mod model_instance;
pub mod provider_base;
pub mod providers;

pub use ai_model::*;
pub use model_instance::*;
pub use provider_base::*;
pub use providers::*;
