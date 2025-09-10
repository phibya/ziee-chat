// Embeddings processing and batch operations

use super::{core::RAGSimpleVectorEngine, queries};
use crate::ai::rag::{
    types::TextChunk,
    RAGErrorCode, RAGIndexingErrorCode, RAGInstanceErrorCode, RAGResult,
};

impl RAGSimpleVectorEngine {
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

    pub(super) async fn store_chunks_with_metadata(
        &self,
        file_id: uuid::Uuid,
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
}
