// Core RAGSimpleVectorEngine struct and basic methods

use super::queries;
use crate::ai::rag::{
    engines::traits::{RAGEngine, RAGEngineType, RAGToolDefinition, RAGToolCall, RAGToolResult},
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGInstanceErrorCode, RAGQuery,
    RAGQueryResponse, RAGResult, QueryMode,
};
use async_trait::async_trait;
use serde_json::json;
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

    /// Get RAG instance info
    pub fn rag_instance(&self) -> &crate::ai::rag::types::RAGInstanceInfo {
        &self.rag_instance
    }
}

#[async_trait]
impl RAGEngine for RAGSimpleVectorEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleVector
    }

    async fn process_file(&self, file_id: Uuid) -> RAGResult<()> {
        self.process_file_impl(file_id).await
    }

    async fn initialize(&self, _settings: serde_json::Value) -> RAGResult<()> {
        Ok(())
    }

    async fn query(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        self.query_impl(query).await
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleVector)
    }

    fn get_tools(&self) -> Vec<RAGToolDefinition> {
        vec![RAGToolDefinition {
            name: "query".to_string(),
            description: "Search the knowledge base for relevant information using semantic similarity.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["text"]
            }),
        }]
    }

    async fn execute_tool(&self, call: RAGToolCall) -> RAGResult<RAGToolResult> {
        match call.tool_name.as_str() {
            "query" => {
                let text = call.arguments
                    .get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RAGErrorCode::Instance(
                        RAGInstanceErrorCode::ConfigurationError
                    ))?;

                let query = RAGQuery {
                    text: text.to_string(),
                    mode: QueryMode::Naive, // Default mode
                };

                let response = self.query(query).await?;

                Ok(RAGToolResult {
                    success: true,
                    result: serde_json::to_value(&response)
                        .map_err(|_| RAGErrorCode::Instance(
                            RAGInstanceErrorCode::ConfigurationError
                        ))?,
                    error_message: None,
                })
            }
            _ => Err(RAGErrorCode::Instance(
                RAGInstanceErrorCode::ConfigurationError
            )),
        }
    }
}