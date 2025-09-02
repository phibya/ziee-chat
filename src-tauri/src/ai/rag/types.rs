// RAG-specific type definitions and utilities

use crate::ai::core::AIProvider;
use crate::database::models::{rag_instance::RAGInstance, Model, Provider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// A model with its corresponding AI provider
pub struct RAGModel {
    pub model: Model,
    pub ai_provider: Arc<dyn AIProvider>,
}

/// Collection of models for a RAG instance
pub struct RAGModels {
    pub embedding_model: RAGModel,
    pub llm_model: Option<RAGModel>,
}

/// Complete RAG instance information including provider and models
pub struct RAGInstanceInfo {
    pub instance: RAGInstance,
    pub provider: Provider,
    pub models: RAGModels,
}

/// Text chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub id: Option<Uuid>,
    pub content: String,
    pub content_hash: String,
    pub token_count: usize,
    pub chunk_index: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Embedding vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub vector: Vec<f32>,
    pub model_name: String,
    pub dimensions: usize,
    pub created_at: DateTime<Utc>,
}

/// Entity extracted from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Option<Uuid>,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub importance_score: f32,
    pub extraction_metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Option<Uuid>,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub relationship_type: String,
    pub description: Option<String>,
    pub weight: f32,
    pub extraction_metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Community detected in graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: Option<Uuid>,
    pub community_id: i32,
    pub entity_ids: Vec<Uuid>,
    pub summary: Option<String>,
    pub importance_score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Token counting and chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    pub max_chunk_size: usize,
    pub chunk_overlap: usize,
    pub min_chunk_size: usize,
    pub preserve_sentence_boundaries: bool,
    pub preserve_paragraph_boundaries: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 512,
            chunk_overlap: 64,
            min_chunk_size: 100,
            preserve_sentence_boundaries: true,
            preserve_paragraph_boundaries: true,
        }
    }
}

/// Embedding generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub dimensions: usize,
    pub batch_size: usize,
    pub max_retries: usize,
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "text-embedding-ada-002".to_string(),
            dimensions: 1536,
            batch_size: 100,
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

/// Entity extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractionConfig {
    pub max_entities_per_chunk: usize,
    pub min_entity_length: usize,
    pub max_entity_length: usize,
    pub entity_types: Vec<String>,
    pub gleaning_iterations: usize,
    pub confidence_threshold: f32,
    pub use_cot_reasoning: bool,
}

impl Default for EntityExtractionConfig {
    fn default() -> Self {
        Self {
            max_entities_per_chunk: 20,
            min_entity_length: 2,
            max_entity_length: 100,
            entity_types: vec![
                "PERSON".to_string(),
                "ORGANIZATION".to_string(),
                "LOCATION".to_string(),
                "EVENT".to_string(),
                "CONCEPT".to_string(),
                "TECHNOLOGY".to_string(),
                "PRODUCT".to_string(),
            ],
            gleaning_iterations: 2,
            confidence_threshold: 0.7,
            use_cot_reasoning: true,
        }
    }
}

/// LLM service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model_name: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub system_prompt: Option<String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model_name: "gpt-3.5-turbo".to_string(),
            max_tokens: 2048,
            temperature: 0.1,
            timeout_seconds: 60,
            max_retries: 3,
            system_prompt: None,
        }
    }
}

/// Text extraction results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedText {
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub page_count: Option<usize>,
    pub word_count: usize,
    pub extraction_method: String,
    pub quality_score: f32,
}

/// Similarity search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub chunk_id: Uuid,
    pub similarity_score: f32,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Graph query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub paths: Vec<GraphPath>,
    pub communities: Vec<Community>,
}

/// Graph path between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub total_weight: f32,
    pub path_length: usize,
}

/// Query execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionContext {
    pub start_time: DateTime<Utc>,
    pub timeout_at: DateTime<Utc>,
    pub max_results: usize,
    pub similarity_threshold: f32,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for QueryExecutionContext {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start_time: now,
            timeout_at: now + chrono::Duration::seconds(30),
            max_results: 100,
            similarity_threshold: 0.7,
            enable_caching: true,
            cache_ttl_seconds: 3600, // 1 hour
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    pub batch_size: usize,
    pub max_concurrent_batches: usize,
    pub retry_failed_items: bool,
    pub max_retries_per_item: usize,
    pub progress_callback_interval: usize,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            max_concurrent_batches: 4,
            retry_failed_items: true,
            max_retries_per_item: 3,
            progress_callback_interval: 10,
        }
    }
}

/// Cache entry for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub hit_count: usize,
    pub last_accessed: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        let now = Utc::now();
        Self {
            data,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_seconds as i64),
            hit_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn access(&mut self) -> &T {
        self.hit_count += 1;
        self.last_accessed = Utc::now();
        &self.data
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration_ms: u64,
    pub items_processed: usize,
    pub bytes_processed: u64,
    pub success_count: usize,
    pub error_count: usize,
    pub throughput_items_per_second: f32,
    pub memory_usage_mb: f32,
    pub timestamp: DateTime<Utc>,
}

/// Validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_error(error: String) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
}
