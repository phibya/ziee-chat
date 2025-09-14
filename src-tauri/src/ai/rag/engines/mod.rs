// RAG Engine implementations

pub mod settings;
pub mod simple_vector;
pub mod traits;

pub use simple_vector::RAGSimpleVectorEngine;
pub use traits::{RAGEngine, RAGEngineType};

use crate::ai::rag::{
    rag_file_storage::RagFileStorage, service::queries::get_engine_type_for_instance, RAGErrorCode, RAGInstanceErrorCode, RAGResult,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

/// Global file storage instance
static GLOBAL_FILE_STORAGE: OnceLock<RagFileStorage> = OnceLock::new();

/// Initialize global file storage
pub fn initialize_global_file_storage() {
    let app_data_dir = crate::get_app_data_dir();
    let file_storage = RagFileStorage::new(&app_data_dir);
    if GLOBAL_FILE_STORAGE.set(file_storage).is_err() {
        tracing::warn!("Global file storage was already initialized");
    }
}

/// Get global file storage instance
pub fn get_rag_file_storage() -> &'static RagFileStorage {
    GLOBAL_FILE_STORAGE.get_or_init(|| {
        let app_data_dir = crate::get_app_data_dir();
        RagFileStorage::new(&app_data_dir)
    })
}

/// Factory for creating RAG engines
pub struct RAGEngineFactory;

impl RAGEngineFactory {
    /// Create a new RAG engine based on instance ID (queries engine type from database)
    pub async fn create_engine(
        instance_id: Uuid,
    ) -> RAGResult<Box<dyn RAGEngine>> {
        // Query engine type from database
        let engine_type = get_engine_type_for_instance(instance_id).await?;

        match engine_type {
            RAGEngineType::SimpleVector => {
                let engine = RAGSimpleVectorEngine::new(instance_id).await?;
                Ok(Box::new(engine))
            }
            RAGEngineType::SimpleGraph => Err(RAGErrorCode::Instance(
                RAGInstanceErrorCode::ConfigurationError,
            )),
        }
    }

    /// Get supported engine types
    pub fn supported_engine_types() -> Vec<RAGEngineType> {
        vec![RAGEngineType::SimpleVector]
    }
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub engine_type: RAGEngineType,
    pub settings: serde_json::Value,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
}

/// Engine capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    pub supports_vector_similarity: bool,
    pub supports_graph_queries: bool,
    pub supports_entity_extraction: bool,
    pub supports_relationship_extraction: bool,
    pub supports_community_detection: bool,
    pub supports_hybrid_queries: bool,
    pub max_chunk_size: usize,
    pub supported_file_types: Vec<String>,
}

impl EngineCapabilities {
    pub fn for_engine_type(engine_type: &RAGEngineType) -> Self {
        match engine_type {
            RAGEngineType::SimpleVector => Self {
                supports_vector_similarity: true,
                supports_graph_queries: false,
                supports_entity_extraction: false,
                supports_relationship_extraction: false,
                supports_community_detection: false,
                supports_hybrid_queries: false,
                max_chunk_size: 8192,
                supported_file_types: vec![
                    "text/plain".to_string(),
                    "text/markdown".to_string(),
                    "application/pdf".to_string(),
                    "text/html".to_string(),
                    "application/msword".to_string(),
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                        .to_string(),
                ],
            },
            RAGEngineType::SimpleGraph => Self {
                supports_vector_similarity: false,
                supports_graph_queries: true,
                supports_entity_extraction: true,
                supports_relationship_extraction: true,
                supports_community_detection: true,
                supports_hybrid_queries: true,
                max_chunk_size: 4096,
                supported_file_types: vec![
                    "text/plain".to_string(),
                    "text/markdown".to_string(),
                    "application/pdf".to_string(),
                    "text/html".to_string(),
                    "application/msword".to_string(),
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                        .to_string(),
                ],
            },
        }
    }
}
