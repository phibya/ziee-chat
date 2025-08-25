// Text chunking service using token-based approach (similar to tiktoken)

use crate::ai::rag::{
    services::ServiceHealth,
    types::{ChunkingConfig, TextChunk},
    RAGError, RAGResult,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use uuid::Uuid;

/// Chunking service trait
#[async_trait]
pub trait ChunkingService: Send + Sync {
    /// Split text into chunks based on configuration
    async fn chunk_text(
        &self,
        text: &str,
        file_id: Uuid,
        config: ChunkingConfig,
    ) -> RAGResult<Vec<TextChunk>>;

    /// Count tokens in text (approximate)
    fn count_tokens(&self, text: &str) -> usize;

    /// Validate chunking configuration
    fn validate_config(&self, config: &ChunkingConfig) -> RAGResult<()>;

    /// Health check
    async fn health_check(&self) -> RAGResult<ServiceHealth>;
}

/// Token-based text chunker implementation
pub struct TokenBasedChunker {
    // Simple tokenization patterns
    word_pattern: Regex,
    sentence_pattern: Regex,
    paragraph_pattern: Regex,
}

impl TokenBasedChunker {
    pub fn new() -> Self {
        Self {
            word_pattern: Regex::new(r"\b\w+\b").unwrap(),
            sentence_pattern: Regex::new(r"[.!?]+\s+").unwrap(),
            paragraph_pattern: Regex::new(r"\n\s*\n").unwrap(),
        }
    }

    /// Approximate token count using simple word-based heuristic
    /// Real implementation would use tiktoken-like tokenization
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough approximation: 1 token ≈ 0.75 words for English text
        let word_count = self.word_pattern.find_iter(text).count();
        ((word_count as f32) * 1.33) as usize
    }

    /// Split text into sentences
    fn split_into_sentences<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.sentence_pattern
            .split(text)
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    /// Split text into paragraphs
    fn split_into_paragraphs<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.paragraph_pattern
            .split(text)
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    /// Create overlapping windows for better context preservation
    fn create_overlapping_chunks(
        &self,
        sentences: Vec<String>,
        max_chunk_tokens: usize,
        overlap_tokens: usize,
    ) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_tokens = 0;
        let mut sentence_buffer: Vec<String> = Vec::new();
        let mut buffer_tokens = 0;

        for sentence in sentences {
            let sentence_tokens = self.estimate_tokens(&sentence);
            
            // If adding this sentence would exceed the limit, finalize current chunk
            if current_tokens + sentence_tokens > max_chunk_tokens && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                
                // Create overlap from sentence buffer
                let mut overlap_chunk = String::new();
                let mut overlap_token_count = 0;
                
                // Add sentences from the end of the buffer for overlap
                for buffered_sentence in sentence_buffer.iter().rev() {
                    let buffered_tokens = self.estimate_tokens(buffered_sentence);
                    if overlap_token_count + buffered_tokens <= overlap_tokens {
                        if overlap_chunk.is_empty() {
                            overlap_chunk = buffered_sentence.clone();
                        } else {
                            overlap_chunk = format!("{} {}", buffered_sentence, overlap_chunk);
                        }
                        overlap_token_count += buffered_tokens;
                    } else {
                        break;
                    }
                }
                
                // Start new chunk with overlap
                current_chunk = overlap_chunk;
                current_tokens = overlap_token_count;
                sentence_buffer.clear();
            }
            
            // Add current sentence to chunk
            if current_chunk.is_empty() {
                current_chunk = sentence.clone();
            } else {
                current_chunk = format!("{} {}", current_chunk, sentence);
            }
            current_tokens += sentence_tokens;
            
            // Keep track of recent sentences for overlap
            sentence_buffer.push(sentence);
            buffer_tokens += sentence_tokens;
            
            // Limit buffer size to prevent memory issues
            while buffer_tokens > overlap_tokens * 2 && !sentence_buffer.is_empty() {
                let removed = sentence_buffer.remove(0);
                buffer_tokens -= self.estimate_tokens(&removed);
            }
        }

        // Add the last chunk if it has content
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Calculate content hash for chunk deduplication
    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[async_trait]
impl ChunkingService for TokenBasedChunker {
    async fn chunk_text(
        &self,
        text: &str,
        file_id: Uuid,
        config: ChunkingConfig,
    ) -> RAGResult<Vec<TextChunk>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        // Validate configuration
        self.validate_config(&config)?;

        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        // Handle different chunking strategies based on configuration
        let text_chunks = if config.preserve_paragraph_boundaries {
            // Split by paragraphs first, then by sentences if needed
            let paragraphs = self.split_into_paragraphs(text);
            let mut processed_chunks = Vec::new();

            for paragraph in paragraphs {
                let paragraph_tokens = self.estimate_tokens(paragraph);
                
                if paragraph_tokens <= config.max_chunk_size {
                    // Paragraph fits in one chunk
                    processed_chunks.push(paragraph.to_string());
                } else {
                    // Split paragraph into smaller chunks
                    let sentences: Vec<String> = self
                        .split_into_sentences(paragraph)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    
                    let paragraph_chunks = self.create_overlapping_chunks(
                        sentences,
                        config.max_chunk_size,
                        config.chunk_overlap,
                    );
                    
                    processed_chunks.extend(paragraph_chunks);
                }
            }
            processed_chunks
        } else if config.preserve_sentence_boundaries {
            // Split by sentences with overlap
            let sentences: Vec<String> = self
                .split_into_sentences(text)
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            
            self.create_overlapping_chunks(
                sentences,
                config.max_chunk_size,
                config.chunk_overlap,
            )
        } else {
            // Simple token-based splitting without boundary preservation
            let words: Vec<&str> = text.split_whitespace().collect();
            let mut current_chunk = String::new();
            let mut current_tokens = 0;
            let mut processed_chunks = Vec::new();

            for word in words {
                let word_tokens = self.estimate_tokens(word);
                
                if current_tokens + word_tokens > config.max_chunk_size && !current_chunk.is_empty() {
                    processed_chunks.push(current_chunk.trim().to_string());
                    
                    // Create overlap
                    let overlap_words: Vec<&str> = current_chunk
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .take_while(|&w| {
                            let overlap_tokens = self.estimate_tokens(&current_chunk.split_whitespace().rev().take_while(|&word| word != w).collect::<Vec<_>>().join(" "));
                            overlap_tokens <= config.chunk_overlap
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    
                    current_chunk = overlap_words.join(" ");
                    current_tokens = self.estimate_tokens(&current_chunk);
                }
                
                if current_chunk.is_empty() {
                    current_chunk = word.to_string();
                } else {
                    current_chunk = format!("{} {}", current_chunk, word);
                }
                current_tokens += word_tokens;
            }
            
            if !current_chunk.trim().is_empty() {
                processed_chunks.push(current_chunk.trim().to_string());
            }
            
            processed_chunks
        };

        // Create TextChunk objects
        for chunk_text in text_chunks {
            if chunk_text.trim().is_empty() || 
               chunk_text.trim().len() < config.min_chunk_size {
                continue;
            }

            let token_count = self.estimate_tokens(&chunk_text);
            let content_hash = self.calculate_content_hash(&chunk_text);
            
            let mut metadata = HashMap::new();
            metadata.insert("chunk_method".to_string(), serde_json::Value::String("token_based".to_string()));
            metadata.insert("preserve_sentences".to_string(), serde_json::Value::Bool(config.preserve_sentence_boundaries));
            metadata.insert("preserve_paragraphs".to_string(), serde_json::Value::Bool(config.preserve_paragraph_boundaries));
            metadata.insert("created_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

            chunks.push(TextChunk {
                id: None, // Will be set when saved to database
                content: chunk_text.trim().to_string(),
                content_hash,
                token_count,
                chunk_index,
                file_id,
                metadata,
            });

            chunk_index += 1;
        }

        Ok(chunks)
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.estimate_tokens(text)
    }

    fn validate_config(&self, config: &ChunkingConfig) -> RAGResult<()> {
        if config.max_chunk_size == 0 {
            return Err(RAGError::ConfigurationError(
                "max_chunk_size must be greater than 0".to_string(),
            ));
        }

        if config.chunk_overlap >= config.max_chunk_size {
            return Err(RAGError::ConfigurationError(
                "chunk_overlap must be less than max_chunk_size".to_string(),
            ));
        }

        if config.min_chunk_size > config.max_chunk_size {
            return Err(RAGError::ConfigurationError(
                "min_chunk_size must be less than or equal to max_chunk_size".to_string(),
            ));
        }

        Ok(())
    }

    async fn health_check(&self) -> RAGResult<ServiceHealth> {
        let start_time = std::time::Instant::now();
        
        // Test chunking with sample text
        let test_text = "This is the first sentence. This is the second sentence. This is a paragraph.\n\nThis is another paragraph with multiple sentences. It should be split appropriately.";
        let test_file_id = Uuid::new_v4();
        let test_config = ChunkingConfig::default();
        
        match self.chunk_text(test_text, test_file_id, test_config).await {
            Ok(chunks) => {
                if chunks.len() > 0 && chunks.iter().all(|chunk| !chunk.content.is_empty()) {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    Ok(ServiceHealth {
                        is_healthy: true,
                        status: crate::ai::rag::services::ServiceStatus::Healthy,
                        error_message: None,
                        response_time_ms: Some(response_time),
                        last_check: chrono::Utc::now(),
                    })
                } else {
                    Ok(ServiceHealth {
                        is_healthy: false,
                        status: crate::ai::rag::services::ServiceStatus::Error,
                        error_message: Some("Health check failed: no valid chunks generated".to_string()),
                        response_time_ms: None,
                        last_check: chrono::Utc::now(),
                    })
                }
            }
            Err(e) => Ok(ServiceHealth {
                is_healthy: false,
                status: crate::ai::rag::services::ServiceStatus::Error,
                error_message: Some(format!("Health check failed: {}", e)),
                response_time_ms: None,
                last_check: chrono::Utc::now(),
            }),
        }
    }
}