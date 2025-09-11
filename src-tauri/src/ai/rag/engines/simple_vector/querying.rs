// Query processing methods for Simple Vector RAG Engine

use super::queries::similarity_search_documents;
use super::utils::{
    apply_rerank_if_enabled, deduplicate_chunks_by_id, format_chunks_as_context,
    get_max_total_tokens, get_tokenizer, post_process_llm_response,
    truncate_chunks_by_tokens, TokenBudget,
};
use super::RAGSimpleVectorEngine;
use crate::ai::rag::{
    RAGErrorCode, RAGQuery, RAGQueryResponse, RAGQueryingErrorCode, RAGResult, RAGSource,
    QueryMode, SimpleVectorDocument,
};
use std::collections::HashMap;

// LightRAG response template for context formatting
const NAIVE_RAG_RESPONSE_TEMPLATE: &str = r#"
---Role---
You are a helpful assistant responding to questions based on the provided context information.

---Goal---
Generate an informative answer to the user's question based on the provided documents. Use the retrieved content to provide accurate and comprehensive information.

---Context---
{content_data}

---Response Type---
{response_type}

---Conversation History---
{history}

---Instructions---
- Use the context information to provide a detailed and accurate response
- If the context doesn't contain sufficient information, acknowledge this limitation
- Maintain a helpful and informative tone
- Structure your response clearly

---User Prompt---
{user_prompt}
"#;

const FAIL_RESPONSE: &str = "Sorry, I couldn't find relevant information to answer your question.";

impl RAGSimpleVectorEngine {
    /// Retrieve text chunks from vector database (LightRAG _get_vector_context pattern)
    pub(super) async fn get_vector_context(
        &self,
        query_text: &str,
        query: &RAGQuery,
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
                    (20, None)
                }
            }
        };

        // 3. Execute similarity search with complete document data
        let results = similarity_search_documents(
            self.id,
            &query_embedding,
            search_top_k,
            similarity_threshold,
            query.context.as_ref()
                .and_then(|c| c.chat_request.as_ref())
                .and_then(|r| r.file_ids.clone()),
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

    /// Calculate dynamic token allocation (LightRAG naive_query token management)
    pub(super) async fn calculate_token_budget(&self, query: &RAGQuery) -> RAGResult<TokenBudget> {
        // 1. Get tokenizer and configuration
        let max_total_tokens = get_max_total_tokens();
        let tokenizer = get_tokenizer();

        // 2. Calculate conversation history tokens
        // For now, skip conversation history since we need user_id which isn't available in RAG testing context
        // TODO: When RAG is integrated with chat, add user_id to RAGQuery to enable conversation history
        let history_context = String::new();
        let history_tokens = tokenizer.count_tokens(&history_context);

        // 3. Calculate system prompt template overhead (empty content_data)
        let response_type = "Multiple Paragraphs"; // Use default response type
        let user_prompt = self.rag_instance.instance.engine_settings.simple_vector
            .as_ref()
            .and_then(|s| s.querying.as_ref())
            .and_then(|q| q.user_prompt.as_deref())
            .unwrap_or("");

        // Create sample system prompt to calculate overhead (LightRAG pattern)
        let sample_system_prompt = NAIVE_RAG_RESPONSE_TEMPLATE
            .replace("{content_data}", "")
            .replace("{response_type}", response_type)
            .replace("{history}", &history_context)
            .replace("{user_prompt}", user_prompt);

        let system_prompt_overhead = tokenizer.count_tokens(&sample_system_prompt);

        // 4. Calculate query tokens
        let query_tokens = tokenizer.count_tokens(&query.text);

        // 5. Calculate total system prompt overhead (template + query tokens like LightRAG)
        let total_system_overhead = system_prompt_overhead + query_tokens;

        // 6. Calculate available tokens for chunks
        let buffer_tokens = 100; // Safety buffer like LightRAG
        let used_tokens = total_system_overhead + buffer_tokens;
        let available_chunk_tokens = max_total_tokens.saturating_sub(used_tokens);

        tracing::debug!(
            "Token budget - Total: {}, System: {}, Query: {}, History: {}, Buffer: {}, Available for chunks: {}",
            max_total_tokens, total_system_overhead, query_tokens, history_tokens, buffer_tokens, available_chunk_tokens
        );

        Ok(TokenBudget {
            max_total_tokens,
            system_prompt_overhead: total_system_overhead,
            query_tokens,
            history_tokens,
            buffer_tokens,
            available_chunk_tokens,
        })
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

    /// Generate LLM response using formatted context (LightRAG pattern)
    pub(super) async fn generate_llm_response(
        &self,
        query_text: &str,
        context_data: &str,
        _query: &RAGQuery,
        token_budget: &TokenBudget,
    ) -> RAGResult<String> {
        // 1. Get LLM model
        let llm_model = self
            .rag_instance
            .models
            .llm_model
            .as_ref()
            .ok_or(RAGErrorCode::Querying(
                RAGQueryingErrorCode::LlmModelUnavailable,
            ))?;

        // 2. Format system prompt (LightRAG naive_rag_response pattern)
        let response_type = "Multiple Paragraphs"; // Use default response type
        // For now, skip conversation history since we need user_id which isn't available in RAG testing context
        // TODO: When RAG is integrated with chat, add user_id to RAGQuery to enable conversation history
        let history_context = String::new();
        let user_prompt = self.rag_instance.instance.engine_settings.simple_vector
            .as_ref()
            .and_then(|s| s.querying.as_ref())
            .and_then(|q| q.user_prompt.as_deref())
            .unwrap_or("");

        let system_prompt = NAIVE_RAG_RESPONSE_TEMPLATE
            .replace("{content_data}", context_data)
            .replace("{response_type}", response_type)
            .replace("{history}", &history_context)
            .replace("{user_prompt}", user_prompt);

        // 3. Log token usage like LightRAG
        let tokenizer = get_tokenizer();
        let total_prompt_tokens =
            tokenizer.count_tokens(&format!("{}\n\n{}", system_prompt, query_text));
        tracing::debug!(
            "Sending to LLM: {} tokens (Query: {}, System: {})",
            total_prompt_tokens,
            token_budget.query_tokens,
            tokenizer.count_tokens(&system_prompt)
        );

        // 4. Calculate max tokens for response generation (LightRAG pattern)
        let total_prompt_tokens =
            tokenizer.count_tokens(&format!("{}\n\n{}", system_prompt, query_text));
        let max_response_tokens = token_budget.max_total_tokens.saturating_sub(total_prompt_tokens);

        // Validate we have enough tokens for a meaningful response
        if max_response_tokens < 50 {
            tracing::warn!(
                "Very limited tokens for response: {} (total: {}, prompt: {})",
                max_response_tokens,
                token_budget.max_total_tokens,
                total_prompt_tokens
            );
        }

        // 5. Generate response using AIModel
        let completion_request = crate::ai::SimplifiedChatRequest {
            messages: vec![
                crate::ai::core::providers::ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.clone().into(),
                },
                crate::ai::core::providers::ChatMessage {
                    role: "user".to_string(),
                    content: query_text.to_string().into(),
                },
            ],
            stream: false, // Streaming is handled by chat feature, not RAG
        };

        let response = llm_model.chat(completion_request).await.map_err(|e| {
            tracing::error!("LLM generation failed: {}", e);
            RAGErrorCode::Querying(RAGQueryingErrorCode::LlmGenerationFailed)
        })?;

        // Extract response content
        let content = response.content;

        // Post-process response
        Ok(post_process_llm_response(
            content,
            query_text,
            &system_prompt,
        ))
    }

    /// Build retrieval-only response
    pub(super) fn build_retrieval_response(
        &self,
        documents_with_scores: Vec<(SimpleVectorDocument, f32)>,
        processing_time: u64,
    ) -> RAGQueryResponse {
        let sources: Vec<RAGSource> = documents_with_scores
            .into_iter()
            .map(|(document, similarity_score)| RAGSource {
                document,
                similarity_score,
                entity_matches: Vec::new(),    // TODO: Implement entity extraction
                relationship_matches: Vec::new(), // TODO: Implement relationship extraction
            })
            .collect();

        let mut metadata = HashMap::new();
        metadata.insert(
            "chunks_retrieved".to_string(),
            serde_json::json!(sources.len()),
        );

        RAGQueryResponse {
            answer: String::new(), // Empty for retrieval-only
            sources,
            mode_used: QueryMode::Bypass,
            confidence_score: None,
            processing_time_ms: processing_time,
            metadata,
        }
    }

    /// Build generation response with LLM answer and token budget metadata
    pub(super) fn build_generation_response(
        &self,
        answer: String,
        documents_with_scores: Vec<(SimpleVectorDocument, f32)>,
        processing_time: u64,
        mode: QueryMode,
        token_budget: &TokenBudget,
    ) -> RAGQueryResponse {
        let sources: Vec<RAGSource> = documents_with_scores
            .into_iter()
            .map(|(document, similarity_score)| RAGSource {
                document,
                similarity_score,
                entity_matches: Vec::new(),    // TODO: Implement entity extraction
                relationship_matches: Vec::new(), // TODO: Implement relationship extraction
            })
            .collect();

        // Include token budget information in metadata for debugging/monitoring
        let mut metadata = HashMap::new();
        metadata.insert("chunks_used".to_string(), serde_json::json!(sources.len()));
        metadata.insert(
            "max_total_tokens".to_string(),
            serde_json::json!(token_budget.max_total_tokens),
        );
        metadata.insert(
            "system_prompt_overhead".to_string(),
            serde_json::json!(token_budget.system_prompt_overhead),
        );
        metadata.insert(
            "query_tokens".to_string(),
            serde_json::json!(token_budget.query_tokens),
        );
        metadata.insert(
            "history_tokens".to_string(),
            serde_json::json!(token_budget.history_tokens),
        );
        metadata.insert(
            "buffer_tokens".to_string(),
            serde_json::json!(token_budget.buffer_tokens),
        );
        metadata.insert(
            "available_chunk_tokens".to_string(),
            serde_json::json!(token_budget.available_chunk_tokens),
        );

        RAGQueryResponse {
            answer,
            sources,
            mode_used: mode,
            confidence_score: None, // TODO: Calculate confidence
            processing_time_ms: processing_time,
            metadata,
        }
    }

    /// Graceful failure handling
    pub(super) fn handle_empty_results(&self, query: &RAGQuery, processing_time: u64) -> RAGQueryResponse {
        RAGQueryResponse {
            answer: FAIL_RESPONSE.to_string(),
            sources: vec![],
            mode_used: query.mode.clone(),
            confidence_score: Some(0.0),
            processing_time_ms: processing_time,
            metadata: HashMap::new(),
        }
    }

    /// Complete RAG query processing
    pub async fn query_impl(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
        let start_time = std::time::Instant::now();

        tracing::info!(
            "Starting RAG query: {} (mode: {:?})",
            query.text,
            query.mode
        );

        // 1. Vector Context Retrieval (_get_vector_context equivalent)
        let raw_chunks = self.get_vector_context(&query.text, &query).await?;

        if raw_chunks.is_empty() {
            tracing::warn!("No relevant chunks found for query: {}", query.text);
            return Ok(self.handle_empty_results(&query, start_time.elapsed().as_millis() as u64));
        }

        // 2. Token Budget Calculation (LightRAG's dynamic token management)
        let token_budget = self.calculate_token_budget(&query).await?;

        // 3. Unified Chunk Processing (process_chunks_unified equivalent)
        let processed_chunks = self
            .process_chunks_unified(
                &query.text,
                raw_chunks,
                &query,
                token_budget.available_chunk_tokens,
            )
            .await?;

        if processed_chunks.is_empty() {
            tracing::warn!("No chunks survived processing for query: {}", query.text);
            return Ok(self.handle_empty_results(&query, start_time.elapsed().as_millis() as u64));
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        match query.mode {
            QueryMode::Bypass => {
                // Return just the sources without LLM generation
                Ok(self.build_retrieval_response(processed_chunks, processing_time))
            }
            QueryMode::Naive | QueryMode::Local | QueryMode::Global | QueryMode::Hybrid | QueryMode::Mix => {
                // 4. Context Assembly (format chunks for LLM)
                let context_data = format_chunks_as_context(&processed_chunks);

                // 5. LLM Response Generation
                let llm_response = self
                    .generate_llm_response(&query.text, &context_data, &query, &token_budget)
                    .await?;

                Ok(self.build_generation_response(
                    llm_response,
                    processed_chunks,
                    processing_time,
                    query.mode,
                    &token_budget,
                ))
            }
        }
    }
}