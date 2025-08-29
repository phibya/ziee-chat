use crate::ai::engines::{LlamaCppEngine, LocalEngine, MistralRsEngine};
use crate::api::errors::ApiResult;
use axum::{debug_handler, http::StatusCode, response::Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    Mistralrs,
    Llamacpp,
    None,
}

impl EngineType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mistralrs" => Some(Self::Mistralrs),
            "llamacpp" => Some(Self::Llamacpp),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mistralrs => "mistralrs",
            Self::Llamacpp => "llamacpp",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EngineInfo {
    pub engine_type: EngineType,
    pub name: String,
    pub version: String,
    pub status: String, // "available" | "unavailable"
    pub supported_architectures: Option<Vec<String>>,
}

static ENGINES: LazyLock<Vec<EngineInfo>> = LazyLock::new(|| {
    let mistralrs = MistralRsEngine;
    let llamacpp = LlamaCppEngine;

    vec![
        EngineInfo {
            engine_type: EngineType::Mistralrs,
            name: mistralrs.name().to_string(),
            version: mistralrs.version(),
            status: "available".to_string(),
            supported_architectures: Some(vec![
                "llama".to_string(),
                "mistral".to_string(),
                "gemma".to_string(),
                "phi".to_string(),
            ]),
        },
        EngineInfo {
            engine_type: EngineType::Llamacpp,
            name: llamacpp.name().to_string(),
            version: llamacpp.version(),
            status: "unavailable".to_string(),
            supported_architectures: Some(vec![
                "llama".to_string(),
                "gpt".to_string(),
                "falcon".to_string(),
            ]),
        },
    ]
});

/// List all available ML inference engines
#[debug_handler]
pub async fn list_engines() -> ApiResult<Json<Vec<EngineInfo>>> {
    Ok((StatusCode::OK, Json(ENGINES.clone())))
}
