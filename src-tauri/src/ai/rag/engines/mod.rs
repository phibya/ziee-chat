// RAG Engine implementations

pub mod settings;
pub mod simple_graph;
pub mod simple_vector;
pub mod simple_vector_modules;
pub mod traits;

pub use simple_graph::RAGSimpleGraphEngine;
pub use simple_vector::RAGSimpleVectorEngine;
pub use traits::{RAGEngine, RAGEngineType};

use crate::ai::rag::{RAGError, RAGResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Factory for creating RAG engines
pub struct RAGEngineFactory;

impl RAGEngineFactory {
    /// Create a new RAG engine based on type
    pub fn create_engine(
        engine_type: RAGEngineType,
        database: Arc<sqlx::PgPool>,
    ) -> RAGResult<Box<dyn RAGEngine>> {
        match engine_type {
            RAGEngineType::SimpleVector => {
                let engine = RAGSimpleVectorEngine::new(database);
                Ok(Box::new(engine))
            }
            RAGEngineType::SimpleGraph => {
                let engine = RAGSimpleGraphEngine::new(database);
                Ok(Box::new(engine))
            }
        }
    }

    /// Get engine type from string
    pub fn parse_engine_type(engine_type: &str) -> RAGResult<RAGEngineType> {
        match engine_type {
            "simple_vector" => Ok(RAGEngineType::SimpleVector),
            "simple_graph" => Ok(RAGEngineType::SimpleGraph),
            _ => Err(RAGError::ConfigurationError(format!(
                "Unknown engine type: {}",
                engine_type
            ))),
        }
    }

    /// Get supported engine types
    pub fn supported_engine_types() -> Vec<RAGEngineType> {
        vec![RAGEngineType::SimpleVector, RAGEngineType::SimpleGraph]
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
