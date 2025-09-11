// Core RAGSimpleVectorEngine struct and basic methods

use super::queries;
use crate::ai::rag::{
    engines::traits::{RAGEngine, RAGEngineType},
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGInstanceErrorCode, RAGQuery,
    RAGQueryResponse, RAGResult,
};
use async_trait::async_trait;
use uuid::Uuid;

/// Simple Vector RAG Engine
pub struct RAGSimpleVectorEngine {
    // === INSTANCE ===
    pub(super) id: Uuid,
    pub(super) rag_instance: crate::ai::rag::types::RAGInstanceInfo,
}

impl RAGSimpleVectorEngine {
    pub async fn new(instance_id: Uuid) -> RAGResult<Self> {
        let instance_info = crate::ai::rag::utils::get_rag_instance_info(instance_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get RAG instance info: {}", e);
                RAGErrorCode::Instance(RAGInstanceErrorCode::ConfigurationError)
            })?;

        Ok(Self {
            // === INSTANCE ===
            id: instance_id,
            rag_instance: instance_info,
        })
    }

    pub(super) async fn update_pipeline_status(
        &self,
        file_id: Uuid,
        stage: PipelineStage,
        status: ProcessingStatus,
    ) -> RAGResult<()> {
        queries::update_pipeline_status(self.id, file_id, stage, status)
            .await
    }

    /// Get filename from the files table
    pub(super) async fn get_filename_from_db(&self, file_id: Uuid) -> RAGResult<String> {
        queries::get_filename_from_db(file_id).await
    }
}

#[async_trait]
impl RAGEngine for RAGSimpleVectorEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleVector
    }

    async fn process_file(&self, file_id: Uuid) -> RAGResult<()> {
        self.process_file_complete(file_id).await
    }

    async fn initialize(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // For simple vector engine, initialization is minimal
        // We might create indices or validate configuration here

        // Vector extension is always available

        Ok(())
    }

    async fn query(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        self.query_complete(query).await
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Vector extension is always available

        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleVector)
    }
}