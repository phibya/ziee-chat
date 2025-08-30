// Embeddings processing and batch operations

use super::core::RAGSimpleVectorEngine;
use crate::ai::rag::{
    types::{EmbeddingVector, TextChunk},
    RAGError, RAGResult,
};
use chrono::Utc;
use futures;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

impl RAGSimpleVectorEngine {
    pub(super) async fn process_embeddings_in_batches(
        &self,
        chunks: &[TextChunk],
        ai_provider: &Arc<dyn crate::ai::core::AIProvider>,
    ) -> RAGResult<Vec<EmbeddingVector>> {
        let batch_size = self.embedding_batch_size as usize;
        let total_chunks = chunks.len();

        tracing::info!(
            "Processing {} chunks in batches of {} using AI provider directly",
            total_chunks,
            batch_size
        );

        // Split into batches like LightRAG: contents[i:i+batch_size] for i in range(0, len(contents), batch_size)
        let batches: Vec<Vec<String>> = chunks
            .chunks(batch_size)
            .map(|chunk_batch| chunk_batch.iter().map(|c| c.content.clone()).collect())
            .collect();

        // Create embedding tasks for each batch using AI provider directly
        let embedding_model = self.embedding_model.clone();
        let mut batch_futures = Vec::new();
        for batch in batches {
            let ai_provider = ai_provider.clone();
            let model_name = embedding_model.clone();

            let future = async move {
                // Create embeddings request using AI provider's standard format
                let embedding_request = crate::ai::core::providers::EmbeddingsRequest {
                    model: model_name.clone(),
                    input: crate::ai::core::providers::EmbeddingsInput::Multiple(batch),
                    encoding_format: Some("float".to_string()),
                    dimensions: None,
                };

                // Call AI provider embeddings API
                let response = ai_provider
                    .embeddings(embedding_request)
                    .await
                    .map_err(|e| {
                        RAGError::EmbeddingError(format!("AI provider embeddings error: {}", e))
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

        // Flatten the results like LightRAG: np.concatenate(embeddings_list, axis=0)
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
        instance_id: Uuid,
        chunks: Vec<TextChunk>,
        embeddings: Vec<EmbeddingVector>,
    ) -> RAGResult<()> {
        if chunks.len() != embeddings.len() {
            return Err(RAGError::ProcessingError(
                "Mismatch between chunks and embeddings count".to_string(),
            ));
        }

        tracing::info!("Storing {} chunks with advanced metadata", chunks.len());

        // Create semaphore for parallel processing control
        let semaphore = Arc::new(Semaphore::new(self.max_parallel_insert));
        let database = self.database.clone();

        let chunk_embedding_pairs: Vec<_> =
            chunks.into_iter().zip(embeddings.into_iter()).collect();

        for batch in chunk_embedding_pairs.chunks(self.embedding_batch_size as usize) {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                RAGError::ProcessingError(format!("Failed to acquire semaphore: {}", e))
            })?;

            let batch_data = batch.to_vec();
            let database = database.clone();

            tokio::spawn(async move {
                let _permit = permit;
                for (chunk, embedding) in batch_data {
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
                        serde_json::json!("text-embedding-ada-002"),
                    );

                    let _ = sqlx::query(
                        r#"
                        INSERT INTO simple_vector_documents (
                            rag_instance_id, file_id, chunk_index, content, content_hash,
                            token_count, embedding, metadata
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                        ON CONFLICT (rag_instance_id, file_id, chunk_index) DO UPDATE SET
                            content = EXCLUDED.content,
                            content_hash = EXCLUDED.content_hash,
                            token_count = EXCLUDED.token_count,
                            embedding = EXCLUDED.embedding,
                            metadata = EXCLUDED.metadata,
                            updated_at = NOW()
                        "#,
                    )
                    .bind(instance_id)
                    .bind(chunk.file_id)
                    .bind(chunk.chunk_index as i32)
                    .bind(&chunk.content)
                    .bind(&chunk.content_hash)
                    .bind(chunk.token_count as i32)
                    .bind(&embedding.vector)
                    .bind(serde_json::to_value(&enhanced_metadata).unwrap_or_default())
                    .execute(&*database)
                    .await;
                }
            });
        }

        Ok(())
    }
}
