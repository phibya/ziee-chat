// RAG Engine traits and common types

use crate::ai::rag::{ProcessingOptions, RAGError, RAGQuery, RAGQueryResponse, RAGResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RAG engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RAGEngineType {
    #[serde(rename = "simple_vector")]
    SimpleVector,
    #[serde(rename = "simple_graph")]
    SimpleGraph,
}

impl std::fmt::Display for RAGEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RAGEngineType::SimpleVector => write!(f, "simple_vector"),
            RAGEngineType::SimpleGraph => write!(f, "simple_graph"),
        }
    }
}

impl std::str::FromStr for RAGEngineType {
    type Err = RAGError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple_vector" => Ok(RAGEngineType::SimpleVector),
            "simple_graph" => Ok(RAGEngineType::SimpleGraph),
            _ => Err(RAGError::ConfigurationError(format!(
                "Invalid engine type: {}",
                s
            ))),
        }
    }
}

/// Main RAG engine trait that all engines must implement
#[async_trait]
pub trait RAGEngine: Send + Sync {
    /// Get the engine type
    fn engine_type(&self) -> RAGEngineType;

    /// Initialize the engine
    async fn initialize(&self, settings: serde_json::Value) -> RAGResult<()>;

    /// Process a file through the RAG pipeline
    async fn process_file(&self, file_id: Uuid) -> RAGResult<()>;

    /// Query the RAG engine
    async fn query(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse>;

    /// Validate engine configuration
    async fn validate_configuration(&self, settings: serde_json::Value) -> RAGResult<()>;

    /// Get engine capabilities
    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities;
}

/// File processing context
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub instance_id: Uuid,
    pub file_id: Uuid,
    pub filename: String,
    pub content: String,
    pub options: ProcessingOptions,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl ProcessingContext {
    pub fn new(
        instance_id: Uuid,
        file_id: Uuid,
        filename: String,
        content: String,
        options: ProcessingOptions,
    ) -> Self {
        Self {
            instance_id,
            file_id,
            filename,
            content,
            options,
            start_time: chrono::Utc::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        let now = chrono::Utc::now();
        (now - self.start_time).num_milliseconds() as u64
    }
}
