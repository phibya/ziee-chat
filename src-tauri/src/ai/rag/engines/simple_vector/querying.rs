// Query processing methods for Simple Vector RAG Engine

use super::queries::similarity_search_documents;
use super::RAGSimpleVectorEngine;
use crate::ai::rag::{
    RAGErrorCode, RAGQuery, RAGQueryResponse, RAGQueryingErrorCode, RAGResult, RAGSource,
    SimpleVectorDocument,
};
use std::collections::HashMap;

impl RAGSimpleVectorEngine {
    /// Retrieve text chunks from vector database (LightRAG _get_vector_context pattern)
    pub(super) async fn get_vector_context(
        &self,
        query_text: &str,
    ) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
        // 1. Generate query embedding
        let query_embedding = self.generate_query_embedding(query_text).await?;

        // 2. Determine search parameters from engine settings
        let (search_top_k, similarity_threshold) = {
            let settings = &self.rag_instance.instance.engine_settings.simple_vector;
            match settings.as_ref().and_then(|s| s.querying.as_ref()) {
                Some(querying_settings) => {
                    let top_k = querying_settings.top_k.unwrap_or(20) as usize;
                    let threshold = querying_settings.similarity_threshold;
                    (top_k, threshold)
                },
                None => {
                    // Use default values when no querying settings configured
                    (20, Some(0.5))
                }
            }
        };

        // 3. Execute similarity search with complete document data
        let results = similarity_search_documents(
            self.id,
            &query_embedding,
            search_top_k,
            similarity_threshold.unwrap_or(0.5),
        )
        .await?;

        tracing::info!(
            "Vector context retrieval: {} documents (search_top_k: {})",
            results.len(),
            search_top_k
        );
        Ok(results)
    }

    /// Generate embedding for query text with high priority
    pub(super) async fn generate_query_embedding(&self, query_text: &str) -> RAGResult<Vec<f32>> {
        let embedding_request = crate::ai::SimplifiedEmbeddingsRequest {
            input: crate::ai::core::providers::EmbeddingsInput::Single(query_text.to_string()),
            encoding_format: Some("float".to_string()),
            dimensions: None,
        };

        let response = self
            .rag_instance
            .models
            .embedding_model
            .embeddings(embedding_request)
            .await
            .map_err(|e| {
                tracing::error!("Query embedding generation failed: {}", e);
                RAGErrorCode::Querying(RAGQueryingErrorCode::EmbeddingGenerationFailed)
            })?;

        Ok(response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .unwrap_or_default())
    }

    /// Graceful failure handling
    pub(super) fn handle_empty_results(&self, query: &RAGQuery, processing_time: u64) -> RAGQueryResponse {
        RAGQueryResponse {
            sources: vec![],
            mode_used: query.mode.clone(),
            confidence_score: Some(0.0),
            processing_time_ms: processing_time,
            metadata: HashMap::new(),
        }
    }

    /// Vector search - retrieval only, no LLM generation
    pub(super) async fn vector_search(&self, query: &RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        // Vector retrieval only
        let raw_chunks = self.get_vector_context(&query.text).await?;

        if raw_chunks.is_empty() {
            tracing::warn!("No relevant chunks found for query: {}", query.text);
            return Ok(self.handle_empty_results(query, start_time.elapsed().as_millis() as u64));
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        let sources: Vec<RAGSource> = raw_chunks
            .into_iter()
            .map(|(document, similarity_score)| RAGSource {
                document,
                similarity_score,
            })
            .collect();

        let mut metadata = HashMap::new();
        metadata.insert(
            "chunks_retrieved".to_string(),
            serde_json::json!(sources.len()),
        );

        Ok(RAGQueryResponse {
            sources,
            mode_used: query.mode.clone(),
            confidence_score: None,
            processing_time_ms: processing_time,
            metadata,
        })
    }

    /// Complete RAG query processing - all modes now use vector search only
    pub async fn query_impl(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        tracing::info!(
            "Starting RAG query: {} (mode: {:?})",
            query.text,
            query.mode
        );

        // All query modes now use vector search only
        self.vector_search(&query).await
    }
}