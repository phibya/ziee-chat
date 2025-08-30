// ChunkingProcessor trait definition

use crate::ai::rag::{
    types::{ChunkingConfig, TextChunk},
    RAGResult,
};
use async_trait::async_trait;
use uuid::Uuid;

use super::types::{ChunkingStrategy, ContentType};

/// Advanced chunking processor trait with LightRAG-inspired methods
#[async_trait]
pub trait ChunkingProcessor: Send + Sync {
    /// Advanced chunking with LightRAG token-based approach
    async fn advanced_chunk_text(&self, content: &str, file_id: Uuid) -> RAGResult<Vec<TextChunk>>;

    /// Split text into chunks based on configuration
    async fn chunk_text(
        &self,
        text: &str,
        file_id: Uuid,
        config: ChunkingConfig,
    ) -> RAGResult<Vec<TextChunk>>;

    /// Hybrid chunking strategy with intelligent fallback
    async fn chunk_hybrid(
        &self,
        content: &str,
        file_id: Uuid,
        primary: &ChunkingStrategy,
        fallback: &ChunkingStrategy,
        switch_threshold: usize,
    ) -> RAGResult<Vec<TextChunk>>;

    /// Adaptive chunking based on content characteristics
    async fn chunk_adaptive(
        &self,
        content: &str,
        file_id: Uuid,
        content_type: &ContentType,
        dynamic_sizing: bool,
        quality_threshold: f64,
    ) -> RAGResult<Vec<TextChunk>>;

    /// Assess chunk quality and adjust as needed
    async fn assess_and_adjust_chunk_quality(
        &self,
        chunks: Vec<TextChunk>,
        quality_threshold: f64,
    ) -> RAGResult<Vec<TextChunk>>;

    /// Get overlap content for chunking
    fn get_overlap_content(&self, content: &str, overlap_tokens: usize) -> String;

    /// Count tokens in text (LightRAG approach)
    fn estimate_tokens(&self, text: &str) -> usize;

    /// Count tokens (legacy method - calls estimate_tokens)
    fn count_tokens(&self, text: &str) -> usize;

    /// Validate chunking configuration
    fn validate_config(&self, config: &ChunkingConfig) -> RAGResult<()>;

    /// Calculate chunk quality score
    fn calculate_chunk_quality(&self, content: &str) -> f64;

    /// Recursive split for oversized chunks
    async fn recursive_split_chunk(&self, content: &str, max_size: usize)
        -> RAGResult<Vec<String>>;

    /// Create TextChunk with metadata
    async fn create_text_chunk(
        &self,
        file_id: Uuid,
        index: usize,
        content: String,
        token_count: usize,
    ) -> RAGResult<TextChunk>;

    /// Ultimate chunk selection with quality scoring
    async fn select_ultimate_chunks(&self, chunks: Vec<TextChunk>) -> RAGResult<Vec<TextChunk>>;

    /// Calculate ultimate chunk quality score
    async fn calculate_ultimate_chunk_quality(&self, content: &str) -> RAGResult<f64>;
}
