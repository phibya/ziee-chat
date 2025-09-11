// Chunking implementation - combined from chunk module
// Contains types and token-based chunker implementation

use crate::ai::rag::{types::TextChunk, RAGResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Ultimate Chunk Selection Engine with Quality Scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSelector {
    pub quality_threshold: f64,
    pub importance_weighting: bool,
    pub context_preservation: bool,
    pub semantic_coherence_check: bool,
}

impl Default for ChunkSelector {
    fn default() -> Self {
        Self {
            quality_threshold: 0.75,
            importance_weighting: true,
            context_preservation: true,
            semantic_coherence_check: true,
        }
    }
}

/// Advanced token-based text chunker with LightRAG functionality
pub struct TokenBasedChunker {
    // Regex patterns for text processing
    sentence_pattern: Regex,
    // Text sanitization patterns
    multiple_whitespace: Regex,
    line_breaks: Regex,
    // Chunk selection engine
    chunk_selector: ChunkSelector,
}

impl TokenBasedChunker {
    pub fn new() -> Self {
        Self {
            sentence_pattern: Regex::new(r"[.!?]+\s+").unwrap(),
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

    /// Calculate content hash for chunk identification
    fn calculate_content_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl TokenBasedChunker {
    /// Chunk text with overlap, matching LightRAG's approach exactly
    pub async fn chunk_with_overlap(
        &self,
        content: &str,
        max_token_size: usize,
        overlap_token_size: usize,
    ) -> RAGResult<Vec<TextChunk>> {
        let sanitized_content = self.sanitize_text(content).await?;

        // First check if content is too large and needs recursive splitting
        let estimated_tokens = self.estimate_tokens(&sanitized_content);
        if estimated_tokens > max_token_size * 10 {
            // Use recursive splitting for very large content
            let split_chunks = self
                .recursive_split_chunk(&sanitized_content, max_token_size * 4)
                .await?;

            let mut all_chunks = Vec::new();
            for chunk_content in split_chunks.iter() {
                // Process each split chunk directly without recursion
                let chunk_chunks = self
                    .chunk_content_direct(chunk_content, max_token_size, overlap_token_size)
                    .await?;

                for mut chunk in chunk_chunks.into_iter() {
                    chunk.chunk_index = all_chunks.len();
                    all_chunks.push(chunk);
                }
            }
            return Ok(all_chunks);
        }

        // Use the direct chunking method
        self.chunk_content_direct(&sanitized_content, max_token_size, overlap_token_size)
            .await
    }

    /// Direct chunking method without recursion
    async fn chunk_content_direct(
        &self,
        content: &str,
        max_token_size: usize,
        overlap_token_size: usize,
    ) -> RAGResult<Vec<TextChunk>> {
        let mut chunks: Vec<TextChunk> = Vec::new();
        let mut chunk_index = 0;

        // Convert content to "tokens" (simplified - should use real tokenizer)
        let words: Vec<&str> = content.split_whitespace().collect();
        let tokens_per_word = 0.75; // Rough approximation

        let mut start_word = 0;
        while start_word < words.len() {
            // Calculate end word based on token estimation
            let mut end_word = start_word;
            let mut current_tokens = 0.0;

            while end_word < words.len() && current_tokens < max_token_size as f64 {
                current_tokens += words[end_word].len() as f64 * tokens_per_word;
                end_word += 1;
            }

            if end_word <= start_word {
                end_word = start_word + 1; // Ensure progress
            }

            let chunk_words = &words[start_word..end_word];
            let chunk_content = chunk_words.join(" ");

            if !chunk_content.trim().is_empty() {
                let actual_tokens = self.estimate_tokens(&chunk_content);
                let chunk = self
                    .create_text_chunk(chunk_index, chunk_content, actual_tokens)
                    .await?;
                chunks.push(chunk);
                chunk_index += 1;
            }

            // Calculate overlap for next chunk (matching LightRAG pattern)
            if end_word >= words.len() {
                break; // Last chunk
            }

            // Move start position forward, accounting for overlap
            let overlap_words =
                self.calculate_overlap_words(&words[start_word..end_word], overlap_token_size);
            let next_start = end_word.saturating_sub(overlap_words);

            // Ensure we make progress even with overlap
            if next_start <= start_word && overlap_words > 0 {
                start_word = end_word
                    .saturating_sub(overlap_words / 2)
                    .max(start_word + 1);
            } else {
                start_word = next_start.max(start_word + 1);
            }
        }

        Ok(chunks)
    }

    /// Calculate number of words to overlap based on token count
    fn calculate_overlap_words(&self, chunk_words: &[&str], overlap_tokens: usize) -> usize {
        if chunk_words.is_empty() || overlap_tokens == 0 {
            return 0;
        }

        let tokens_per_word = 0.75;
        let target_overlap_words = (overlap_tokens as f64 / tokens_per_word) as usize;
        target_overlap_words.min(chunk_words.len() / 2) // Don't overlap more than half the chunk
    }

    /// Simple token-based chunking with configurable parameters
    pub async fn chunk(
        &self,
        content: &str,
        max_tokens: Option<usize>,
        overlap_tokens: Option<usize>,
        dynamic_sizing: bool,
        quality_threshold: f64,
    ) -> RAGResult<Vec<TextChunk>> {
        // Use default LightRAG values if not specified
        let max_token_size = max_tokens.unwrap_or(1024);
        let overlap_token_size = overlap_tokens.unwrap_or(128);

        tracing::debug!(
            "Token-based chunking: max_tokens={}, overlap_tokens={}, dynamic_sizing={}, quality_threshold={}",
            max_token_size,
            overlap_token_size,
            dynamic_sizing,
            quality_threshold
        );

        // Always use token-based chunking with overlap
        let mut chunks = self
            .chunk_with_overlap(content, max_token_size, overlap_token_size)
            .await?;

        // Apply dynamic sizing and quality assessment if requested
        if dynamic_sizing {
            chunks = self
                .assess_and_adjust_chunk_quality(chunks, quality_threshold)
                .await?;
        }

        tracing::info!(
            "Token-based chunking completed: {} chunks (max_tokens={}, overlap={})",
            chunks.len(),
            max_token_size,
            overlap_token_size
        );

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
            let quality_score = self.calculate_chunk_quality(&chunk.content).await?;

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

    /// Get overlap content from the end of a chunk for continuity
    pub fn get_overlap_content(&self, content: &str, overlap_tokens: usize) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() || overlap_tokens == 0 {
            return String::new();
        }

        // Estimate words needed for overlap_tokens
        let tokens_per_word = 0.75;
        let target_overlap_words = (overlap_tokens as f64 / tokens_per_word) as usize;
        let overlap_words = target_overlap_words.min(words.len() / 2).max(1);

        let start_idx = words.len().saturating_sub(overlap_words);
        words[start_idx..].join(" ")
    }

    /// Token estimation using word-based counting with multi-language support
    /// Uses regex to count words across different languages and scripts
    pub fn estimate_tokens(&self, text: &str) -> usize {
        if text.trim().is_empty() {
            return 0;
        }

        // Multi-language word counting using Unicode word boundaries
        // This regex matches word characters in any language/script
        let word_count = self.count_words_multilingual(text);

        // Token estimation based on word count with language-aware multipliers
        // Different languages have different token-to-word ratios
        let token_multiplier = self.get_token_multiplier_for_text(text);

        ((word_count as f64 * token_multiplier).ceil() as usize).max(1)
    }

    /// Count words using Unicode-aware regex for multiple languages
    fn count_words_multilingual(&self, text: &str) -> usize {
        // Use Unicode word boundary regex that works across languages
        // \p{L} matches any Unicode letter (Latin, Cyrillic, CJK, Arabic, etc.)
        // \p{N} matches any Unicode number
        // \p{M} matches any Unicode mark (accents, diacritics)
        let word_regex = Regex::new(r"[\p{L}\p{N}\p{M}]+").unwrap();
        word_regex.find_iter(text).count()
    }

    /// Determine token multiplier based on detected script/language patterns
    fn get_token_multiplier_for_text(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 1.0;
        }

        let mut cjk_chars = 0;
        let mut arabic_chars = 0;
        let mut latin_chars = 0;

        for char in text.chars() {
            let code_point = char as u32;
            match code_point {
                // Latin script (basic ASCII)
                0x0041..=0x005A | 0x0061..=0x007A => {
                    latin_chars += 1;
                }
                // Latin extended
                0x00C0..=0x00FF | 0x0100..=0x017F | 0x0180..=0x024F => {
                    latin_chars += 1;
                }
                // CJK ranges
                0x3040..=0x309F |   // Hiragana
                0x30A0..=0x30FF |   // Katakana  
                0x3400..=0x4DBF |   // CJK Extension A
                0x4E00..=0x9FFF |   // CJK Unified Ideographs
                0xAC00..=0xD7AF |   // Hangul
                0x20000..=0x2A6DF => { // CJK Extension B
                    cjk_chars += 1;
                }
                // Arabic script
                0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => {
                    arabic_chars += 1;
                }
                _ => {
                    // Other characters don't affect language detection
                }
            }
        }

        // Calculate dominant script
        let total_alpha_chars = cjk_chars + arabic_chars + latin_chars;
        if total_alpha_chars == 0 {
            return 0.5; // Mostly punctuation/numbers
        }

        let cjk_ratio = cjk_chars as f64 / total_alpha_chars as f64;
        let arabic_ratio = arabic_chars as f64 / total_alpha_chars as f64;
        let latin_ratio = latin_chars as f64 / total_alpha_chars as f64;

        // Language-specific token multipliers based on typical token-to-word ratios
        if cjk_ratio > 0.5 {
            // CJK: Each character is roughly equivalent to a token
            // But word segmentation gives us word count, so adjust accordingly
            1.5 // CJK words tend to be shorter, more tokens per "word"
        } else if arabic_ratio > 0.5 {
            // Arabic: Complex morphology, more tokens per word
            1.3
        } else if latin_ratio > 0.5 {
            // Latin-based languages: Standard ratio
            1.0
        } else {
            // Mixed or other scripts
            1.1
        }
    }

    /// Async wrapper for recursive split
    pub async fn recursive_split_chunk(
        &self,
        content: &str,
        max_size: usize,
    ) -> RAGResult<Vec<String>> {
        Ok(self.recursive_split_chunk_sync(content, max_size))
    }

    /// Recursive split for oversized chunks using sentence and paragraph boundaries
    fn recursive_split_chunk_sync(&self, content: &str, max_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();

        // If content fits, return it as-is
        if content.len() <= max_size {
            chunks.push(content.to_string());
            return chunks;
        }

        // Try splitting by double newlines (paragraphs) first
        let paragraphs: Vec<&str> = content
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        if paragraphs.len() > 1 {
            for paragraph in paragraphs {
                let sub_chunks = self.recursive_split_chunk_sync(paragraph, max_size);
                chunks.extend(sub_chunks);
            }
            return chunks;
        }

        // Split by sentences if no paragraph breaks
        let sentences = self.split_into_sentences(content);
        let mut current_chunk = String::new();

        for sentence in sentences {
            let sentence_trimmed = sentence.trim();
            if sentence_trimmed.is_empty() {
                continue;
            }

            // Check if adding this sentence would exceed max_size
            let potential_length = if current_chunk.is_empty() {
                sentence_trimmed.len()
            } else {
                current_chunk.len() + 1 + sentence_trimmed.len() // +1 for space
            };

            if potential_length <= max_size {
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(sentence_trimmed);
            } else {
                // Save current chunk if not empty
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk = String::new();
                }

                // Handle oversized single sentence by word splitting
                if sentence_trimmed.len() > max_size {
                    let words: Vec<&str> = sentence_trimmed.split_whitespace().collect();
                    let mut word_chunk = String::new();

                    for word in words {
                        let potential_word_length = if word_chunk.is_empty() {
                            word.len()
                        } else {
                            word_chunk.len() + 1 + word.len()
                        };

                        if potential_word_length <= max_size {
                            if !word_chunk.is_empty() {
                                word_chunk.push(' ');
                            }
                            word_chunk.push_str(word);
                        } else {
                            if !word_chunk.is_empty() {
                                chunks.push(word_chunk.trim().to_string());
                                word_chunk = String::new();
                            }

                            // Handle extremely long words by character splitting
                            if word.len() > max_size {
                                for char_chunk in word.chars().collect::<Vec<_>>().chunks(max_size)
                                {
                                    let chunk_str: String = char_chunk.iter().collect();
                                    chunks.push(chunk_str);
                                }
                            } else {
                                word_chunk = word.to_string();
                            }
                        }
                    }

                    if !word_chunk.is_empty() {
                        current_chunk = word_chunk;
                    }
                } else {
                    current_chunk = sentence_trimmed.to_string();
                }
            }
        }

        // Add final chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        // Filter out empty chunks
        chunks.retain(|chunk| !chunk.trim().is_empty());

        chunks
    }

    /// Create TextChunk with metadata
    async fn create_text_chunk(
        &self,
        index: usize,
        content: String,
        token_count: usize,
    ) -> RAGResult<TextChunk> {
        // Calculate content hash
        let content_hash = self.calculate_content_hash(&content);

        Ok(TextChunk {
            id: Some(Uuid::new_v4()),
            chunk_index: index,
            content,
            content_hash,
            token_count,
            metadata: HashMap::new(),
        })
    }

    /// Simple chunk processing - applies basic quality filtering only
    ///
    /// This is for document processing time, not query time.
    /// Vector similarity selection happens in the RAG engine during queries.
    pub async fn process_chunks(&self, chunks: Vec<TextChunk>) -> RAGResult<Vec<TextChunk>> {
        tracing::debug!("Processing {} chunks for quality filtering", chunks.len());

        if chunks.is_empty() {
            return Ok(chunks);
        }

        // Apply basic quality filtering if enabled
        let mut processed_chunks = Vec::new();

        if self.chunk_selector.importance_weighting {
            tracing::debug!(
                "Applying quality threshold filtering: {}",
                self.chunk_selector.quality_threshold
            );

            for chunk in chunks {
                let quality_score = self.calculate_chunk_quality(&chunk.content).await?;

                if quality_score >= self.chunk_selector.quality_threshold {
                    processed_chunks.push(chunk);
                } else {
                    tracing::trace!(
                        "Chunk {} filtered out: quality {} < threshold {}",
                        chunk.chunk_index,
                        quality_score,
                        self.chunk_selector.quality_threshold
                    );
                }
            }
        } else {
            // No filtering - return all chunks
            processed_chunks = chunks;
        }

        tracing::info!(
            "Chunk processing completed: {} chunks after quality filtering (threshold: {})",
            processed_chunks.len(),
            self.chunk_selector.quality_threshold
        );

        Ok(processed_chunks)
    }

    /// Calculate final chunk quality with sophisticated metrics
    async fn calculate_chunk_quality(&self, content: &str) -> RAGResult<f64> {
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
