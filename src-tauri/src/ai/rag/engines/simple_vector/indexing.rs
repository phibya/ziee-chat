// File processing, indexing methods, and embedding operations for Simple Vector RAG Engine

use super::queries;
use super::RAGSimpleVectorEngine;
use crate::ai::rag::engines::get_rag_file_storage;
use crate::ai::rag::{
    processors::{chunk::TokenBasedChunker, text},
    types::TextChunk,
    PipelineStage, ProcessingStatus, RAGErrorCode, RAGIndexingErrorCode, RAGInstanceErrorCode, RAGResult,
};
use uuid::Uuid;

impl RAGSimpleVectorEngine {
    /// Process embeddings in batches for better performance and error handling
    pub(super) async fn process_embeddings_in_batches(
        &self,
        chunks: &[TextChunk],
    ) -> RAGResult<Vec<Vec<f32>>> {
        // Get engine settings for batch size
        let engine_settings = &self.rag_instance.instance.engine_settings;
        let vector_settings = engine_settings.simple_vector.as_ref().ok_or_else(|| {
            tracing::error!(
                "SimpleVector engine settings not found for instance {}",
                self.id
            );
            RAGErrorCode::Instance(RAGInstanceErrorCode::ConfigurationError)
        })?;
        let indexing_settings = vector_settings.indexing();

        let batch_size = indexing_settings.embedding_batch_size();
        let total_chunks = chunks.len();

        tracing::info!(
            "Processing {} chunks in batches of {} using AI provider directly",
            total_chunks,
            batch_size
        );

        // Get AI model from rag_instance_info (already created)
        let ai_model = self
            .rag_instance
            .models
            .embedding_model
            .clone();

        let batches: Vec<Vec<String>> = chunks
            .chunks(batch_size)
            .map(|chunk_batch| chunk_batch.iter().map(|c| c.content.clone()).collect())
            .collect();

        // Create embedding tasks for each batch using AI model directly
        let mut batch_futures = Vec::new();
        for batch in batches {
            let ai_model_clone = ai_model.clone();

            let future = async move {
                // Create simplified embeddings request using AIModel
                let embedding_request = crate::ai::SimplifiedEmbeddingsRequest {
                    input: crate::ai::core::providers::EmbeddingsInput::Multiple(batch),
                    encoding_format: Some("float".to_string()),
                    dimensions: None,
                };

                // Call AI model embeddings API
                let response = ai_model_clone
                    .embeddings(embedding_request)
                    .await
                    .map_err(|e| {
                        tracing::error!("AI model embeddings error: {}", e);
                        RAGErrorCode::Indexing(RAGIndexingErrorCode::EmbeddingGenerationFailed)
                    })?;

                // Convert to Vec<f32> format
                let embeddings: RAGResult<Vec<Vec<f32>>> = response
                    .data
                    .into_iter()
                    .map(|embedding_data| Ok(embedding_data.embedding))
                    .collect();

                embeddings
            };

            batch_futures.push(future);
        }

        // Execute all embedding tasks in parallel using futures::future::join_all (equivalent to asyncio.gather)
        let batch_results = futures::future::join_all(batch_futures).await;

        let mut all_embeddings = Vec::with_capacity(total_chunks);
        for result in batch_results {
            let batch_embeddings = result?;
            all_embeddings.extend(batch_embeddings);
        }

        tracing::info!(
            "Completed batch embedding processing for {} chunks using AI provider",
            all_embeddings.len()
        );
        Ok(all_embeddings)
    }

    /// Store chunks with metadata and embeddings in the database
    pub(super) async fn store_chunks_with_metadata(
        &self,
        file_id: Uuid,
        chunks: Vec<TextChunk>,
        embeddings: Vec<Vec<f32>>,
    ) -> RAGResult<()> {
        if chunks.len() != embeddings.len() {
            tracing::error!(
                "Mismatch between chunks and embeddings count: {} vs {}",
                chunks.len(),
                embeddings.len()
            );
            return Err(RAGErrorCode::Indexing(
                RAGIndexingErrorCode::ProcessingError,
            ));
        }

        let chunk_count = chunks.len();
        tracing::info!("Storing {} chunks with metadata", chunk_count);

        // Process each chunk-embedding pair sequentially
        for (chunk, embedding_vector) in chunks.into_iter().zip(embeddings.into_iter()) {
            // Enhanced metadata with quality scores and processing info
            let mut enhanced_metadata = chunk.metadata.clone();
            enhanced_metadata.insert(
                "chunk_quality_score".to_string(),
                serde_json::json!(0.85), // Placeholder quality score
            );

            // Store the document and propagate any errors
            queries::upsert_vector_document(
                self.id,
                file_id,
                chunk.chunk_index as i32,
                &chunk.content,
                &chunk.content_hash,
                chunk.token_count as i32,
                &embedding_vector,
                serde_json::to_value(&enhanced_metadata).unwrap_or_default(),
            )
            .await?;
        }

        tracing::info!("Successfully stored all {} chunks", chunk_count);
        Ok(())
    }

    /// Complete file processing pipeline
    pub async fn process_file_impl(&self, file_id: Uuid) -> RAGResult<()> {
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
        let metadata = processing_result.metadata;
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

        // Save file metadata to rag_instance_files table
        let metadata_json = serde_json::to_value(metadata).unwrap_or(serde_json::Value::Null);
        queries::update_file_metadata(self.id, file_id, metadata_json).await?;

        // Step 2: Advanced Chunking with LightRAG-inspired processing
        self.update_pipeline_status(
            file_id,
            PipelineStage::Chunking,
            ProcessingStatus::InProgress,
        )
        .await?;

        let chunker = TokenBasedChunker::new();
        let raw_chunks = match chunker.chunk(&content, None, None, true, 0.7).await {
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
}