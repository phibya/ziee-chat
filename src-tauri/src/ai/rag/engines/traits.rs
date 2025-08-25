// RAG Engine traits and common types

use crate::ai::rag::{
    InstanceStats, PipelineStatus, ProcessingOptions, RAGError, RAGQuery, RAGQueryResponse, RAGResult,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RAG engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RAGEngineType {
    #[serde(rename = "rag_simple_vector")]
    SimpleVector,
    #[serde(rename = "rag_simple_graph")]
    SimpleGraph,
}

impl std::fmt::Display for RAGEngineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RAGEngineType::SimpleVector => write!(f, "rag_simple_vector"),
            RAGEngineType::SimpleGraph => write!(f, "rag_simple_graph"),
        }
    }
}

impl std::str::FromStr for RAGEngineType {
    type Err = RAGError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rag_simple_vector" => Ok(RAGEngineType::SimpleVector),
            "rag_simple_graph" => Ok(RAGEngineType::SimpleGraph),
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

    /// Initialize the engine for a specific RAG instance
    async fn initialize(&self, instance_id: Uuid, settings: serde_json::Value) -> RAGResult<()>;

    /// Process a file through the RAG pipeline
    async fn process_file(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        content: String,
        filename: String,
        options: ProcessingOptions,
    ) -> RAGResult<()>;

    /// Query the RAG engine
    async fn query(&self, instance_id: Uuid, query: RAGQuery) -> RAGResult<RAGQueryResponse>;

    /// Get processing status for files
    async fn get_processing_status(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
    ) -> RAGResult<Vec<PipelineStatus>>;

    /// Delete all data for a RAG instance
    async fn delete_instance_data(&self, instance_id: Uuid) -> RAGResult<()>;

    /// Get instance statistics
    async fn get_instance_stats(&self, instance_id: Uuid) -> RAGResult<InstanceStats>;

    /// Validate engine configuration
    async fn validate_configuration(&self, settings: serde_json::Value) -> RAGResult<()>;

    /// Get engine capabilities
    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities;

    /// Health check for the engine
    async fn health_check(&self, instance_id: Uuid) -> RAGResult<EngineHealth>;

    /// Optimize engine data (cleanup, reindex, etc.)
    async fn optimize(&self, instance_id: Uuid) -> RAGResult<OptimizationResult>;
}

/// Engine health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHealth {
    pub is_healthy: bool,
    pub status: EngineStatus,
    pub messages: Vec<String>,
    pub metrics: EngineMetrics,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Engine status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineStatus {
    Healthy,
    Warning,
    Error,
    Unavailable,
}

/// Engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub query_latency_ms: Option<f64>,
    pub indexing_throughput: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub storage_size_mb: Option<f64>,
    pub error_rate_percentage: Option<f64>,
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub success: bool,
    pub operations_performed: Vec<String>,
    pub space_freed_mb: f64,
    pub performance_improvement_percentage: Option<f64>,
    pub duration_ms: u64,
    pub next_optimization_recommended: Option<chrono::DateTime<chrono::Utc>>,
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