// RAG (Retrieval-Augmented Generation) module
// Core types and traits for RAG functionality

pub mod engines;
pub mod models;
pub mod processors;
pub mod rag_file_storage;
pub mod service;
pub mod types;
pub mod utils;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Import RAGInstanceErrorCode from database models
use crate::database::models::rag_instance::RAGInstanceErrorCode;

// Re-export commonly used types
pub use engines::{RAGEngine, RAGEngineType};
pub use models::*;
pub use processors::*;
pub use service::{RAGService, RAGServiceStatus};
pub use types::*;
use crate::impl_enum_option_from;

/// Indexing error codes for file processing operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum RAGIndexingErrorCode {
    TextExtractionFailed,
    UnsupportedFileFormat,
    FileReadError,
    ChunkingFailed,
    EmbeddingGenerationFailed,
    EmbeddingModelUnavailable,
    IndexStorageFailed,
    ContentValidationFailed,
    FileTooLarge,
    ProcessingTimeout,
    ProcessingError,
    DatabaseError,
}

impl_enum_option_from!(RAGIndexingErrorCode);

impl RAGIndexingErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGIndexingErrorCode::TextExtractionFailed => "text_extraction_failed",
            RAGIndexingErrorCode::UnsupportedFileFormat => "unsupported_file_format",
            RAGIndexingErrorCode::FileReadError => "file_read_error",
            RAGIndexingErrorCode::ChunkingFailed => "chunking_failed",
            RAGIndexingErrorCode::EmbeddingGenerationFailed => "embedding_generation_failed",
            RAGIndexingErrorCode::EmbeddingModelUnavailable => "embedding_model_unavailable",
            RAGIndexingErrorCode::IndexStorageFailed => "index_storage_failed",
            RAGIndexingErrorCode::ContentValidationFailed => "content_validation_failed",
            RAGIndexingErrorCode::FileTooLarge => "file_too_large",
            RAGIndexingErrorCode::ProcessingTimeout => "processing_timeout",
            RAGIndexingErrorCode::ProcessingError => "processing_error",
            RAGIndexingErrorCode::DatabaseError => "database_error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text_extraction_failed" => Some(RAGIndexingErrorCode::TextExtractionFailed),
            "unsupported_file_format" => Some(RAGIndexingErrorCode::UnsupportedFileFormat),
            "file_read_error" => Some(RAGIndexingErrorCode::FileReadError),
            "chunking_failed" => Some(RAGIndexingErrorCode::ChunkingFailed),
            "embedding_generation_failed" => Some(RAGIndexingErrorCode::EmbeddingGenerationFailed),
            "embedding_model_unavailable" => Some(RAGIndexingErrorCode::EmbeddingModelUnavailable),
            "index_storage_failed" => Some(RAGIndexingErrorCode::IndexStorageFailed),
            "content_validation_failed" => Some(RAGIndexingErrorCode::ContentValidationFailed),
            "file_too_large" => Some(RAGIndexingErrorCode::FileTooLarge),
            "processing_timeout" => Some(RAGIndexingErrorCode::ProcessingTimeout),
            "processing_error" => Some(RAGIndexingErrorCode::ProcessingError),
            "database_error" => Some(RAGIndexingErrorCode::DatabaseError),
            _ => None,
        }
    }
}

impl std::fmt::Display for RAGIndexingErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


/// Querying error codes for chat/search operations  
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RAGQueryingErrorCode {
    InvalidQuery,
    SearchIndexUnavailable,
    EmbeddingGenerationFailed,
    SimilaritySearchFailed,
    ResultProcessingFailed,
    LlmModelUnavailable,
    LlmGenerationFailed,
    QueryTimeout,
    RateLimitExceeded,
    InsufficientContext,
}

impl_enum_option_from!(RAGQueryingErrorCode);

impl RAGQueryingErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGQueryingErrorCode::InvalidQuery => "invalid_query",
            RAGQueryingErrorCode::SearchIndexUnavailable => "search_index_unavailable",
            RAGQueryingErrorCode::EmbeddingGenerationFailed => "embedding_generation_failed",
            RAGQueryingErrorCode::SimilaritySearchFailed => "similarity_search_failed",
            RAGQueryingErrorCode::ResultProcessingFailed => "result_processing_failed",
            RAGQueryingErrorCode::LlmModelUnavailable => "llm_model_unavailable",
            RAGQueryingErrorCode::LlmGenerationFailed => "llm_generation_failed",
            RAGQueryingErrorCode::QueryTimeout => "query_timeout",
            RAGQueryingErrorCode::RateLimitExceeded => "rate_limit_exceeded",
            RAGQueryingErrorCode::InsufficientContext => "insufficient_context",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "invalid_query" => Some(RAGQueryingErrorCode::InvalidQuery),
            "search_index_unavailable" => Some(RAGQueryingErrorCode::SearchIndexUnavailable),
            "embedding_generation_failed" => Some(RAGQueryingErrorCode::EmbeddingGenerationFailed),
            "similarity_search_failed" => Some(RAGQueryingErrorCode::SimilaritySearchFailed),
            "result_processing_failed" => Some(RAGQueryingErrorCode::ResultProcessingFailed),
            "llm_model_unavailable" => Some(RAGQueryingErrorCode::LlmModelUnavailable),
            "llm_generation_failed" => Some(RAGQueryingErrorCode::LlmGenerationFailed),
            "query_timeout" => Some(RAGQueryingErrorCode::QueryTimeout),
            "rate_limit_exceeded" => Some(RAGQueryingErrorCode::RateLimitExceeded),
            "insufficient_context" => Some(RAGQueryingErrorCode::InsufficientContext),
            _ => None,
        }
    }
}

impl std::fmt::Display for RAGQueryingErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Unified error code type for all RAG operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RAGErrorCode {
    Instance(RAGInstanceErrorCode),
    Indexing(RAGIndexingErrorCode),
    Querying(RAGQueryingErrorCode),
}

impl RAGErrorCode {
    /// Helper to extract instance error for database storage
    pub fn into_instance_error(self) -> Option<RAGInstanceErrorCode> {
        match self {
            RAGErrorCode::Instance(error) => Some(error),
            _ => None,
        }
    }

    /// Helper to extract indexing error for file-level storage (future)
    pub fn into_indexing_error(self) -> Option<RAGIndexingErrorCode> {
        match self {
            RAGErrorCode::Indexing(error) => Some(error),
            _ => None,
        }
    }

    /// Helper to extract querying error for client responses
    pub fn into_querying_error(self) -> Option<RAGQueryingErrorCode> {
        match self {
            RAGErrorCode::Querying(error) => Some(error),
            _ => None,
        }
    }

    /// Get error context as string
    pub fn context(&self) -> &'static str {
        match self {
            RAGErrorCode::Instance(_) => "instance",
            RAGErrorCode::Indexing(_) => "indexing",
            RAGErrorCode::Querying(_) => "querying",
        }
    }
}

impl std::fmt::Display for RAGErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RAGErrorCode::Instance(error) => write!(f, "Instance error: {}", error),
            RAGErrorCode::Indexing(error) => write!(f, "Indexing error: {}", error),
            RAGErrorCode::Querying(error) => write!(f, "Querying error: {}", error),
        }
    }
}

impl std::error::Error for RAGErrorCode {}

/// Result type for RAG operations
pub type RAGResult<T> = Result<T, RAGErrorCode>;

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
    async fn query(&self, instance_id: Uuid, query: RAGQuery) -> RAGResult<RAGQueryResponse>;
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
    InProgress,
    Completed,
    Failed(String),
}

impl ProcessingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessingStatus::Pending => "pending",
            ProcessingStatus::InProgress => "processing",
            ProcessingStatus::Completed => "completed",
            ProcessingStatus::Failed(_) => "failed",
        }
    }
}

/// Pipeline stage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub stage: PipelineStage,
    pub status: ProcessingStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
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
    pub context: Option<QueryContext>,
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
    pub previous_queries: Vec<String>,
    pub chat_request: Option<crate::api::chat::ChatMessageRequest>,
}

/// Conversation message for history processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RAGSource {
    pub document: SimpleVectorDocument,
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
