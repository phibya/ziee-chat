// Utility functions for Simple Vector RAG Engine

use crate::ai::rag::{RAGResult, ConversationMessage, SimpleVectorDocument};

/// Token budget allocation for query processing
#[derive(Debug, Clone)]
pub struct TokenBudget {
    pub max_total_tokens: usize,
    pub system_prompt_overhead: usize,
    pub query_tokens: usize,
    pub history_tokens: usize,
    pub buffer_tokens: usize,
    pub available_chunk_tokens: usize,
}

/// Simple tokenizer implementation for token counting
pub struct SimpleTokenizer;

impl SimpleTokenizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn count_tokens(&self, text: &str) -> usize {
        // Simplified token counting: roughly 4 characters per token
        // This is a rough approximation and should be replaced with proper tokenizer
        (text.len() + 3) / 4
    }
}

/// Chunk deduplication algorithm (LightRAG process_chunks_unified)
pub fn deduplicate_chunks_by_id(
    chunks: Vec<(SimpleVectorDocument, f32)>
) -> Vec<(SimpleVectorDocument, f32)> {
    let mut seen_ids = std::collections::HashSet::new();
    let mut deduplicated = Vec::new();
    let original_count = chunks.len();
    
    for (document, score) in chunks {
        let chunk_id = format!("{}_{}", document.file_id, document.chunk_index);
        
        if seen_ids.insert(chunk_id) {
            deduplicated.push((document, score));
        }
    }
    
    tracing::debug!("Deduplicated {} chunks to {} unique chunks", 
                   original_count, deduplicated.len());
    deduplicated
}

/// Truncate chunks by token size (LightRAG truncate_list_by_token_size algorithm)
pub async fn truncate_chunks_by_tokens(
    chunks: Vec<(SimpleVectorDocument, f32)>,
    max_token_size: usize,
    tokenizer: &SimpleTokenizer,
) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
    if max_token_size == 0 {
        return Ok(vec![]);
    }
    
    let mut total_tokens = 0;
    let mut truncated_chunks = Vec::new();
    let original_count = chunks.len();
    
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
        original_count,
        total_tokens,
        max_token_size
    );
    
    Ok(truncated_chunks)
}

/// Format chunks as context for LLM (LightRAG context formatting pattern)
pub fn format_chunks_as_context(chunks: &[(SimpleVectorDocument, f32)]) -> String {
    let mut context_parts = Vec::new();
    
    for (index, (document, similarity_score)) in chunks.iter().enumerate() {
        // Format each chunk with metadata like LightRAG
        let chunk_context = format!(
            "## Document Chunk {} (File: {}, Similarity: {:.3})\n{}\n",
            index + 1,
            document.file_id,
            similarity_score,
            document.content.trim()
        );
        context_parts.push(chunk_context);
    }
    
    context_parts.join("\n")
}

/// Advanced conversation history processing (LightRAG get_conversation_turns)
pub fn format_conversation_history(history: &[ConversationMessage], num_turns: usize) -> String {
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
    while i < filtered_messages.len().saturating_sub(1) {
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
        let skip_count = turns.len() - num_turns;
        turns = turns.into_iter().skip(skip_count).collect();
    }
    
    // 4. Format turns as string
    turns.into_iter().map(|(user_msg, assistant_msg)| {
        format!("user: {}\nassistant: {}", user_msg.content, assistant_msg.content)
    }).collect::<Vec<_>>().join("\n")
}

/// Response post-processing (LightRAG response cleaning)
pub fn post_process_llm_response(response: String, query: &str, system_prompt: &str) -> String {
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

/// Two-stage reranking algorithm (LightRAG apply_rerank_if_enabled)
pub async fn apply_rerank_if_enabled(
    _query_text: &str,
    chunks: Vec<(SimpleVectorDocument, f32)>
) -> RAGResult<Vec<(SimpleVectorDocument, f32)>> {
    // For now, just return chunks unchanged since reranking model is not implemented
    // TODO: Implement actual reranking when rerank models are available
    tracing::debug!("Reranking requested but not implemented, returning original chunks");
    Ok(chunks)
}

/// Get maximum total tokens for the model (LightRAG default: 30000)
pub fn get_max_total_tokens() -> usize {
    // LightRAG default is 30000 tokens - should be configurable from engine settings
    // TODO: Get from engine settings: self.rag_instance.instance.engine_settings.simple_vector.querying.max_total_tokens()
    30000
}

/// Get tokenizer for token counting (simplified implementation)
pub fn get_tokenizer() -> SimpleTokenizer {
    SimpleTokenizer::new()
}