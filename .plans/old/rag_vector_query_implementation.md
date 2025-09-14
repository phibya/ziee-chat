# RAG Vector Query Implementation Plan

## Analysis Summary

### Current Implementation Status
- **Indexing**: ✅ Fully implemented with text extraction, chunking, embedding generation, and vector storage
- **Query**: ❌ Currently returns `InvalidQuery` error - needs to be implemented

### Current Architecture Analysis

#### Simple Vector Engine Structure
- **Location**: `src-tauri/src/ai/rag/engines/simple_vector/`
- **Components**:
  - `core.rs`: Main engine struct with indexing pipeline implemented
  - `embeddings.rs`: Batch embedding processing (reusable for queries)
  - `queries.rs`: Database queries for indexing (needs query methods)
  - `types.rs`: Type definitions
  - `overlap.rs`: Overlap management
  - `tokens.rs`: Token management (commented out)

#### Database Schema
- **Table**: `simple_vector_documents`
- **Fields**: `rag_instance_id, file_id, chunk_index, content, content_hash, token_count, embedding, metadata`
- **Vector Extension**: PostgreSQL with pgvector (HalfVector for storage)

#### Existing Infrastructure
- **AI Models**: Already integrated with `RAGModels` containing `embedding_model` and optional `llm_model`
- **Error Handling**: `RAGQueryingErrorCode` with comprehensive error types
- **Type System**: `RAGQuery` and `RAGQueryResponse` structs defined

### LightRAG Query Pattern Analysis

From studying `.other/lightrag.py`, key patterns identified:

#### 1. Vector Retrieval (`_get_vector_context`)
```python
# Query vector database with embedding
results = await chunks_vdb.query(query, top_k=search_top_k, ids=query_param.ids)
```

#### 2. Naive Query Flow (`naive_query`)
```python
# 1. Generate query embedding
# 2. Search vector database  
# 3. Retrieve relevant chunks
# 4. Format context for LLM
# 5. Generate response using LLM
```

#### 3. Response Generation Pattern
- **Context Assembly**: Combine retrieved chunks into context
- **Token Management**: Respect token limits
- **LLM Integration**: Use model to generate final response

## Implementation Plan

### Phase 1: Basic Vector Search
**Implement core vector similarity search functionality**

#### 1.1 Add Query Database Methods (`simple_vector/queries.rs`)
```rust
/// Perform similarity search on vector documents
pub async fn similarity_search(
    instance_id: Uuid,
    query_embedding: &[f32],
    top_k: usize,
    similarity_threshold: Option<f32>,
    file_ids: Option<Vec<Uuid>>,
) -> RAGResult<Vec<VectorSearchResult>>

/// Get document chunks by file IDs for context filtering
pub async fn get_documents_by_files(
    instance_id: Uuid,
    file_ids: Vec<Uuid>
) -> RAGResult<Vec<VectorDocument>>
```

#### 1.2 Add Vector Search Types (`simple_vector/types.rs`)
```rust
/// Vector search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub metadata: serde_json::Value,
    pub similarity_score: f32,
}

/// Complete vector document for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub token_count: i32,
    pub metadata: serde_json::Value,
}
```

### Phase 2: Query Processing Engine - LightRAG naive_query Algorithm
**Implement the main query method following LightRAG's proven naive_query pattern**

#### 2.1 Core Query Flow Algorithm (Based on LightRAG naive_query)
```rust
async fn query(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
    let start_time = std::time::Instant::now();
    
    // 1. Cache Check (optional - implement later)
    // let cache_key = compute_query_hash(&query);
    
    // 2. Vector Context Retrieval (_get_vector_context equivalent)
    let raw_chunks = self.get_vector_context(&query.text, &query).await?;
    
    if raw_chunks.is_empty() {
        return Ok(RAGQueryResponse::empty_response());
    }
    
    // 3. Token Budget Calculation (LightRAG's dynamic token management)
    let token_budget = self.calculate_token_budget(&query).await?;
    
    // 4. Unified Chunk Processing (process_chunks_unified equivalent)
    let processed_chunks = self.process_chunks_unified(
        &query.text,
        raw_chunks,
        &query,
        token_budget.available_chunk_tokens
    ).await?;
    
    match query.mode {
        QueryMode::Retrieval => {
            // Return just the sources without LLM generation
            Ok(self.build_retrieval_response(processed_chunks, start_time))
        },
        QueryMode::Generation => {
            // 5. Context Assembly (format chunks for LLM)
            let context_data = self.format_chunks_as_context(&processed_chunks);
            
            // 6. LLM Response Generation
            let llm_response = self.generate_llm_response(
                &query.text,
                &context_data,
                &query,
                &token_budget
            ).await?;
            
            Ok(self.build_generation_response(llm_response, processed_chunks, start_time))
        }
    }
}
```

#### 2.2 Vector Context Retrieval Algorithm (`_get_vector_context` equivalent)
```rust
/// Retrieve text chunks from vector database (LightRAG _get_vector_context pattern)
async fn get_vector_context(&self, query_text: &str, query: &RAGQuery) -> RAGResult<Vec<VectorSearchResult>> {
    // 1. Generate query embedding
    let query_embedding = self.generate_query_embedding(query_text).await?;
    
    // 2. Determine search parameters
    let search_top_k = query.max_results.unwrap_or(20); // chunk_top_k from LightRAG
    let similarity_threshold = query.similarity_threshold;
    
    // 3. Execute similarity search
    let results = queries::similarity_search(
        self.id,
        &query_embedding,
        search_top_k,
        similarity_threshold,
        query.context.as_ref().and_then(|c| c.file_ids.clone())
    ).await?;
    
    // 4. Convert to standardized format with metadata
    let chunks: Vec<VectorSearchResult> = results.into_iter().map(|result| {
        VectorSearchResult {
            file_id: result.file_id,
            chunk_index: result.chunk_index,
            content: result.content,
            metadata: result.metadata,
            similarity_score: result.similarity_score,
            source_type: "vector".to_string(), // Mark source type like LightRAG
            chunk_id: Some(format!("{}_{}", result.file_id, result.chunk_index)), // For deduplication
        }
    }).collect();
    
    tracing::info!("Vector context retrieval: {} chunks (search_top_k: {})", chunks.len(), search_top_k);
    Ok(chunks)
}
```

#### 2.3 Unified Chunk Processing Algorithm (`process_chunks_unified` equivalent)
```rust
/// Process chunks with deduplication, reranking, and token truncation (LightRAG process_chunks_unified)
async fn process_chunks_unified(
    &self,
    query_text: &str,
    chunks: Vec<VectorSearchResult>,
    query: &RAGQuery,
    available_chunk_tokens: usize,
) -> RAGResult<Vec<VectorSearchResult>> {
    if chunks.is_empty() {
        return Ok(vec![]);
    }
    
    let original_count = chunks.len();
    let mut processed_chunks = chunks;
    
    // 1. Deduplication by chunk_id (LightRAG pattern)
    processed_chunks.dedup_by(|a, b| a.chunk_id == b.chunk_id);
    
    // 2. Reranking (if enabled and rerank model available)
    if query.context.as_ref().map_or(false, |c| c.enable_rerank) {
        processed_chunks = self.apply_rerank_if_enabled(query_text, processed_chunks).await?;
    }
    
    // 3. Token-based truncation (LightRAG truncate_list_by_token_size)
    processed_chunks = self.truncate_chunks_by_tokens(processed_chunks, available_chunk_tokens).await?;
    
    tracing::debug!(
        "Unified chunk processing: {} chunks from {} (available_tokens: {})",
        processed_chunks.len(),
        original_count,
        available_chunk_tokens
    );
    
    Ok(processed_chunks)
}
```

### Phase 3: LLM Integration - Token Management & Context Assembly
**Implement LightRAG's sophisticated token budgeting and context formatting algorithms**

#### 3.1 Token Budget Calculation Algorithm (LightRAG Dynamic Token Management)
```rust
#[derive(Debug, Clone)]
struct TokenBudget {
    max_total_tokens: usize,
    system_prompt_overhead: usize,
    query_tokens: usize,
    history_tokens: usize,
    buffer_tokens: usize,
    available_chunk_tokens: usize,
}

/// Calculate dynamic token allocation (LightRAG naive_query token management)
async fn calculate_token_budget(&self, query: &RAGQuery) -> RAGResult<TokenBudget> {
    // 1. Get tokenizer and configuration
    let max_total_tokens = self.get_max_total_tokens();
    let tokenizer = self.get_tokenizer(); // Will need to implement
    
    // 2. Calculate conversation history tokens
    let history_context = query.context.as_ref()
        .and_then(|c| c.conversation_history.as_ref())
        .map_or(String::new(), |h| self.format_conversation_history(h));
    let history_tokens = tokenizer.count_tokens(&history_context);
    
    // 3. Calculate system prompt template overhead (empty content_data)
    let response_type = query.context.as_ref()
        .and_then(|c| c.response_type.as_ref())
        .unwrap_or("Multiple Paragraphs");
    let user_prompt = query.context.as_ref()
        .and_then(|c| c.user_prompt.as_ref())
        .unwrap_or("");
    
    // Create sample system prompt to calculate overhead (LightRAG pattern)
    let sample_system_prompt = NAIVE_RAG_RESPONSE_TEMPLATE.replace("{content_data}", "")
        .replace("{response_type}", response_type)
        .replace("{history}", &history_context)
        .replace("{user_prompt}", user_prompt);
    
    let system_prompt_overhead = tokenizer.count_tokens(&sample_system_prompt);
    
    // 4. Calculate query tokens
    let query_tokens = tokenizer.count_tokens(&query.text);
    
    // 5. Calculate available tokens for chunks
    let buffer_tokens = 100; // Safety buffer like LightRAG
    let used_tokens = system_prompt_overhead + query_tokens + buffer_tokens;
    let available_chunk_tokens = max_total_tokens.saturating_sub(used_tokens);
    
    tracing::debug!(
        "Token budget - Total: {}, System: {}, Query: {}, History: {}, Buffer: {}, Available for chunks: {}",
        max_total_tokens, system_prompt_overhead, query_tokens, history_tokens, buffer_tokens, available_chunk_tokens
    );
    
    Ok(TokenBudget {
        max_total_tokens,
        system_prompt_overhead,
        query_tokens,
        history_tokens,
        buffer_tokens,
        available_chunk_tokens,
    })
}
```

#### 3.2 Token-Based Chunk Truncation Algorithm (`truncate_list_by_token_size` equivalent)
```rust
/// Truncate chunks by token size (LightRAG truncate_list_by_token_size algorithm)
async fn truncate_chunks_by_tokens(
    &self,
    chunks: Vec<VectorSearchResult>,
    max_token_size: usize,
) -> RAGResult<Vec<VectorSearchResult>> {
    if max_token_size == 0 {
        return Ok(vec![]);
    }
    
    let tokenizer = self.get_tokenizer();
    let mut total_tokens = 0;
    let mut truncated_chunks = Vec::new();
    
    for chunk in chunks {
        // Calculate tokens for this chunk (JSON serialized like LightRAG)
        let chunk_json = serde_json::to_string(&chunk).unwrap_or_default();
        let chunk_tokens = tokenizer.count_tokens(&chunk_json);
        
        if total_tokens + chunk_tokens <= max_token_size {
            total_tokens += chunk_tokens;
            truncated_chunks.push(chunk);
        } else {
            // Stop adding chunks once we hit the limit
            break;
        }
    }
    
    tracing::debug!(
        "Token truncation: {} chunks from {} (used {} of {} tokens)",
        truncated_chunks.len(),
        chunks.len(),
        total_tokens,
        max_token_size
    );
    
    Ok(truncated_chunks)
}
```

#### 3.3 Context Formatting Algorithm (LightRAG context assembly)
```rust
/// Format chunks as context for LLM (LightRAG context formatting pattern)
fn format_chunks_as_context(&self, chunks: &[VectorSearchResult]) -> String {
    let mut context_parts = Vec::new();
    
    for (index, chunk) in chunks.iter().enumerate() {
        // Format each chunk with metadata like LightRAG
        let chunk_context = format!(
            "## Document Chunk {} (File: {}, Similarity: {:.3})\n{}\n",
            index + 1,
            chunk.file_id,
            chunk.similarity_score,
            chunk.content.trim()
        );
        context_parts.push(chunk_context);
    }
    
    context_parts.join("\n")
}

/// Generate LLM response using formatted context (LightRAG pattern)
async fn generate_llm_response(
    &self,
    query_text: &str,
    context_data: &str,
    query: &RAGQuery,
    token_budget: &TokenBudget,
) -> RAGResult<String> {
    // 1. Get LLM model
    let llm_model = self.rag_instance.models.llm_model.as_ref()
        .ok_or(RAGErrorCode::Querying(RAGQueryingErrorCode::LlmModelUnavailable))?;
    
    // 2. Format system prompt (LightRAG naive_rag_response pattern)
    let response_type = query.context.as_ref()
        .and_then(|c| c.response_type.as_ref())
        .unwrap_or("Multiple Paragraphs");
    let history_context = query.context.as_ref()
        .and_then(|c| c.conversation_history.as_ref())
        .map_or(String::new(), |h| self.format_conversation_history(h));
    let user_prompt = query.context.as_ref()
        .and_then(|c| c.user_prompt.as_ref())
        .unwrap_or("");
    
    let system_prompt = NAIVE_RAG_RESPONSE_TEMPLATE
        .replace("{content_data}", context_data)
        .replace("{response_type}", response_type)
        .replace("{history}", &history_context)
        .replace("{user_prompt}", user_prompt);
    
    // 3. Log token usage like LightRAG
    let tokenizer = self.get_tokenizer();
    let total_prompt_tokens = tokenizer.count_tokens(&format!("{}\n\n{}", system_prompt, query_text));
    tracing::debug!(
        "Sending to LLM: {} tokens (Query: {}, System: {})",
        total_prompt_tokens,
        token_budget.query_tokens,
        tokenizer.count_tokens(&system_prompt)
    );
    
    // 4. Generate response using AIModel
    let completion_request = crate::ai::SimplifiedCompletionRequest {
        messages: vec![
            crate::ai::core::providers::ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            crate::ai::core::providers::ChatMessage {
                role: "user".to_string(),
                content: query_text.to_string(),
            },
        ],
        stream: query.context.as_ref().map_or(false, |c| c.stream),
        max_tokens: Some(token_budget.max_total_tokens - total_prompt_tokens),
        temperature: Some(0.7),
    };
    
    let response = llm_model.completions(completion_request).await
        .map_err(|e| {
            tracing::error!("LLM generation failed: {}", e);
            RAGErrorCode::Querying(RAGQueryingErrorCode::LlmGenerationFailed)
        })?;
    
    // Extract response content
    let content = response.choices.first()
        .and_then(|choice| choice.message.as_ref())
        .map(|msg| msg.content.clone())
        .unwrap_or_default();
    
    Ok(content)
}
```

### Phase 4: Advanced Features
**Implement advanced querying capabilities**

#### 4.1 Context Filtering
- File-based filtering using `QueryContext.file_ids`
- Date-range filtering using `QueryContext.date_range`
- Metadata-based filtering

#### 4.2 Result Processing
- Relevance scoring and ranking
- Duplicate detection and removal
- Result summarization

#### 4.3 Caching (Optional)
- Query result caching
- Embedding caching for repeated queries

## Database Queries Implementation

### Core Vector Search Query (PostgreSQL pgvector cosine distance)
```sql
-- LightRAG similarity search equivalent using pgvector
SELECT 
    file_id, 
    chunk_index, 
    content, 
    metadata,
    1 - (embedding <=> $1::halfvec) as similarity_score
FROM simple_vector_documents 
WHERE rag_instance_id = $2
  AND ($3::uuid[] IS NULL OR file_id = ANY($3))
  AND ($4 IS NULL OR 1 - (embedding <=> $1::halfvec) >= $4)
ORDER BY embedding <=> $1::halfvec  -- Cosine distance (0 = identical, 2 = opposite)
LIMIT $5
```

### Batch Search Query for Multiple Files
```sql
-- Get chunks ordered for context assembly
SELECT 
    file_id, 
    chunk_index, 
    content, 
    token_count, 
    metadata,
    content_hash
FROM simple_vector_documents
WHERE rag_instance_id = $1 
  AND file_id = ANY($2)
ORDER BY file_id, chunk_index
```

## Integration Points

### 1. Reuse Existing Infrastructure
- **Embedding Generation**: Reuse `process_embeddings_in_batches` pattern for single query embeddings
- **AI Model Access**: Use existing `self.rag_instance.models.embedding_model` and `llm_model`
- **Error Handling**: Leverage existing `RAGQueryingErrorCode` variants

### 2. Configuration Integration
- Use existing `SimpleVectorSettings` for query parameters
- Respect `max_results`, `similarity_threshold` from settings
- Honor token limits from engine settings

### 3. Database Integration
- Leverage existing database pool access pattern
- Use existing error mapping from database to RAG errors
- Follow existing transaction patterns

## Error Handling Strategy

### Query-Specific Errors
- `InvalidQuery`: Malformed query parameters
- `EmbeddingGenerationFailed`: Query embedding creation fails
- `SimilaritySearchFailed`: Vector search database errors
- `LlmGenerationFailed`: Response generation fails
- `InsufficientContext`: No relevant results found

### Recovery Strategies
- Graceful degradation for LLM failures (return retrieval-only results)
- Fallback similarity thresholds for sparse results
- Timeout handling for long-running queries

## Testing Strategy

### Unit Tests
- Vector search query correctness
- Embedding generation for queries
- Result formatting and conversion

### Integration Tests
- End-to-end query flow
- Multiple file filtering
- Different query modes

### Performance Tests
- Large-scale similarity search
- Memory usage with large result sets
- Response time benchmarks

## Success Metrics

### Functional Requirements ✅
- [ ] Vector similarity search working
- [ ] Query embedding generation
- [ ] Result filtering and ranking
- [ ] LLM response generation
- [ ] Error handling for all failure modes

### Performance Requirements ✅
- [ ] Sub-second response for typical queries
- [ ] Handle 1000+ document chunks efficiently  
- [ ] Memory-efficient result processing
- [ ] Concurrent query support

## Implementation Notes

### Key Differences from LightRAG
- **No Graph Components**: Focus only on vector search, skip entity/relationship extraction
- **Rust-Specific Optimizations**: Leverage Rust's type system and async patterns
- **PostgreSQL pgvector**: Use native PostgreSQL vector operations vs. in-memory vector stores
- **Existing Infrastructure**: Build on established patterns in the codebase

### Compatibility Considerations
- Maintain existing `RAGEngine` trait compatibility
- Preserve current indexing functionality unchanged
- Follow existing error handling patterns
- Use established database query patterns

### Future Extensibility
- Design allows for future graph query integration
- Plugin architecture for custom similarity functions
- Extensible metadata filtering system
- Modular LLM integration for different response types

---

## LightRAG Algorithm Summary

### Key Algorithms Implemented

1. **`naive_query` Main Flow**:
   ```
   Cache Check → Vector Context Retrieval → Token Budget Calculation → 
   Unified Chunk Processing → Context Formatting → LLM Response Generation
   ```

2. **`_get_vector_context` Vector Retrieval**:
   ```
   Query Embedding → Similarity Search → Result Standardization → Metadata Enrichment
   ```

3. **`process_chunks_unified` Processing Pipeline**:
   ```
   Deduplication → Optional Reranking → Token-Based Truncation → Quality Filtering
   ```

4. **Dynamic Token Management**:
   ```
   System Prompt Overhead → History Tokens → Buffer Allocation → Available Chunk Tokens
   ```

5. **`truncate_list_by_token_size` Smart Truncation**:
   ```
   JSON Serialization → Progressive Token Counting → Budget-Respecting Truncation
   ```

### Algorithm Advantages
- **Proven in Production**: LightRAG algorithms are battle-tested in real applications
- **Token-Efficient**: Sophisticated token management maximizes context utilization
- **Flexible**: Supports multiple query modes and response types
- **Scalable**: Handles large document collections efficiently
- **Modular**: Each algorithm component can be optimized independently

This implementation plan provides a comprehensive roadmap for adding robust query functionality to the simple_vector RAG engine, following proven algorithms from LightRAG while adapting them to Rust and PostgreSQL infrastructure.

---

## Critical Missing Details Discovered

### 1. Advanced Reranking System
**Critical Algorithm**: LightRAG uses a sophisticated 2-stage reranking system:

```rust
/// Two-stage reranking algorithm (LightRAG apply_rerank_if_enabled)
async fn apply_rerank_if_enabled(&self, query_text: &str, chunks: Vec<VectorSearchResult>) -> RAGResult<Vec<VectorSearchResult>> {
    // 1. Check if reranking is enabled and rerank model is available
    if !self.rerank_enabled() || !self.has_rerank_model() {
        tracing::warning("Rerank enabled but no rerank model configured");
        return Ok(chunks);
    }
    
    // 2. Extract content for reranking
    let document_texts: Vec<String> = chunks.iter().map(|chunk| {
        // Try multiple content fields (content, text, chunk_content, document)
        chunk.content.clone()
    }).collect();
    
    // 3. Call rerank model with index-based results
    let rerank_results = self.rerank_model.rerank(query_text, &document_texts, chunks.len()).await?;
    
    // 4. Process index-based rerank results
    let mut reranked_chunks = Vec::new();
    for result in rerank_results {
        if let (Some(index), Some(score)) = (result.index, result.relevance_score) {
            if index < chunks.len() {
                let mut chunk = chunks[index].clone();
                chunk.rerank_score = Some(score);  // Add rerank score to chunk
                reranked_chunks.push(chunk);
            }
        }
    }
    
    // 5. Filter by minimum rerank score threshold
    let min_rerank_score = self.get_min_rerank_score(); // Default: 0.5
    reranked_chunks.retain(|chunk| {
        chunk.rerank_score.unwrap_or(1.0) >= min_rerank_score
    });
    
    tracing::info!("Reranked {} chunks from {} (min_score: {})", 
                  reranked_chunks.len(), chunks.len(), min_rerank_score);
    
    Ok(reranked_chunks)
}
```

### 2. Sophisticated Conversation History Processing  
**Critical Algorithm**: LightRAG has complex conversation turn management:

```rust
/// Advanced conversation history processing (LightRAG get_conversation_turns)
fn format_conversation_history(&self, history: &[ConversationMessage], num_turns: usize) -> String {
    if num_turns == 0 {
        return String::new();
    }
    
    // 1. Filter out keyword extraction messages
    let filtered_messages: Vec<&ConversationMessage> = history.iter()
        .filter(|msg| {
            !(msg.role == "assistant" && 
              (msg.content.starts_with(r#"{ "high_level_keywords""#) ||
               msg.content.starts_with(r#"{'high_level_keywords'"#)))
        })
        .collect();
    
    // 2. Group messages into complete turns (user-assistant pairs)
    let mut turns = Vec::new();
    let mut i = 0;
    while i < filtered_messages.len() - 1 {
        let msg1 = &filtered_messages[i];
        let msg2 = &filtered_messages[i + 1];
        
        // Check for user-assistant or assistant-user pairs
        if (msg1.role == "user" && msg2.role == "assistant") ||
           (msg1.role == "assistant" && msg2.role == "user") {
            
            // Always put user message first in turn
            if msg1.role == "assistant" {
                turns.push((msg2, msg1)); // (user, assistant)
            } else {
                turns.push((msg1, msg2)); // (user, assistant)
            }
        }
        i += 2;
    }
    
    // 3. Keep only the most recent num_turns
    if turns.len() > num_turns {
        turns = turns.into_iter().skip(turns.len() - num_turns).collect();
    }
    
    // 4. Format turns as string
    turns.into_iter().map(|(user_msg, assistant_msg)| {
        format!("user: {}\nassistant: {}", user_msg.content, assistant_msg.content)
    }).collect::<Vec<_>>().join("\n")
}
```

### 3. Priority-Based Embedding Generation
**Critical Detail**: LightRAG uses priority system for API calls:

```rust
/// Priority-based embedding generation (LightRAG _priority system)
async fn generate_query_embedding(&self, query_text: &str) -> RAGResult<Vec<f32>> {
    // Higher priority (5) for query embeddings vs indexing embeddings
    let embedding_request = crate::ai::SimplifiedEmbeddingsRequest {
        input: crate::ai::core::providers::EmbeddingsInput::Single(query_text.to_string()),
        encoding_format: Some("float".to_string()),
        dimensions: None,
        priority: Some(5), // Critical: Higher priority for queries
    };
    
    let response = self.rag_instance.models.embedding_model
        .embeddings(embedding_request)
        .await
        .map_err(|e| {
            tracing::error!("Query embedding generation failed: {}", e);
            RAGErrorCode::Querying(RAGQueryingErrorCode::EmbeddingGenerationFailed)
        })?;
    
    Ok(response.data.into_iter().next()
       .map(|d| d.embedding)
       .unwrap_or_default())
}
```

### 4. Response Post-Processing Algorithm
**Critical Detail**: LightRAG cleans LLM responses extensively:

```rust
/// Response post-processing (LightRAG response cleaning)
fn post_process_llm_response(&self, response: String, query: &str, system_prompt: &str) -> String {
    if response.len() <= system_prompt.len() {
        return response;
    }
    
    // Remove system prompt artifacts and common LLM artifacts
    response
        .replace(system_prompt, "")
        .replace("user", "")
        .replace("model", "") 
        .replace(query, "")
        .replace("<system>", "")
        .replace("</system>", "")
        .trim()
        .to_string()
}
```

### 5. Streaming Response Support
**Critical Feature**: LightRAG supports both sync and streaming responses:

```rust
/// Streaming response support (LightRAG pattern)
pub enum RAGQueryResponse {
    Complete {
        answer: String,
        sources: Vec<RAGSource>,
        confidence_score: Option<f32>,
        processing_time_ms: u64,
    },
    Streaming {
        stream: Pin<Box<dyn Stream<Item = String> + Send>>,
        sources: Vec<RAGSource>,
    },
}
```

### 6. Cache-First Architecture
**Critical Algorithm**: LightRAG has sophisticated caching:

```rust
/// Cache-first query processing (LightRAG handle_cache)
async fn query_with_cache(&self, query: RAGQuery) -> RAGResult<RAGQueryResponse> {
    // 1. Compute cache hash from all query parameters
    let cache_hash = self.compute_query_hash(&query);
    
    // 2. Check cache first
    if let Some(cached_response) = self.get_cached_response(&cache_hash).await? {
        tracing::debug!("Cache hit for query: {}", cache_hash);
        return Ok(cached_response);
    }
    
    // 3. Process query if not cached
    let response = self.process_query_without_cache(query).await?;
    
    // 4. Save to cache before returning
    self.save_response_to_cache(&cache_hash, &response).await?;
    
    Ok(response)
}
```

### 7. Chunk Deduplication by ID
**Critical Detail**: LightRAG deduplicates by `chunk_id`, not content:

```rust
/// Chunk deduplication algorithm (LightRAG process_chunks_unified)
fn deduplicate_chunks_by_id(&self, chunks: Vec<VectorSearchResult>) -> Vec<VectorSearchResult> {
    let mut seen_ids = std::collections::HashSet::new();
    let mut deduplicated = Vec::new();
    
    for chunk in chunks {
        let chunk_id = chunk.chunk_id.as_ref()
            .unwrap_or(&format!("{}_{}", chunk.file_id, chunk.chunk_index));
        
        if seen_ids.insert(chunk_id.clone()) {
            deduplicated.push(chunk);
        }
    }
    
    tracing::debug!("Deduplicated {} chunks to {} unique chunks", 
                   chunks.len(), deduplicated.len());
    deduplicated
}
```

### 8. Fail-Safe Response Handling
**Critical Pattern**: LightRAG returns specific fail responses:

```rust
const FAIL_RESPONSE: &str = "Sorry, I couldn't find relevant information to answer your question.";

/// Graceful failure handling
async fn handle_empty_results(&self, query: &RAGQuery) -> RAGQueryResponse {
    RAGQueryResponse {
        answer: FAIL_RESPONSE.to_string(),
        sources: vec![],
        mode_used: query.mode.clone(),
        confidence_score: Some(0.0),
        processing_time_ms: 0,
        metadata: std::collections::HashMap::new(),
    }
}
```

These critical details ensure the implementation matches LightRAG's production-tested reliability and performance characteristics.