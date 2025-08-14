use super::{LocalEngine, EngineType, EngineInstance, EngineError, ServerInfo};
use crate::database::models::model::Model;

pub struct LlamaCppEngine;

impl LlamaCppEngine {
    pub fn new() -> Self {
        LlamaCppEngine
    }
}

#[async_trait::async_trait]
impl LocalEngine for LlamaCppEngine {
    fn engine_type(&self) -> EngineType {
        EngineType::LlamaCpp
    }

    fn name(&self) -> &'static str {
        "LlamaCpp"
    }

    fn version(&self) -> String {
        "placeholder".to_string()
    }

    async fn start(&self, _model: &Model) -> Result<EngineInstance, EngineError> {
        Err(EngineError::NotImplemented("LlamaCpp support coming soon".to_string()))
    }

    async fn stop(&self, _instance: &EngineInstance) -> Result<(), EngineError> {
        Err(EngineError::NotImplemented("LlamaCpp support coming soon".to_string()))
    }

    async fn health_check(&self, _instance: &EngineInstance) -> Result<bool, EngineError> {
        Err(EngineError::NotImplemented("LlamaCpp support coming soon".to_string()))
    }

    async fn get_server_info(&self, _instance: &EngineInstance) -> Result<ServerInfo, EngineError> {
        Err(EngineError::NotImplemented("LlamaCpp support coming soon".to_string()))
    }
}