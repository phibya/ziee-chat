// Type definitions for Simple Vector RAG Engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Multi-Pass Gleaning Processor from LightRAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPassGleaningProcessor {
    pub max_gleaning_rounds: u32,
    pub continuation_detection: bool,
    pub history_tracking: bool,
    pub merge_strategy: GleaningMergeStrategy,
    pub gleaning_cache_enabled: bool,
}

/// Enterprise-Grade Caching Infrastructure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseCachingSystem {
    pub llm_response_cache: bool,
    pub chunk_storage_cache: bool,
    pub cache_similarity_threshold: f32,
    pub cache_compression: bool,
    pub cache_expiry_hours: u32,
}

/// Cross-Process Synchronization Manager from Revolutionary Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProcessSynchronizationManager {
    pub shared_memory_coordination: bool,
    pub process_lock_timeout_ms: u64,
    pub inter_process_communication: InterProcessCommunicationConfig,
    pub distributed_cache_sync: bool,
    pub coordination_strategy: CoordinationStrategy,
}

/// Inter-Process Communication Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterProcessCommunicationConfig {
    pub message_queue_enabled: bool,
    pub shared_memory_pool_size: usize,
    pub process_heartbeat_interval_ms: u64,
    pub coordination_port: Option<u16>,
}

/// Coordination Strategy for Multi-Process Operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStrategy {
    LockBased {
        max_wait_time_ms: u64,
        fair_locking: bool,
    },
    LeaderElection {
        election_timeout_ms: u64,
        leadership_lease_duration_ms: u64,
    },
    Consensus {
        quorum_size: usize,
        consensus_timeout_ms: u64,
    },
}

/// Process Lock for Cross-Process Coordination
#[derive(Debug, Clone)]
pub struct ProcessLock {
    pub lock_key: String,
    pub lock_id: i64,
    pub acquired_at: DateTime<Utc>,
    pub strategy: CoordinationStrategy,
}

/// Candidate Document for Reranking
#[derive(Debug, Clone)]
pub struct CandidateDocument {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub metadata: HashMap<String, String>,
    pub reranking_metadata: HashMap<String, String>,
    pub source_path: Option<String>,
}

/// Token Allocation Result
#[derive(Debug, Clone)]
pub struct TokenAllocation {
    pub allocated_tokens: u64,
    pub provider: String,
    pub allocation_strategy: String,
    pub estimated_cost: f64,
}

/// Enterprise Reranking Infrastructure from Revolutionary Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseRerankingInfrastructure {
    pub reranking_enabled: bool,
    pub multi_provider_support: MultiProviderRerankingConfig,
    pub advanced_scoring: AdvancedScoringConfig,
    pub hybrid_reranking: HybridRerankingStrategy,
    pub performance_optimization: RerankingPerformanceConfig,
}

/// Multi-Provider Reranking Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProviderRerankingConfig {
    pub primary_provider: RerankingProvider,
    pub fallback_providers: Vec<RerankingProvider>,
    pub provider_switching_threshold: f64,
    pub cross_provider_validation: bool,
}

/// Reranking Provider Options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RerankingProvider {
    Cohere {
        model: String,
        api_key: Option<String>,
        top_k: usize,
    },
    OpenAI {
        model: String,
        api_key: Option<String>,
        similarity_threshold: f64,
    },
    SentenceTransformers {
        model_path: String,
        device: String,
        batch_size: usize,
    },
    Custom {
        endpoint: String,
        headers: HashMap<String, String>,
        request_format: String,
    },
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

/// Hybrid Reranking Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HybridRerankingStrategy {
    Sequential {
        stages: Vec<RerankingStage>,
        early_stopping_threshold: Option<f64>,
    },
    Ensemble {
        rerankers: Vec<RerankingProvider>,
        combination_method: EnsembleCombinationMethod,
        weights: Vec<f64>,
    },
    Adaptive {
        query_complexity_threshold: f64,
        simple_strategy: Box<HybridRerankingStrategy>,
        complex_strategy: Box<HybridRerankingStrategy>,
    },
}

/// Reranking Stage Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankingStage {
    pub name: String,
    pub provider: RerankingProvider,
    pub input_size: usize,
    pub output_size: usize,
    pub score_threshold: Option<f64>,
}

/// Ensemble Combination Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnsembleCombinationMethod {
    WeightedAverage,
    RankFusion,
    BordaCount,
    ReciprocalRankFusion,
    LearningToRank,
}

/// Reranking Performance Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankingPerformanceConfig {
    pub batch_processing: bool,
    pub batch_size: usize,
    pub parallel_processing: bool,
    pub max_parallel_requests: usize,
    pub caching_enabled: bool,
    pub cache_ttl_seconds: u32,
    pub timeout_ms: u64,
}

/// Unified Token Control System from Ultimate Implementation Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTokenControlSystem {
    pub token_management_enabled: bool,
    pub sophisticated_token_tracking: SophisticatedTokenTrackingConfig,
    pub dynamic_token_allocation: DynamicTokenAllocationConfig,
    pub token_optimization: TokenOptimizationConfig,
    pub cross_provider_token_coordination: CrossProviderTokenConfig,
}

/// Sophisticated Token Tracking Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SophisticatedTokenTrackingConfig {
    pub real_time_tracking: bool,
    pub token_usage_analytics: bool,
    pub usage_prediction_enabled: bool,
    pub cost_estimation_enabled: bool,
    pub quota_monitoring: QuotaMonitoringConfig,
    pub usage_alerting: UsageAlertingConfig,
}

/// Quota Monitoring Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaMonitoringConfig {
    pub daily_quota_limit: Option<u64>,
    pub hourly_quota_limit: Option<u64>,
    pub per_operation_limit: Option<u64>,
    pub provider_specific_limits: HashMap<String, u64>,
    pub soft_limit_percentage: f64, // Alert at this percentage of limit
}

/// Usage Alerting Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAlertingConfig {
    pub alert_threshold_percentage: f64,
    pub cost_alert_threshold: Option<f64>,
    pub notification_channels: Vec<AlertChannel>,
    pub rate_limit_alerts: bool,
}

/// Alert Channel Options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Log {
        level: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
    Email {
        recipients: Vec<String>,
    },
    Database {
        table: String,
    },
}

/// Dynamic Token Allocation Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTokenAllocationConfig {
    pub adaptive_allocation: bool,
    pub priority_based_allocation: bool,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub allocation_algorithms: Vec<AllocationAlgorithm>,
    pub reallocation_triggers: Vec<ReallocationTrigger>,
}

/// Load Balancing Strategy for Token Allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin {
        weights: HashMap<String, f64>,
    },
    LeastUsed,
    ResponseTimeBased,
    CostOptimized,
    Hybrid {
        primary: Box<LoadBalancingStrategy>,
        fallback: Box<LoadBalancingStrategy>,
    },
}

/// Token Allocation Algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationAlgorithm {
    EvenDistribution,
    PriorityQueue {
        levels: u8,
    },
    WeightedFairQueuing {
        weights: HashMap<String, f64>,
    },
    TokenBucket {
        bucket_size: u64,
        refill_rate: u64,
    },
    SlidingWindow {
        window_size_ms: u64,
        max_tokens: u64,
    },
}

/// Reallocation Triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReallocationTrigger {
    UsageThreshold { percentage: f64 },
    ResponseTime { max_latency_ms: u64 },
    ErrorRate { max_error_rate: f64 },
    CostThreshold { max_cost_per_token: f64 },
    TimeWindow { interval_ms: u64 },
}

/// Token Optimization Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOptimizationConfig {
    pub compression_enabled: bool,
    pub deduplication_enabled: bool,
    pub caching_strategy: TokenCachingStrategy,
    pub batch_optimization: BatchOptimizationConfig,
    pub prompt_optimization: PromptOptimizationConfig,
}

/// Token Caching Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenCachingStrategy {
    Disabled,
    LRU {
        max_cache_size: usize,
    },
    LFU {
        max_cache_size: usize,
    },
    TTL {
        ttl_seconds: u32,
        max_cache_size: usize,
    },
    Adaptive {
        initial_size: usize,
        growth_factor: f64,
    },
}

/// Batch Optimization Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptimizationConfig {
    pub auto_batching: bool,
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub size_optimization: bool,
    pub compression_threshold: usize,
}

/// Prompt Optimization Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptimizationConfig {
    pub template_optimization: bool,
    pub redundancy_removal: bool,
    pub compression_techniques: Vec<CompressionTechnique>,
    pub length_optimization: bool,
    pub semantic_preservation: bool,
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

/// Cross-Provider Token Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProviderTokenConfig {
    pub unified_quota_management: bool,
    pub cross_provider_failover: bool,
    pub cost_optimization: bool,
    pub provider_selection_strategy: ProviderSelectionStrategy,
    pub token_exchange_rates: HashMap<String, f64>,
}

/// Provider Selection Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderSelectionStrategy {
    CostOptimal,
    PerformanceOptimal,
    AvailabilityFirst,
    Balanced {
        cost_weight: f64,
        performance_weight: f64,
        availability_weight: f64,
    },
    Custom {
        algorithm: String,
        parameters: HashMap<String, f64>,
    },
}

/// Advanced Compression Engine for Vector Storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCompressionEngine {
    pub compression_algorithm: CompressionAlgorithm,
    pub float_precision: FloatPrecision,
    pub quantization_enabled: bool,
    pub quantization_bits: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Zlib,
    Lz4,
    Zstd,
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

/// Linear Gradient Weighted Polling Algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearGradientWeightedPolling {
    pub max_related_chunks: usize,
    pub min_related_chunks: usize,
    pub importance_decay_factor: f64,
    pub quota_redistribution: bool,
}