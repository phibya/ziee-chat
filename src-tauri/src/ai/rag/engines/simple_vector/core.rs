// Core RAGSimpleVectorEngine struct and basic methods

use super::queries;
use crate::ai::rag::engines::get_rag_file_storage;
use crate::ai::rag::{
    engines::traits::{RAGEngine, RAGEngineType},
    processors::{chunk::TokenBasedChunker, text},
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGIndexingErrorCode, RAGInstanceErrorCode,
    RAGQuery, RAGQueryResponse, RAGQueryingErrorCode, RAGResult,
};
use async_trait::async_trait;
use uuid::Uuid;

/// Simple Vector RAG Engine
pub struct RAGSimpleVectorEngine {
    // === INSTANCE ===
    pub(super) id: Uuid,
    pub(super) rag_instance: crate::ai::rag::types::RAGInstanceInfo,
}

impl RAGSimpleVectorEngine {
    pub async fn new(instance_id: Uuid) -> RAGResult<Self> {
        let instance_info = crate::ai::rag::utils::get_rag_instance_info(instance_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get RAG instance info: {}", e);
                RAGErrorCode::Instance(RAGInstanceErrorCode::ConfigurationError)
            })?;

        Ok(Self {
            // === INSTANCE ===
            id: instance_id,
            rag_instance: instance_info,
        })
    }

    pub(super) async fn update_pipeline_status(
        &self,
        file_id: Uuid,
        stage: PipelineStage,
        status: ProcessingStatus,
    ) -> RAGResult<()> {
        queries::update_pipeline_status(self.id, file_id, stage, status)
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
            ProcessingStatus::InProgress,
        )
        .await?;

        // Get the file extension from filename
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");

        let file_path = get_rag_file_storage().get_file_path(self.id, file_id, extension);

        if !file_path.exists() {
            let error_msg = format!("File not found at path: {:?}", file_path);
            self.update_pipeline_status(
                file_id,
                PipelineStage::TextExtraction,
                ProcessingStatus::Failed(error_msg.clone()),
            )
            .await?;
            return Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::FileReadError));
        }

        // Step 2: Extract text content using text processor
        self.update_pipeline_status(
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::InProgress,
        )
        .await?;

        let file_path_str = match file_path.to_str() {
            Some(path) => path,
            None => {
                let error_msg = format!("Invalid file path encoding for file: {:?}", file_path);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::TextExtraction,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(RAGErrorCode::Indexing(RAGIndexingErrorCode::ProcessingError));
            }
        };

        let processing_result = match text::extract_text_from_file(file_path_str).await {
            Ok(result) => result,
            Err(e) => {
                let error_msg = format!("Text extraction failed for {}: {}", filename, e);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::TextExtraction,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(e);
            }
        };

        let content = processing_result.content;
        let _metadata = processing_result.metadata;
        let quality_score = processing_result.quality_score;

        tracing::info!(
            "Text extraction completed for {}: {} characters, quality score: {:.2}",
            filename,
            content.len(),
            quality_score
        );

        self.update_pipeline_status(
            file_id,
            PipelineStage::TextExtraction,
            ProcessingStatus::Completed,
        )
        .await?;

        // Step 2: Advanced Chunking with LightRAG-inspired processing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::InProgress,
        )
        .await?;

        let chunker = TokenBasedChunker::new();
        let raw_chunks = match chunker.chunk(&content, None, None, false, 0.7).await {
            Ok(chunks) => chunks,
            Err(e) => {
                let error_msg = format!("Text chunking failed for {}: {}", filename, e);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::Chunking,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(e);
            }
        };

        let optimized_chunks = match chunker.process_chunks(raw_chunks).await {
            Ok(chunks) => chunks,
            Err(e) => {
                let error_msg = format!("Chunk processing failed for {}: {}", filename, e);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::Chunking,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(e);
            }
        };

        tracing::info!(
            "Advanced processing completed: {} optimized chunks selected via chunking service",
            optimized_chunks.len()
        );

        self.update_pipeline_status(
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::Completed,
        )
        .await?;

        // Step 3: Advanced Batch Embedding Processing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::InProgress,
        )
        .await?;

        let embeddings = match self
            .process_embeddings_in_batches(&optimized_chunks)
            .await
        {
            Ok(embeddings) => embeddings,
            Err(e) => {
                let error_msg = format!("Embedding generation failed for {}: {}", filename, e);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::Embedding,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(e);
            }
        };

        self.update_pipeline_status(
            file_id,
            PipelineStage::Embedding,
            ProcessingStatus::Completed,
        )
        .await?;

        // Step 4: Advanced Storage with Metadata Indexing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::InProgress,
        )
        .await?;

        match self.store_chunks_with_metadata(file_id, optimized_chunks, embeddings)
            .await
        {
            Ok(()) => (),
            Err(e) => {
                let error_msg = format!("Index storage failed for {}: {}", filename, e);
                tracing::error!("{}", error_msg);
                self.update_pipeline_status(
                    file_id,
                    PipelineStage::Indexing,
                    ProcessingStatus::Failed(error_msg.clone()),
                )
                .await?;
                return Err(e);
            }
        };

        self.update_pipeline_status(
            file_id,
            PipelineStage::Indexing,
            ProcessingStatus::Completed,
        )
        .await?;

        // Mark as completed
        self.update_pipeline_status(
            file_id,
            PipelineStage::Completed,
            ProcessingStatus::Completed,
        )
        .await?;

        let elapsed = start_time.elapsed();
        tracing::info!(
            "Processed file {} for instance {} in {:?}",
            filename,
            self.id,
            elapsed
        );

        Ok(())
    }

    async fn initialize(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // For simple vector engine, initialization is minimal
        // We might create indices or validate configuration here

        // Vector extension is always available

        Ok(())
    }

    async fn query(&self, _query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        // Query functionality removed - this engine is for indexing only
        Err(RAGErrorCode::Querying(RAGQueryingErrorCode::InvalidQuery))
    }

    async fn validate_configuration(&self, _settings: serde_json::Value) -> RAGResult<()> {
        // Vector extension is always available

        Ok(())
    }

    fn get_capabilities(&self) -> crate::ai::rag::engines::EngineCapabilities {
        crate::ai::rag::engines::EngineCapabilities::for_engine_type(&RAGEngineType::SimpleVector)
    }
}
