use serde::{Deserialize, Serialize};
use std::fmt;

pub mod llamacpp;
pub mod mistralrs;

pub use llamacpp::LlamaCppEngine;
pub use mistralrs::MistralRsEngine;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EngineType {
    #[serde(rename = "mistralrs")]
    MistralRs,
    #[serde(rename = "llamacpp")]
    LlamaCpp,
}

impl fmt::Display for EngineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineType::MistralRs => write!(f, "mistralrs"),
            EngineType::LlamaCpp => write!(f, "llamacpp"),
        }
    }
}

impl std::str::FromStr for EngineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mistralrs" => Ok(EngineType::MistralRs),
            "llamacpp" => Ok(EngineType::LlamaCpp),
            _ => Err(format!("Unknown engine type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineInstance {
    pub model_uuid: String,
    pub port: u16,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug)]
pub enum EngineError {
    NotImplemented(String),
    StartupFailed(String),
    HealthCheckFailed(String),
    NetworkError(String),
    ConfigurationError(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            EngineError::StartupFailed(msg) => write!(f, "Startup failed: {}", msg),
            EngineError::HealthCheckFailed(msg) => write!(f, "Health check failed: {}", msg),
            EngineError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            EngineError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

#[async_trait::async_trait]
pub trait LocalEngine: Send + Sync {
    fn engine_type(&self) -> EngineType;
    fn name(&self) -> &'static str;
    fn version(&self) -> String;

    async fn start(
        &self,
        model: &crate::database::models::model::Model,
    ) -> Result<EngineInstance, EngineError>;
    async fn stop(&self, instance: &EngineInstance) -> Result<(), EngineError>;
    async fn health_check(&self, instance: &EngineInstance) -> Result<bool, EngineError>;
    async fn get_server_models(
        &self,
        instance: &EngineInstance,
    ) -> Result<Vec<ModelInfo>, EngineError>;
}
