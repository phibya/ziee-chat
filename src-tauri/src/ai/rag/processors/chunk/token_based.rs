// Token-based chunker implementation

use crate::ai::rag::{
    types::{ChunkingConfig, TextChunk},
    RAGError, RAGResult,
};
use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    r#trait::ChunkingProcessor,
    types::{ChunkingStrategy, ContentType, ChunkSelector},
};

/// Advanced token-based text chunker with LightRAG functionality
pub struct TokenBasedChunker {
    // Regex patterns for text processing
    word_pattern: Regex,
    sentence_pattern: Regex,
    paragraph_pattern: Regex,
    // Text sanitization patterns
    multiple_whitespace: Regex,
    line_breaks: Regex,
    // Chunk selection engine
    chunk_selector: ChunkSelector,
}

impl TokenBasedChunker {
    pub fn new() -> Self {
        Self {
            word_pattern: Regex::new(r"\b\w+\b").unwrap(),
            sentence_pattern: Regex::new(r"[.!?]+\s+").unwrap(),
            paragraph_pattern: Regex::new(r"\n\s*\n").unwrap(),
            multiple_whitespace: Regex::new(r"\s+").unwrap(),
            line_breaks: Regex::new(r"\n\r?").unwrap(),
            chunk_selector: ChunkSelector::default(),
        }
    }

    /// Sanitize text for processing
    async fn sanitize_text(&self, text: &str) -> RAGResult<String> {
        // Remove excessive whitespace and normalize line breaks
        let cleaned = self.multiple_whitespace.replace_all(text, " ");
        let normalized = self.line_breaks.replace_all(&cleaned, "\n");
        Ok(normalized.trim().to_string())
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

    /// Tokenize text into words (simple tokenization)
    fn tokenize(&self, text: &str) -> RAGResult<Vec<String>> {
        Ok(self
            .word_pattern
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect())
    }

    /// Assess chunk quality (placeholder implementation)
    async fn assess_chunk_quality(
        &self,
        content: &str,
        _quality_threshold: f64,
    ) -> RAGResult<f64> {
        let mut score = 0.5; // Base score

        // Length-based scoring
        let char_count = content.len();
        if char_count > 100 {
            score += 0.1;
        }
        if char_count > 500 {
            score += 0.1;
        }

        // Word count scoring
        let words = self.word_pattern.find_iter(content).count();
        if words > 20 {
            score += 0.1;
        }

        // Sentence structure scoring
        let sentences = self.split_into_sentences(content);
        if sentences.len() > 2 {
            score += 0.1;
        }

        // Vocabulary diversity (simplified)
        let unique_words: std::collections::HashSet<_> = self
            .word_pattern
            .find_iter(content)
            .map(|m| m.as_str().to_lowercase())
            .collect();
        let vocabulary_ratio = unique_words.len() as f64 / words.max(1) as f64;
        score += vocabulary_ratio * 0.2;

        Ok(score.min(1.0))
    }

    /// Calculate coherence score for content
    async fn calculate_coherence_score(&self, content: &str) -> RAGResult<f64> {
        let mut score = 0.6; // Base score
        let mut weight_sum: f64 = 1.0;

        // Sentence count analysis
        let sentences = self.split_into_sentences(content);
        if sentences.len() >= 2 && sentences.len() <= 10 {
            score += 0.7 * 0.2;
            weight_sum += 0.2;
        }

        // Average sentence length
        if !sentences.is_empty() {
            let avg_sentence_len: f64 = sentences.iter().map(|s| s.len()).sum::<usize>() as f64
                / sentences.len() as f64;
            if (20.0..200.0).contains(&avg_sentence_len) {
                score += 0.8 * 0.15;
                weight_sum += 0.15;
            }
        }

        // Paragraph structure
        let paragraphs = self.split_into_paragraphs(content);
        if paragraphs.len() <= 5 {
            score += 0.7 * 0.1;
            weight_sum += 0.1;
        }

        // Word density - check for reasonable text density
        let word_count = self.word_pattern.find_iter(content).count();
        let char_count = content.len();
        if char_count > 0 {
            let density = word_count as f64 / char_count as f64;
            if (0.08..0.25).contains(&density) {
                // Reasonable text density
                let density_score = 0.9;
                score += density_score * 0.3;
                weight_sum += 0.3;
            }
        }

        // Check for structural integrity
        let structural_score = self.calculate_structural_integrity(content);
        score += structural_score * 0.15;
        weight_sum += 0.15;

        Ok(score / weight_sum.max(1.0))
    }

    /// Calculate content hash for chunk identification
    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[async_trait]
impl ChunkingProcessor for TokenBasedChunker {
    /// Advanced chunking with LightRAG token-based approach
    /// LightRAG uses: max_token_size=1024, overlap_token_size=128
    async fn advanced_chunk_text(&self, content: &str, file_id: Uuid) -> RAGResult<Vec<TextChunk>> {
        let sanitized_content = self.sanitize_text(content).await?;

        // Use simple token-based chunking like LightRAG
        // LightRAG uses: max_token_size=1024, overlap_token_size=128
        let max_token_size = 1024;
        let overlap_token_size = 128;

        // Simple token estimation (will be replaced with actual tokenizer)
        let total_tokens = self.estimate_tokens(&sanitized_content);
        let mut chunks: Vec<TextChunk> = Vec::new();
        let mut chunk_index = 0;

        // Split content into chunks with overlap, matching LightRAG pattern
        let mut start = 0;
        while start < total_tokens {
            let end = (start + max_token_size).min(total_tokens);

            // Extract content for this token range (simplified - actual tokenizer would be better)
            let char_start = (start * sanitized_content.len()) / total_tokens.max(1);
            let char_end = (end * sanitized_content.len()) / total_tokens.max(1);
            let chunk_content = sanitized_content
                .get(char_start..char_end)
                .unwrap_or(&sanitized_content[char_start..])
                .trim()
                .to_string();

            if !chunk_content.is_empty() {
                let actual_tokens = self.estimate_tokens(&chunk_content);
                let chunk = self
                    .create_text_chunk(file_id, chunk_index, chunk_content, actual_tokens)
                    .await?;
                chunks.push(chunk);
                chunk_index += 1;
            }

            // Move to next chunk position with overlap
            start += max_token_size - overlap_token_size;
        }

        Ok(chunks)
    }

    async fn chunk_text(
        &self,
        text: &str,
        file_id: Uuid,
        _config: ChunkingConfig,
    ) -> RAGResult<Vec<TextChunk>> {
        // Use advanced_chunk_text as default implementation
        self.advanced_chunk_text(text, file_id).await
    }

    /// Hybrid chunking strategy with intelligent fallback
    async fn chunk_hybrid(
        &self,
        content: &str,
        file_id: Uuid,
        primary: &ChunkingStrategy,
        fallback: &ChunkingStrategy,
        switch_threshold: usize,
    ) -> RAGResult<Vec<TextChunk>> {
        // Try primary strategy first
        let primary_result = match primary {
            ChunkingStrategy::TokenBased {
                max_tokens,
                overlap_tokens: _,
                preserve_sentence_boundaries: _,
            } => {
                if content.len() / 4 <= *max_tokens {
                    self.advanced_chunk_text(content, file_id).await
                } else {
                    return Err(RAGError::ChunkingError(
                        "Content too large for token-based chunking".to_string(),
                    ));
                }
            }
            _ => self.advanced_chunk_text(content, file_id).await, // Default fallback
        };

        match primary_result {
            Ok(chunks) => {
                if chunks.len() < switch_threshold {
                    // Switch to fallback strategy
                    tracing::warn!(
                        "Primary chunking strategy produced {} chunks, switching to fallback",
                        chunks.len()
                    );
                    match fallback {
                        ChunkingStrategy::TokenBased {
                            max_tokens: _,
                            overlap_tokens: _,
                            preserve_sentence_boundaries: _,
                        } => self.advanced_chunk_text(content, file_id).await,
                        _ => Ok(chunks), // Use primary results if fallback not supported
                    }
                } else {
                    Ok(chunks)
                }
            }
            Err(_) => {
                // Fallback on error
                tracing::warn!("Primary chunking strategy failed, using fallback");
                match fallback {
                    ChunkingStrategy::TokenBased {
                        max_tokens: _,
                        overlap_tokens: _,
                        preserve_sentence_boundaries: _,
                    } => self.advanced_chunk_text(content, file_id).await,
                    _ => self.advanced_chunk_text(content, file_id).await, // Default fallback
                }
            }
        }
    }

    /// Token estimation matching LightRAG's approach
    /// LightRAG uses: tokenizer.encode(text) -> len(tokens)  
    /// This is a placeholder that should be replaced with actual tokenizer integration
    fn estimate_tokens(&self, text: &str) -> usize {
        // TODO: Replace with actual tokenizer like LightRAG
        // LightRAG: _tokens = tokenizer.encode(chunk); len(_tokens)
        // Temporary rough estimation: ~4 characters per token for English
        (text.len() / 4).max(1)
    }

    fn count_tokens(&self, text: &str) -> usize {
        self.estimate_tokens(text)
    }

    fn validate_config(&self, config: &ChunkingConfig) -> RAGResult<()> {
        if config.max_chunk_size == 0 {
            return Err(RAGError::ChunkingError(
                "Chunk size cannot be zero".to_string(),
            ));
        }
        if config.chunk_overlap >= config.max_chunk_size {
            return Err(RAGError::ChunkingError(
                "Overlap size must be less than chunk size".to_string(),
            ));
        }
        Ok(())
    }

    /// Calculate chunk quality based on content characteristics
    fn calculate_chunk_quality(&self, content: &str) -> f64 {
        let length = content.len() as f64;
        let word_count = self.word_pattern.find_iter(content).count() as f64;
        let sentence_count = self
            .sentence_pattern
            .split(content)
            .filter(|s| !s.trim().is_empty())
            .count() as f64;

        // Basic quality metrics
        let length_factor = (length / 1000.0).min(1.0);
        let word_density = if length > 0.0 {
            word_count / (length / 100.0)
        } else {
            0.0
        };
        let sentence_factor = if word_count > 0.0 {
            sentence_count / word_count * 10.0
        } else {
            0.0
        };

        // Combine factors (weights can be adjusted)
        (length_factor * 0.4 + word_density.min(1.0) * 0.4 + sentence_factor.min(1.0) * 0.2)
            .min(1.0)
    }

    /// Recursive split for oversized chunks
    async fn recursive_split_chunk(
        &self,
        content: &str,
        max_size: usize,
    ) -> RAGResult<Vec<String>> {
        let mut chunks = Vec::new();

        if content.len() <= max_size {
            chunks.push(content.to_string());
            return Ok(chunks);
        }

        // Split by sentences first
        let sentences = self.split_into_sentences(content);
        let mut current_chunk = String::new();

        for sentence in sentences {
            if current_chunk.len() + sentence.len() <= max_size {
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(sentence);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = String::new();
                }

                // If single sentence is too large, split by words
                if sentence.len() > max_size {
                    let words: Vec<&str> = sentence.split_whitespace().collect();
                    let mut word_chunk = String::new();

                    for word in words {
                        if word_chunk.len() + word.len() + 1 <= max_size {
                            if !word_chunk.is_empty() {
                                word_chunk.push(' ');
                            }
                            word_chunk.push_str(word);
                        } else {
                            if !word_chunk.is_empty() {
                                chunks.push(word_chunk);
                                word_chunk = String::new();
                            }
                            word_chunk.push_str(word);
                        }
                    }

                    if !word_chunk.is_empty() {
                        current_chunk = word_chunk;
                    }
                } else {
                    current_chunk = sentence.to_string();
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        Ok(chunks)
    }

    /// Create TextChunk with metadata
    async fn create_text_chunk(
        &self,
        file_id: Uuid,
        index: usize,
        content: String,
        token_count: usize,
    ) -> RAGResult<TextChunk> {
        // Calculate content hash
        let content_hash = self.calculate_content_hash(&content);

        Ok(TextChunk {
            id: Some(Uuid::new_v4()),
            file_id,
            chunk_index: index,
            content,
            content_hash,
            token_count,
            metadata: HashMap::new(),
        })
    }

    /// Ultimate chunk selection with quality scoring
    async fn select_ultimate_chunks(&self, chunks: Vec<TextChunk>) -> RAGResult<Vec<TextChunk>> {
        // TODO: Replace with actual vector similarity selection like LightRAG
        // LightRAG: pick_by_vector_similarity(query, text_chunks_storage, chunks_vdb, num_of_chunks, entity_info, embedding_func)
        // Steps: 1) Get query embedding, 2) Get chunk embeddings from vector DB, 3) Calculate cosine similarities, 4) Sort by similarity

        // Temporary implementation: return all chunks (should be replaced with vector similarity search)
        // Real implementation should:
        // 1. Get embeddings for all chunks from vector database
        // 2. Calculate cosine similarity with query embedding
        // 3. Sort by similarity score (highest first)
        // 4. Return top-k chunks based on similarity

        // For now, filter by quality threshold if quality scoring is enabled
        let mut filtered_chunks = Vec::new();

        if self.chunk_selector.importance_weighting {
            for chunk in chunks {
                let quality_score = self
                    .calculate_ultimate_chunk_quality(&chunk.content)
                    .await?;
                if quality_score >= self.chunk_selector.quality_threshold {
                    filtered_chunks.push(chunk);
                }
            }
        } else {
            filtered_chunks = chunks;
        }

        tracing::info!(
            "Ultimate chunk selection completed: {} chunks returned (quality threshold: {})",
            filtered_chunks.len(),
            self.chunk_selector.quality_threshold
        );

        Ok(filtered_chunks)
    }

    /// Adaptive chunking based on content type (from simple_vector.rs)
    async fn chunk_adaptive(
        &self,
        content: &str,
        file_id: Uuid,
        content_type: &ContentType,
        dynamic_sizing: bool,
        quality_threshold: f64,
    ) -> RAGResult<Vec<TextChunk>> {
        // Choose strategy based on content type
        let _strategy = match content_type {
            ContentType::PlainText => ChunkingStrategy::TokenBased {
                max_tokens: 512,
                overlap_tokens: 64,
                preserve_sentence_boundaries: true,
            },
            ContentType::Markdown => ChunkingStrategy::CharacterDelimited {
                delimiter: "##".to_string(),
                split_only: false,
                max_chunk_size: Some(2048),
                recursive_splitting: true,
            },
            ContentType::Code => ChunkingStrategy::CharacterDelimited {
                delimiter: "\n\n".to_string(),
                split_only: false,
                max_chunk_size: Some(1024),
                recursive_splitting: true,
            },
            ContentType::Academic | ContentType::Technical | ContentType::Legal => {
                ChunkingStrategy::TokenBased {
                    max_tokens: 768,
                    overlap_tokens: 96,
                    preserve_sentence_boundaries: true,
                }
            }
        };

        let mut chunks = self.advanced_chunk_text(content, file_id).await?;

        if dynamic_sizing {
            chunks = self
                .assess_and_adjust_chunk_quality(chunks, quality_threshold)
                .await?;
        }

        Ok(chunks)
    }

    /// Assess chunk quality and adjust as needed (from simple_vector.rs)
    async fn assess_and_adjust_chunk_quality(
        &self,
        chunks: Vec<TextChunk>,
        quality_threshold: f64,
    ) -> RAGResult<Vec<TextChunk>> {
        let mut improved_chunks = Vec::new();

        for chunk in chunks {
            let quality_score = self.calculate_chunk_quality(&chunk.content);

            if quality_score >= quality_threshold {
                improved_chunks.push(chunk);
            } else {
                tracing::debug!(
                    "Chunk quality {} below threshold {}, attempting improvement",
                    quality_score,
                    quality_threshold
                );
                // For now, just keep the chunk but mark it for potential improvement
                improved_chunks.push(chunk);
            }
        }

        Ok(improved_chunks)
    }

    /// Get overlap content for chunking (from simple_vector.rs)
    fn get_overlap_content(&self, content: &str, overlap_tokens: usize) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        let overlap_words = overlap_tokens.min(words.len());
        words[words.len().saturating_sub(overlap_words)..].join(" ")
    }

    /// Calculate ultimate chunk quality with sophisticated metrics
    async fn calculate_ultimate_chunk_quality(&self, content: &str) -> RAGResult<f64> {
        let mut score = 0.0;
        let mut weight_sum: f64 = 0.0;

        // Content length score (prefer medium-sized chunks)
        let length_score = if content.len() < 100 {
            0.3
        } else if content.len() > 2000 {
            0.6
        } else {
            1.0 - (content.len() as f64 - 1000.0).abs() / 1000.0
        };
        score += length_score * 0.25;
        weight_sum += 0.25;

        // Sentence completeness score
        let sentence_score =
            if content.ends_with('.') || content.ends_with('!') || content.ends_with('?') {
                1.0
            } else if content.contains('.') {
                0.7
            } else {
                0.4
            };
        score += sentence_score * 0.2;
        weight_sum += 0.2;

        // Information density score
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        let density_score = if chars > 0 {
            (words as f64 / chars as f64 * 100.0).min(1.0)
        } else {
            0.0
        };
        score += density_score * 0.2;
        weight_sum += 0.2;

        // Structural integrity score
        let structural_score = self.calculate_structural_integrity(content);
        score += structural_score * 0.15;
        weight_sum += 0.15;

        // Semantic richness score
        let semantic_score = self.calculate_semantic_richness(content);
        score += semantic_score * 0.2;
        weight_sum += 0.2;

        Ok(score / weight_sum)
    }
}

// Additional helper methods for TokenBasedChunker (not part of the trait)
impl TokenBasedChunker {
    /// Calculate structural integrity of content
    fn calculate_structural_integrity(&self, content: &str) -> f64 {
        let mut score = 0.8; // Base structural score

        // Check for balanced punctuation
        let open_parens = content.chars().filter(|&c| c == '(').count();
        let close_parens = content.chars().filter(|&c| c == ')').count();
        let open_brackets = content.chars().filter(|&c| c == '[').count();
        let close_brackets = content.chars().filter(|&c| c == ']').count();

        if open_parens != close_parens || open_brackets != close_brackets {
            score *= 0.9;
        }

        // Check for reasonable sentence structure
        let sentences = content.split('.').count();
        let words = content.split_whitespace().count();
        if sentences > 0 && words / sentences > 50 {
            score *= 0.8; // Very long sentences
        }

        score
    }

    /// Calculate semantic richness of content
    fn calculate_semantic_richness(&self, content: &str) -> f64 {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }

        // Unique word ratio
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        let uniqueness_ratio = unique_words.len() as f64 / words.len() as f64;

        // Vocabulary sophistication (simplified)
        let avg_word_length: f64 =
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / words.len() as f64;

        let length_score = (avg_word_length / 10.0).min(1.0);

        (uniqueness_ratio + length_score) / 2.0
    }
}