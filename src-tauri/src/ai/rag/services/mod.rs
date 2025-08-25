// RAG shared services

pub mod chunking;
pub mod embedding;
pub mod entity_extraction;
pub mod llm;
pub mod text_extraction;

pub use chunking::{ChunkingService, TokenBasedChunker};
pub use embedding::{EmbeddingService, EmbeddingServiceImpl};
pub use entity_extraction::{EntityExtractionService, EntityExtractionServiceImpl};
pub use llm::{LLMService, LLMServiceImpl};
pub use text_extraction::{TextExtractionService, TextExtractionServiceImpl};

use crate::ai::rag::RAGResult;
use async_trait::async_trait;
use std::sync::Arc;

/// Service manager for coordinating all RAG services
pub struct RAGServiceManager {
    pub text_extraction: Arc<dyn TextExtractionService>,
    pub chunking: Arc<dyn ChunkingService>,
    pub embedding: Arc<dyn EmbeddingService>,
    pub llm: Arc<dyn LLMService>,
    pub entity_extraction: Arc<dyn EntityExtractionService>,
}

impl RAGServiceManager {
    pub fn new(
        _database: Arc<sqlx::PgPool>,
        ai_provider_service: Arc<dyn crate::ai::core::AIProvider>,
    ) -> Self {
        let text_extraction = Arc::new(TextExtractionServiceImpl::new());
        let chunking = Arc::new(TokenBasedChunker::new());
        let embedding = Arc::new(EmbeddingServiceImpl::new(ai_provider_service.clone()));
        let llm = Arc::new(LLMServiceImpl::new(ai_provider_service));
        let entity_extraction = Arc::new(EntityExtractionServiceImpl::new(llm.clone()));

        Self {
            text_extraction,
            chunking,
            embedding,
            llm,
            entity_extraction,
        }
    }

    /// Health check for all services
    pub async fn health_check(&self) -> RAGResult<ServiceHealthReport> {
        let mut report = ServiceHealthReport::default();

        // Check text extraction service
        match self.text_extraction.health_check().await {
            Ok(health) => report.text_extraction = health,
            Err(e) => {
                report.text_extraction.is_healthy = false;
                report.text_extraction.error_message = Some(e.to_string());
            }
        }

        // Check chunking service
        match self.chunking.health_check().await {
            Ok(health) => report.chunking = health,
            Err(e) => {
                report.chunking.is_healthy = false;
                report.chunking.error_message = Some(e.to_string());
            }
        }

        // Check embedding service
        match self.embedding.health_check().await {
            Ok(health) => report.embedding = health,
            Err(e) => {
                report.embedding.is_healthy = false;
                report.embedding.error_message = Some(e.to_string());
            }
        }

        // Check LLM service
        match self.llm.health_check().await {
            Ok(health) => report.llm = health,
            Err(e) => {
                report.llm.is_healthy = false;
                report.llm.error_message = Some(e.to_string());
            }
        }

        // Check entity extraction service
        match self.entity_extraction.health_check().await {
            Ok(health) => report.entity_extraction = health,
            Err(e) => {
                report.entity_extraction.is_healthy = false;
                report.entity_extraction.error_message = Some(e.to_string());
            }
        }

        Ok(report)
    }
}

/// Health report for all services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceHealthReport {
    pub text_extraction: ServiceHealth,
    pub chunking: ServiceHealth,
    pub embedding: ServiceHealth,
    pub llm: ServiceHealth,
    pub entity_extraction: ServiceHealth,
}

impl Default for ServiceHealthReport {
    fn default() -> Self {
        Self {
            text_extraction: ServiceHealth::default(),
            chunking: ServiceHealth::default(),
            embedding: ServiceHealth::default(),
            llm: ServiceHealth::default(),
            entity_extraction: ServiceHealth::default(),
        }
    }
}

/// Health status for individual services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub status: ServiceStatus,
    pub error_message: Option<String>,
    pub response_time_ms: Option<u64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            status: ServiceStatus::Unknown,
            error_message: None,
            response_time_ms: None,
            last_check: chrono::Utc::now(),
        }
    }
}

/// Service status enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Warning,
    Error,
    Unavailable,
    Unknown,
}

/// Service performance metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceMetrics {
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub active_connections: u64,
    pub memory_usage_mb: f64,
}

/// Base trait for all service health checks
#[async_trait]
pub trait ServiceHealthCheck {
    async fn health_check(&self) -> RAGResult<ServiceHealth>;
}