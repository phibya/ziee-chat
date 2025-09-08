// Embeddings processing and batch operations

use super::{core::RAGSimpleVectorEngine, queries};
use crate::ai::rag::{
    types::{EmbeddingVector, TextChunk},
    RAGErrorCode, RAGIndexingErrorCode, RAGInstanceErrorCode, RAGResult,
};
use chrono::Utc;

impl RAGSimpleVectorEngine {
    pub(super) async fn process_embeddings_in_batches(
        &self,
        chunks: &[TextChunk],
    ) -> RAGResult<Vec<EmbeddingVector>> {
        // Get engine settings for batch size
        let engine_settings =
            crate::ai::rag::utils::get_rag_engine_settings(&self.instance_info.instance);
        let vector_settings = engine_settings.simple_vector.as_ref().ok_or_else(|| {
            tracing::error!(
                "SimpleVector engine settings not found for instance {}",
                self.instance_id
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

        // Get AI provider from rag_instance_info (already created)
        let ai_provider = self
            .instance_info
            .models
            .embedding_model
            .ai_provider
            .clone();

        let batches: Vec<Vec<String>> = chunks
            .chunks(batch_size)
            .map(|chunk_batch| chunk_batch.iter().map(|c| c.content.clone()).collect())
            .collect();

        // Create embedding tasks for each batch using AI provider directly
        let embedding_model_name = &self.instance_info.models.embedding_model.model.name;
        let model_id = self.instance_info.models.embedding_model.model.id;
        let mut batch_futures = Vec::new();
        for batch in batches {
            let ai_provider = ai_provider.clone();
            let model_name = embedding_model_name.to_string();

            let future = async move {
                // Create embeddings request using AI provider's standard format
                let embedding_request = crate::ai::core::providers::EmbeddingsRequest {
                    model_id,
                    model_name: model_name.clone(),
                    input: crate::ai::core::providers::EmbeddingsInput::Multiple(batch),
                    encoding_format: Some("float".to_string()),
                    dimensions: None,
                };

                // Call AI provider embeddings API
                let response = ai_provider
                    .embeddings(embedding_request)
                    .await
                    .map_err(|e| {
                        tracing::error!("AI provider embeddings error: {}", e);
                        RAGErrorCode::Indexing(RAGIndexingErrorCode::EmbeddingGenerationFailed)
                    })?;

                // Convert to EmbeddingVector format
                let embeddings: RAGResult<Vec<EmbeddingVector>> = response
                    .data
                    .into_iter()
                    .map(|embedding_data| {
                        let dimensions = embedding_data.embedding.len();
                        Ok(EmbeddingVector {
                            vector: embedding_data.embedding,
                            model_name: model_name.clone(),
                            dimensions,
                            created_at: chrono::Utc::now(),
                        })
                    })
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
        embeddings: Vec<EmbeddingVector>,
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
        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            // Enhanced metadata with quality scores and processing info
            let mut enhanced_metadata = chunk.metadata.clone();
            enhanced_metadata.insert(
                "processing_timestamp".to_string(),
                serde_json::json!(Utc::now().to_rfc3339()),
            );
            enhanced_metadata.insert(
                "chunk_quality_score".to_string(),
                serde_json::json!(0.85), // Placeholder quality score
            );
            enhanced_metadata.insert(
                "embedding_model".to_string(),
                serde_json::json!(embedding.model_name),
            );

            // Store the document and propagate any errors
            queries::upsert_vector_document(
                self.instance_id,
                file_id,
                chunk.chunk_index as i32,
                &chunk.content,
                &chunk.content_hash,
                chunk.token_count as i32,
                &embedding.vector,
                serde_json::to_value(&enhanced_metadata).unwrap_or_default(),
            )
            .await?;
        }

        tracing::info!("Successfully stored all {} chunks", chunk_count);
        Ok(())
    }
}
