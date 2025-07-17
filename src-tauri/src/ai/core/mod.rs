//! Core AI provider types, traits, and utilities
//!
//! This module contains the fundamental types and traits used across all AI providers,
//! including common HTTP client builders and proxy configuration.

pub mod device_detection;
pub mod provider_base;
pub mod providers;

pub use provider_base::*;
pub use providers::*;
