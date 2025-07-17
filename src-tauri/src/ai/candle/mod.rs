//! Local ML inference using the Candle framework
//!
//! This module provides local machine learning inference capabilities using the Candle framework,
//! including model management, device detection, scheduling, and optimizations like paged attention.

pub mod candle;
pub mod device_detection;
pub mod inference;
pub mod management;
pub mod models;
pub mod quantization;
pub mod server;

pub use candle::*;
pub use device_detection::*;
pub use inference::*;
pub use management::*;
pub use models::*;
pub use server::*;