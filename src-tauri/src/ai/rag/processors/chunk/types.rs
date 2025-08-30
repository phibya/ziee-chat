// Chunking types and configurations

use serde::{Deserialize, Serialize};

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

/// Enhanced chunking strategies from simple_vector.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    TokenBased {
        max_tokens: usize,
        overlap_tokens: usize,
        preserve_sentence_boundaries: bool,
    },
    CharacterDelimited {
        delimiter: String,
        split_only: bool,
        max_chunk_size: Option<usize>,
        recursive_splitting: bool,
    },
    Hybrid {
        primary: Box<ChunkingStrategy>,
        fallback: Box<ChunkingStrategy>,
        switch_threshold: usize,
    },
    Adaptive {
        content_type: ContentType,
        dynamic_sizing: bool,
        quality_threshold: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    PlainText,
    Markdown,
    Code,
    Academic,
    Technical,
    Legal,
}
