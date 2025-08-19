use crate::ai::engines::{LlamaCppEngine, LocalEngine, MistralRsEngine};
use crate::api::errors::ApiResult2;
use axum::{debug_handler, http::StatusCode, response::Json};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct EngineInfo {
    pub engine_type: String,
    pub name: String,
    pub version: String,
    pub status: String, // "available" | "unavailable"
    pub description: Option<String>,
    pub supported_architectures: Option<Vec<String>>,
    pub required_dependencies: Option<Vec<String>>,
}

/// List all available ML inference engines
#[debug_handler]
pub async fn list_engines() -> ApiResult2<Json<Vec<EngineInfo>>> {
    let mistralrs = MistralRsEngine;
    let llamacpp = LlamaCppEngine;

    let engines = vec![
        EngineInfo {
            engine_type: "mistralrs".to_string(),
            name: mistralrs.name().to_string(),
            version: mistralrs.version(),
            status: "available".to_string(),
            description: Some(
                "High-performance Rust inference engine with quantization support".to_string(),
            ),
            supported_architectures: Some(vec![
                "llama".to_string(),
                "mistral".to_string(),
                "gemma".to_string(),
                "phi".to_string(),
            ]),
            required_dependencies: Some(vec!["mistralrs-server".to_string()]),
        },
        EngineInfo {
            engine_type: "llamacpp".to_string(),
            name: llamacpp.name().to_string(),
            version: llamacpp.version(),
            status: "unavailable".to_string(),
            description: Some("GGML-based inference engine (coming soon)".to_string()),
            supported_architectures: Some(vec![
                "llama".to_string(),
                "gpt".to_string(),
                "falcon".to_string(),
            ]),
            required_dependencies: Some(vec!["llama-server".to_string()]),
        },
    ];

    Ok((StatusCode::OK, Json(engines)))
}
