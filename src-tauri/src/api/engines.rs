use axum::response::Json;
use serde::{Deserialize, Serialize};
use crate::api::errors::ApiResult;
use crate::ai::engines::{MistralRsEngine, LlamaCppEngine, LocalEngine};

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub engine_type: String,
    pub name: String,
    pub version: String,
    pub status: String, // "available" | "unavailable"
    pub description: String,
    pub supported_features: Vec<String>,
}

/// List all available engines
pub async fn list_engines() -> ApiResult<Json<Vec<EngineInfo>>> {
    let mistralrs = MistralRsEngine;
    let llamacpp = LlamaCppEngine;

    let engines = vec![
        EngineInfo {
            engine_type: "mistralrs".to_string(),
            name: mistralrs.name().to_string(),
            version: mistralrs.version(),
            status: "available".to_string(),
            description: "High-performance Rust inference engine with quantization support".to_string(),
            supported_features: vec![
                "quantization".to_string(),
                "paged_attention".to_string(),
                "vision_models".to_string(),
                "metal_acceleration".to_string(),
                "cuda_acceleration".to_string(),
            ],
        },
        EngineInfo {
            engine_type: "llamacpp".to_string(),
            name: llamacpp.name().to_string(),
            version: llamacpp.version(),
            status: "unavailable".to_string(),
            description: "GGML-based inference engine (coming soon)".to_string(),
            supported_features: vec![
                "ggml_quantization".to_string(),
                "cpu_optimization".to_string(),
            ],
        },
    ];
    
    Ok(Json(engines))
}