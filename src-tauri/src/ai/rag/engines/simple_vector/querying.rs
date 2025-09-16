// Query processing methods for Simple Vector RAG Engine

use super::queries::similarity_search_documents;
use super::utils::{
    apply_rerank_if_enabled, deduplicate_chunks_by_id,
    get_tokenizer, truncate_chunks_by_tokens,
};
use super::RAGSimpleVectorEngine;
use crate::ai::rag::{
    RAGErrorCode, RAGQuery, RAGQueryResponse, RAGQueryingErrorCode, RAGResult, RAGSource,
    QueryMode, SimpleVectorDocument,
};
use std::collections::HashMap;



impl RAGSimpleVectorEngine {
    /// Helper function to create a chat request for query refinement
    fn create_refinement_request(prompt: String) -> crate::ai::SimplifiedChatRequest {
        crate::ai::SimplifiedChatRequest {
            messages: vec![
                crate::ai::core::providers::ChatMessage {
                    role: "user".to_string(),
                    content: prompt.into(),
                },
            ],
            stream: false,
        }
    }
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

    /// Process chunks with deduplication, reranking, and token truncation (LightRAG process_chunks_unified)
    pub(super) async fn process_chunks_unified(
        &self,
        query_text: &str,
        chunks: Vec<(SimpleVectorDocument, f32)>,
        _query: &RAGQuery,
        available_chunk_tokens: usize,
    ) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
        if chunks.is_empty() {
            return Ok(vec![]);
        }

        let original_count = chunks.len();
        let mut processed_chunks = chunks;

        // 1. Deduplication by chunk_id (LightRAG pattern)
        processed_chunks = deduplicate_chunks_by_id(processed_chunks);

        // 2. Reranking (if enabled and rerank model available)
        if self.rag_instance.instance.engine_settings.simple_vector
            .as_ref()
            .map_or(false, |s| s.querying.as_ref().map_or(false, |q| q.enable_rerank()))
        {
            processed_chunks = apply_rerank_if_enabled(query_text, processed_chunks).await?;
        }

        // 3. Token-based truncation (LightRAG truncate_list_by_token_size)
        let tokenizer = get_tokenizer();
        processed_chunks =
            truncate_chunks_by_tokens(processed_chunks, available_chunk_tokens, &tokenizer).await?;

        tracing::debug!(
            "Unified chunk processing: {} chunks from {} (available_tokens: {})",
            processed_chunks.len(),
            original_count,
            available_chunk_tokens
        );

        Ok(processed_chunks)
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

    /// Handle bypass mode queries - vector retrieval only, no LLM generation
    pub(super) async fn query_bypass(&self, query: &RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        // Bypass mode: Only vector retrieval, no LLM generation
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
            mode_used: QueryMode::Bypass,
            confidence_score: None,
            processing_time_ms: processing_time,
            metadata,
        })
    }

    /// Handle full RAG pipeline queries - vector retrieval + LLM generation
    pub(super) async fn query_with_llm(&self, query: &RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        // 1. Generate refined query using LLM
        let prompt_template = self.rag_instance.instance.engine_settings.simple_vector
            .as_ref()
            .and_then(|s| s.querying.as_ref())
            .and_then(|q| q.prompt_template_pre_query.as_deref())
            .unwrap_or("{query}"); // Fallback to just the content if no template

        let history_context = String::new(); // TODO: Get conversation history from database
        let refined_query_prompt = prompt_template
            .replace("{query}", &query.text)
            .replace("{history}", &history_context);

        // Use LLM to generate refined query text
        let refined_query_text = if let Some(chat_request) = query.context.as_ref().and_then(|c| c.chat_request.as_ref()) {
            // Get the LLM model from the chat request's model_id
            let llm_model = crate::ai::model_manager::model_factory::create_ai_model(chat_request.model_id).await
                .map_err(|e| {
                    tracing::error!("Failed to create AI model for query refinement: {}", e);
                    RAGErrorCode::Querying(RAGQueryingErrorCode::LlmModelUnavailable)
                })?;

            let completion_request = Self::create_refinement_request(refined_query_prompt.clone());

            let response = llm_model.chat(completion_request).await.map_err(|e| {
                tracing::error!("Query refinement failed: {}", e);
                RAGErrorCode::Querying(RAGQueryingErrorCode::LlmGenerationFailed)
            })?;

            response.content
        } else {
            // No chat context provided - this is required for query refinement
            tracing::error!("No chat request provided for query refinement");
            return Err(RAGErrorCode::Querying(RAGQueryingErrorCode::LlmModelUnavailable));
        };

        tracing::debug!("Original query: '{}' -> Refined query: '{}'", query.text, refined_query_text);

        // 2. Vector Context Retrieval with refined query
        let raw_chunks = self.get_vector_context(&refined_query_text).await?;

        if raw_chunks.is_empty() {
            tracing::warn!("No relevant chunks found for query: {}", query.text);
            return Ok(self.handle_empty_results(query, start_time.elapsed().as_millis() as u64));
        }
        
        // 3. Unified Chunk Processing
        let processed_chunks = self
            .process_chunks_unified(
                &query.text,
                raw_chunks,
                query,
                4000, // Default available tokens for chunks
            )
            .await?;

        if processed_chunks.is_empty() {
            tracing::warn!("No chunks survived processing for query: {}", query.text);
            return Ok(self.handle_empty_results(query, start_time.elapsed().as_millis() as u64));
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        let sources: Vec<RAGSource> = processed_chunks
            .into_iter()
            .map(|(document, similarity_score)| RAGSource {
                document,
                similarity_score,
            })
            .collect();

        // Include basic metadata
        let mut metadata = HashMap::new();
        metadata.insert("chunks_used".to_string(), serde_json::json!(sources.len()));

        Ok(RAGQueryResponse {
            sources,
            mode_used: query.mode.clone(),
            confidence_score: None, // TODO: Calculate confidence
            processing_time_ms: processing_time,
            metadata,
        })
    }

    /// Complete RAG query processing
    pub async fn query_impl(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        tracing::info!(
            "Starting RAG query: {} (mode: {:?})",
            query.text,
            query.mode
        );

        match query.mode {
            QueryMode::Bypass => {
                self.query_bypass(&query).await
            }
            QueryMode::Naive | QueryMode::Local | QueryMode::Global | QueryMode::Hybrid | QueryMode::Mix => {
                self.query_with_llm(&query).await
            }
        }
    }
}