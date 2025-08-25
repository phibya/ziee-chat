// RAG (Retrieval-Augmented Generation) module
// Core types and traits for RAG functionality

pub mod engines;
pub mod models;
pub mod services;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export commonly used types
pub use engines::{RAGEngine, RAGEngineType};
pub use models::*;
pub use services::*;
pub use types::*;

/// Result type for RAG operations
pub type RAGResult<T> = Result<T, RAGError>;

/// Error types for RAG operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RAGError {
    DatabaseError(String),
    EmbeddingError(String),
    LLMError(String),
    TextExtractionError(String),
    ChunkingError(String),
    EntityExtractionError(String),
    GraphError(String),
    ConfigurationError(String),
    ProcessingError(String),
    ValidationError(String),
    NotFound(String),
    PermissionDenied(String),
}

impl std::fmt::Display for RAGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RAGError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            RAGError::EmbeddingError(msg) => write!(f, "Embedding error: {}", msg),
            RAGError::LLMError(msg) => write!(f, "LLM error: {}", msg),
            RAGError::TextExtractionError(msg) => write!(f, "Text extraction error: {}", msg),
            RAGError::ChunkingError(msg) => write!(f, "Chunking error: {}", msg),
            RAGError::EntityExtractionError(msg) => write!(f, "Entity extraction error: {}", msg),
            RAGError::GraphError(msg) => write!(f, "Graph error: {}", msg),
            RAGError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            RAGError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            RAGError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RAGError::NotFound(msg) => write!(f, "Not found: {}", msg),
            RAGError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for RAGError {}

/// Main RAG manager trait
#[async_trait]
pub trait RAGManager: Send + Sync {
    /// Initialize a RAG instance
    async fn initialize_instance(&self, instance_id: Uuid) -> RAGResult<()>;

    /// Process a file through the RAG pipeline
    async fn process_file(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        options: ProcessingOptions,
    ) -> RAGResult<ProcessingStatus>;

    /// Query the RAG instance
    async fn query(
        &self,
        instance_id: Uuid,
        query: RAGQuery,
    ) -> RAGResult<RAGQueryResponse>;

    /// Get processing status for a file
    async fn get_processing_status(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
    ) -> RAGResult<Vec<PipelineStatus>>;

    /// Delete instance data
    async fn delete_instance_data(&self, instance_id: Uuid) -> RAGResult<()>;

    /// Get instance statistics
    async fn get_instance_stats(&self, instance_id: Uuid) -> RAGResult<InstanceStats>;
}

/// Processing options for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub force_reprocess: bool,
    pub chunk_size: Option<usize>,
    pub chunk_overlap: Option<usize>,
    pub enable_entity_extraction: bool,
    pub entity_extraction_mode: EntityExtractionMode,
    pub embedding_model_override: Option<String>,
    pub llm_model_override: Option<String>,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            force_reprocess: false,
            chunk_size: Some(512),
            chunk_overlap: Some(64),
            enable_entity_extraction: true,
            entity_extraction_mode: EntityExtractionMode::Standard,
            embedding_model_override: None,
            llm_model_override: None,
        }
    }
}

/// Entity extraction modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityExtractionMode {
    /// Standard extraction with single pass
    Standard,
    /// Multi-pass extraction with gleaning (like LightRAG)
    Gleaning,
    /// Disable entity extraction
    Disabled,
}

/// Processing status for files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    InProgress { stage: String, progress: f32 },
    Completed,
    Failed(String),
}

/// Pipeline stage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub stage: PipelineStage,
    pub status: ProcessingStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub progress_percentage: u8,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineStage {
    TextExtraction,
    Chunking,
    Embedding,
    EntityExtraction,
    RelationshipExtraction,
    Indexing,
    Completed,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStage::TextExtraction => write!(f, "text_extraction"),
            PipelineStage::Chunking => write!(f, "chunking"),
            PipelineStage::Embedding => write!(f, "embedding"),
            PipelineStage::EntityExtraction => write!(f, "entity_extraction"),
            PipelineStage::RelationshipExtraction => write!(f, "relationship_extraction"),
            PipelineStage::Indexing => write!(f, "indexing"),
            PipelineStage::Completed => write!(f, "completed"),
        }
    }
}

/// RAG query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQuery {
    pub text: String,
    pub mode: QueryMode,
    pub max_results: Option<usize>,
    pub similarity_threshold: Option<f32>,
    pub context: Option<QueryContext>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// Query modes (inspired by LightRAG)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryMode {
    /// Local search within similar content
    Local,
    /// Global search using community summaries
    Global,
    /// Hybrid approach combining local and global
    Hybrid,
    /// Mix of different approaches
    Mix,
    /// Naive keyword-based search
    Naive,
    /// Bypass RAG, direct LLM query
    Bypass,
}

/// Query context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub conversation_id: Option<Uuid>,
    pub previous_queries: Vec<String>,
    pub user_preferences: HashMap<String, serde_json::Value>,
}

/// RAG query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryResponse {
    pub answer: String,
    pub sources: Vec<RAGSource>,
    pub mode_used: QueryMode,
    pub confidence_score: Option<f32>,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Source information for RAG responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSource {
    pub file_id: Uuid,
    pub filename: String,
    pub chunk_index: Option<usize>,
    pub content_snippet: String,
    pub similarity_score: f32,
    pub entity_matches: Vec<String>,
    pub relationship_matches: Vec<String>,
}

/// Instance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStats {
    pub total_files: usize,
    pub processed_files: usize,
    pub failed_files: usize,
    pub total_chunks: usize,
    pub total_entities: usize,
    pub total_relationships: usize,
    pub embedding_dimensions: Option<usize>,
    pub storage_size_bytes: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}