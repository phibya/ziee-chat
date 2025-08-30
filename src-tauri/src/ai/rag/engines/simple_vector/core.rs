// Core RAGSimpleVectorEngine struct and basic methods

use super::queries;
use crate::ai::rag::{
    engines::traits::{RAGEngine, RAGEngineType},
    processors::{
        chunk::TokenBasedChunker,
        text,
    },
    PipelineStage, ProcessingStatus, RAGError, RAGQuery, RAGQueryResponse, RAGResult,
};
use crate::ai::rag::engines::get_rag_file_storage;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;


/// Simple Vector RAG Engine
pub struct RAGSimpleVectorEngine {
    // === INSTANCE ===
    pub(super) instance_id: Uuid,

    // === PROCESSING CONTROL ===
    pub(super) max_parallel_insert: usize,
    pub(super) embedding_batch_size: u32,
    pub(super) embedding_model: String,
}


impl RAGSimpleVectorEngine {
    pub fn new(instance_id: Uuid) -> Self {
        Self {
            // === INSTANCE ===
            instance_id,

            // === PROCESSING CONTROL ===
            max_parallel_insert: 10,
            embedding_batch_size: 32,
            embedding_model: "text-embedding-ada-002".to_string(),
        }
    }

    pub(super) async fn update_pipeline_status(
        &self,
        file_id: Uuid,
        stage: PipelineStage,
        status: ProcessingStatus,
        progress: u8,
        error_message: Option<String>,
    ) -> RAGResult<()> {
        queries::update_pipeline_status(
            self.instance_id,
            file_id,
            stage,
            status,
            progress,
            error_message,
        )
        .await
    }

    /// Get filename from the files table
    async fn get_filename_from_db(&self, file_id: Uuid) -> RAGResult<String> {
        queries::get_filename_from_db(file_id).await
    }
}

#[async_trait]
impl RAGEngine for RAGSimpleVectorEngine {
    fn engine_type(&self) -> RAGEngineType {
        RAGEngineType::SimpleVector
    }

    async fn process_file(&self, file_id: Uuid) -> RAGResult<()> {
        let start_time = std::time::Instant::now();

        // Get filename from database
        let filename = self.get_filename_from_db(file_id).await?;

        tracing::info!(
            "Starting file processing with RAG file storage and text extraction: {}",
            filename
        );

        // Step 1: Get file path from RAG file storage
        self.update_pipeline_status(
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::InProgress {
                stage: "getting_file_path".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        // Get the file extension from filename
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");

        let file_path = get_rag_file_storage()
            .get_file_path(self.instance_id, file_id, extension);

        if !file_path.exists() {
            return Err(RAGError::NotFound(format!(
                "File not found at path: {:?}",
                file_path
            )));
        }

        // Step 2: Extract text content using text processor
        self.update_pipeline_status(
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::InProgress {
                stage: "text_extraction".to_string(),
                progress: 50.0,
            },
            50,
            None,
        )
        .await?;

        let processing_result =
            text::extract_text_from_file(file_path.to_str().ok_or_else(|| {
                RAGError::ProcessingError("Invalid file path encoding".to_string())
            })?)
            .await?;

        let content = processing_result.content;
        let _metadata = processing_result.metadata;
        let quality_score = processing_result.quality_score;

        tracing::info!(
            "Text extraction completed for {}: {} characters, quality score: {:.2}",
            filename,
            content.len(),
            quality_score
        );

        // Create temporary AI provider for embedding processing
        let ai_provider: Arc<dyn crate::ai::core::AIProvider> = Arc::new(
            crate::ai::providers::openai::OpenAIProvider::new(
                "dummy_key".to_string(),
                None,
                None,
                uuid::Uuid::new_v4(),
            )
            .unwrap(),
        );

        self.update_pipeline_status(
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 2: Advanced Chunking with LightRAG-inspired processing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::InProgress {
                stage: "advanced_chunking".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        let chunker = TokenBasedChunker::new();
        let raw_chunks = chunker
            .chunk(&content, None, None, false, 0.7)
            .await?;

        let optimized_chunks = chunker
            .process_chunks(raw_chunks)
            .await?;

        tracing::info!(
            "Advanced processing completed: {} optimized chunks selected via chunking service",
            optimized_chunks.len()
        );

        self.update_pipeline_status(
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 3: Advanced Batch Embedding Processing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::InProgress {
                stage: "batch_embedding".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        let embeddings = self
            .process_embeddings_in_batches(&optimized_chunks, &ai_provider)
            .await?;

        self.update_pipeline_status(
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Step 4: Advanced Storage with Metadata Indexing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::InProgress {
                stage: "advanced_storage".to_string(),
                progress: 0.0,
            },
            0,
            None,
        )
        .await?;

        self.store_chunks_with_metadata(file_id, optimized_chunks, embeddings)
            .await?;

        self.update_pipeline_status(
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        // Mark as completed
        self.update_pipeline_status(
            file_id,
            PipelineStage::Completed,
            ProcessingStatus::Completed,
            100,
            None,
        )
        .await?;

        let elapsed = start_time.elapsed();
        tracing::info!(
            "Processed file {} for instance {} in {:?}",
            filename,
            self.instance_id,
            elapsed
        );

        Ok(())
    }

    async fn initialize(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // For simple vector engine, initialization is minimal
        // We might create indices or validate configuration here

        // Check if vector extension is available
        let result = queries::check_vector_extension_available().await?;

        if !result {
            return Err(RAGError::ConfigurationError(
                "PostgreSQL vector extension is not installed".to_string(),
            ));
        }

        Ok(())
    }

    async fn query(&self, _query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        // Query functionality removed - this engine is for indexing only
        Err(RAGError::ProcessingError(
            "Query functionality not implemented in indexing-only engine".to_string(),
        ))
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Validate vector extension availability
        let result = queries::check_vector_extension_available().await?;

        if !result {
            return Err(RAGError::ConfigurationError(
                "PostgreSQL vector extension is required but not installed".to_string(),
            ));
        }

        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleVector)
    }
}
