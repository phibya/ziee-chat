use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Supporting enums
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub enum RAGChunkSelectionMethod {
    #[serde(rename = "weight")]
    Weight,
    #[serde(rename = "vector")]
    Vector,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub enum RAGSimpleGraphQueryMode {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "hybrid")]
    Hybrid,
    #[serde(rename = "naive")]
    Naive,
    #[serde(rename = "mix")]
    Mix,
    #[serde(rename = "bypass")]
    Bypass,
}

// Vector Engine Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleVectorIndexingSettings {
    pub chunk_token_size: Option<i32>,
    pub chunk_overlap_token_size: Option<i32>,
    pub max_parallel_insert: Option<i32>,
    pub embedding_batch_size: Option<i32>,
}

impl RAGSimpleVectorIndexingSettings {
    pub fn chunk_token_size(&self) -> usize {
        self.chunk_token_size.unwrap_or(1200) as usize // CHUNK_SIZE
    }

    pub fn chunk_overlap_token_size(&self) -> usize {
        self.chunk_overlap_token_size.unwrap_or(100) as usize // CHUNK_OVERLAP_SIZE
    }

    pub fn max_parallel_insert(&self) -> usize {
        self.max_parallel_insert.unwrap_or(10) as usize // DEFAULT_MAX_PARALLEL_INSERT
    }

    pub fn embedding_batch_size(&self) -> usize {
        self.embedding_batch_size.unwrap_or(32) as usize // DEFAULT_EMBEDDING_BATCH_SIZE
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleVectorQueryingSettings {
    pub top_k: Option<i32>,
    pub chunk_top_k: Option<i32>,
    pub similarity_threshold: Option<f32>,
    pub related_chunk_number: Option<i32>,
    pub max_total_tokens: Option<i32>,
    pub chunk_selection_method: Option<RAGChunkSelectionMethod>,
    pub user_prompt: Option<String>,
    pub enable_rerank: Option<bool>,
    pub min_rerank_score: Option<f32>,
}

impl RAGSimpleVectorQueryingSettings {
    pub fn top_k(&self) -> usize {
        self.top_k.unwrap_or(40) as usize // DEFAULT_TOP_K
    }

    pub fn chunk_top_k(&self) -> usize {
        self.chunk_top_k.unwrap_or(20) as usize // DEFAULT_CHUNK_TOP_K
    }

    pub fn similarity_threshold(&self) -> f32 {
        self.similarity_threshold.unwrap_or(0.2) // DEFAULT_COSINE_THRESHOLD
    }

    pub fn related_chunk_number(&self) -> usize {
        self.related_chunk_number.unwrap_or(5) as usize // DEFAULT_RELATED_CHUNK_NUMBER
    }

    pub fn max_total_tokens(&self) -> usize {
        self.max_total_tokens.unwrap_or(30000) as usize // DEFAULT_MAX_TOTAL_TOKENS
    }

    pub fn chunk_selection_method(&self) -> RAGChunkSelectionMethod {
        self.chunk_selection_method
            .clone()
            .unwrap_or(RAGChunkSelectionMethod::Vector)
    }

    pub fn enable_rerank(&self) -> bool {
        self.enable_rerank.unwrap_or(false) // ENABLE_RERANK env defaults to false
    }

    pub fn min_rerank_score(&self) -> f32 {
        self.min_rerank_score.unwrap_or(0.0) // DEFAULT_MIN_RERANK_SCORE
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleVectorEngineSettings {
    pub indexing: Option<RAGSimpleVectorIndexingSettings>,
    pub querying: Option<RAGSimpleVectorQueryingSettings>,
}

impl RAGSimpleVectorEngineSettings {
    pub fn indexing(&self) -> RAGSimpleVectorIndexingSettings {
        self.indexing
            .clone()
            .unwrap_or(RAGSimpleVectorIndexingSettings {
                chunk_token_size: None,
                chunk_overlap_token_size: None,
                max_parallel_insert: None,
                embedding_batch_size: None,
            })
    }

    pub fn querying(&self) -> RAGSimpleVectorQueryingSettings {
        self.querying
            .clone()
            .unwrap_or(RAGSimpleVectorQueryingSettings {
                top_k: None,
                chunk_top_k: None,
                similarity_threshold: None,
                related_chunk_number: None,
                max_total_tokens: None,
                chunk_selection_method: None,
                user_prompt: None,
                enable_rerank: None,
                min_rerank_score: None,
            })
    }
}

// Graph Engine Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleGraphIndexingSettings {
    pub chunk_token_size: Option<i32>,
    pub chunk_overlap_token_size: Option<i32>,
    pub entity_extract_max_gleaning: Option<i32>,
    pub force_llm_summary_on_merge: Option<i32>,
    pub max_graph_nodes: Option<i32>,
    pub summary_max_tokens: Option<i32>,
    pub entity_types: Option<Vec<String>>,
    pub extraction_language: Option<String>,
}

impl RAGSimpleGraphIndexingSettings {
    pub fn chunk_token_size(&self) -> usize {
        self.chunk_token_size.unwrap_or(1200) as usize // CHUNK_SIZE
    }

    pub fn chunk_overlap_token_size(&self) -> usize {
        self.chunk_overlap_token_size.unwrap_or(100) as usize // CHUNK_OVERLAP_SIZE
    }

    pub fn entity_extract_max_gleaning(&self) -> usize {
        self.entity_extract_max_gleaning.unwrap_or(1) as usize // DEFAULT_MAX_GLEANING
    }

    pub fn force_llm_summary_on_merge(&self) -> usize {
        self.force_llm_summary_on_merge.unwrap_or(4) as usize // DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE
    }

    pub fn max_graph_nodes(&self) -> usize {
        self.max_graph_nodes.unwrap_or(1000) as usize // DEFAULT_MAX_GRAPH_NODES
    }

    pub fn summary_max_tokens(&self) -> usize {
        self.summary_max_tokens.unwrap_or(30000) as usize // DEFAULT_SUMMARY_MAX_TOKENS
    }

    pub fn entity_types(&self) -> Vec<String> {
        self.entity_types.clone().unwrap_or_else(|| {
            vec![
                "organization".to_string(),
                "person".to_string(),
                "geo".to_string(),
                "event".to_string(),
                "category".to_string(),
            ]
        }) // DEFAULT_ENTITY_TYPES
    }

    pub fn extraction_language(&self) -> String {
        self.extraction_language
            .clone()
            .unwrap_or_else(|| "English".to_string()) // DEFAULT_SUMMARY_LANGUAGE
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleGraphQueryingSettings {
    pub max_entity_tokens: Option<i32>,
    pub max_relation_tokens: Option<i32>,
    pub max_total_tokens: Option<i32>,
    pub max_graph_nodes_per_query: Option<i32>,
    pub top_k: Option<i32>,
    pub chunk_top_k: Option<i32>,
    pub related_chunk_number: Option<i32>,
    pub query_mode: Option<RAGSimpleGraphQueryMode>,
    pub chunk_selection_method: Option<RAGChunkSelectionMethod>,
    pub user_prompt: Option<String>,
    pub enable_rerank: Option<bool>,
    pub min_rerank_score: Option<f32>,
}

impl RAGSimpleGraphQueryingSettings {
    pub fn max_entity_tokens(&self) -> usize {
        self.max_entity_tokens.unwrap_or(6000) as usize // DEFAULT_MAX_ENTITY_TOKENS
    }

    pub fn max_relation_tokens(&self) -> usize {
        self.max_relation_tokens.unwrap_or(8000) as usize // DEFAULT_MAX_RELATION_TOKENS
    }

    pub fn max_total_tokens(&self) -> usize {
        self.max_total_tokens.unwrap_or(30000) as usize // DEFAULT_MAX_TOTAL_TOKENS
    }

    pub fn max_graph_nodes_per_query(&self) -> usize {
        self.max_graph_nodes_per_query.unwrap_or(1000) as usize // DEFAULT_MAX_GRAPH_NODES
    }

    pub fn top_k(&self) -> usize {
        self.top_k.unwrap_or(40) as usize // DEFAULT_TOP_K
    }

    pub fn chunk_top_k(&self) -> usize {
        self.chunk_top_k.unwrap_or(20) as usize // DEFAULT_CHUNK_TOP_K
    }

    pub fn related_chunk_number(&self) -> usize {
        self.related_chunk_number.unwrap_or(5) as usize // DEFAULT_RELATED_CHUNK_NUMBER
    }

    pub fn query_mode(&self) -> RAGSimpleGraphQueryMode {
        self.query_mode
            .clone()
            .unwrap_or(RAGSimpleGraphQueryMode::Mix)
    }

    pub fn chunk_selection_method(&self) -> RAGChunkSelectionMethod {
        self.chunk_selection_method
            .clone()
            .unwrap_or(RAGChunkSelectionMethod::Vector)
    }

    pub fn enable_rerank(&self) -> bool {
        self.enable_rerank.unwrap_or(false) // ENABLE_RERANK env defaults to false
    }

    pub fn min_rerank_score(&self) -> f32 {
        self.min_rerank_score.unwrap_or(0.0) // DEFAULT_MIN_RERANK_SCORE
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RAGSimpleGraphEngineSettings {
    pub indexing: Option<RAGSimpleGraphIndexingSettings>,
    pub querying: Option<RAGSimpleGraphQueryingSettings>,
}

impl RAGSimpleGraphEngineSettings {
    pub fn indexing(&self) -> RAGSimpleGraphIndexingSettings {
        self.indexing
            .clone()
            .unwrap_or(RAGSimpleGraphIndexingSettings {
                chunk_token_size: None,
                chunk_overlap_token_size: None,
                entity_extract_max_gleaning: None,
                force_llm_summary_on_merge: None,
                max_graph_nodes: None,
                summary_max_tokens: None,
                entity_types: None,
                extraction_language: None,
            })
    }

    pub fn querying(&self) -> RAGSimpleGraphQueryingSettings {
        self.querying
            .clone()
            .unwrap_or(RAGSimpleGraphQueryingSettings {
                max_entity_tokens: None,
                max_relation_tokens: None,
                max_total_tokens: None,
                max_graph_nodes_per_query: None,
                top_k: None,
                chunk_top_k: None,
                related_chunk_number: None,
                query_mode: None,
                chunk_selection_method: None,
                user_prompt: None,
                enable_rerank: None,
                min_rerank_score: None,
            })
    }
}
