//! Local ML inference using the Candle framework
//!
//! This module provides local machine learning inference capabilities using the Candle framework,
//! including model management, device detection, scheduling, and optimizations like paged attention.

pub mod candle;
pub mod candle_config;
pub mod candle_models;
pub mod device_detection;
pub mod model_manager;
pub mod model_server;
pub mod paged_attention;
pub mod scheduler;

pub use candle::*;
pub use candle_config::*;
pub use candle_models::*;
pub use device_detection::*;
pub use model_manager::*;
// Re-export specific items from model_server to avoid conflicts
pub use model_server::TokenizerConfig;
pub use paged_attention::*;
pub use scheduler::*;