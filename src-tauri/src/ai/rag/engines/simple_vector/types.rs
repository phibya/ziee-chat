// Type definitions for Simple Vector RAG Engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// SimpleVectorDocument moved to crate::ai::rag::models to avoid duplication
// and ensure it includes the embedding field that matches the database schema

/// Multi-pass gleaning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleaningConfig {
    pub max_gleaning_rounds: u32,
    pub merge_strategy: GleaningMergeStrategy,
    pub continuation_detection: bool,
    pub history_tracking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GleaningMergeStrategy {
    NewNamesOnly,
    FullMerge,
    SimilarityBased { threshold: f64 },
}

/// Document processing status with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocProcessingStatus {
    pub content_summary: String,
    pub content_length: usize,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: DocumentStatus,
    pub error_msg: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentStatus {
    Pending,
    Processing,
    Processed,
    Failed,
}

/// Advanced Scoring Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedScoringConfig {
    pub semantic_similarity_weight: f64,
    pub lexical_similarity_weight: f64,
    pub context_coherence_weight: f64,
    pub temporal_relevance_weight: f64,
    pub source_authority_weight: f64,
    pub score_normalization: ScoreNormalizationMethod,
}

/// Score Normalization Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreNormalizationMethod {
    MinMax,
    ZScore,
    Sigmoid,
    SoftMax,
    RankBased,
}

/// Compression Techniques for Prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionTechnique {
    Truncation { max_length: usize },
    Summarization { target_ratio: f64 },
    KeywordExtraction { top_k: usize },
    SemanticCompression { compression_ratio: f64 },
    TemplateReplacement { templates: HashMap<String, String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloatPrecision {
    Float32,
    Float16,
    BFloat16,
}

/// Semantic Overlap Management System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticOverlapManager {
    pub overlap_strategy: OverlapStrategy,
    pub minimum_overlap: usize,
    pub maximum_overlap: usize,
    pub context_window_size: usize,
    pub semantic_boundary_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlapStrategy {
    Semantic,      // Intelligent semantic boundary detection
    Fixed,         // Fixed token overlap
    Dynamic,       // Dynamic based on content analysis
    ContextWindow, // Context window management
}
