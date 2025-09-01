// Database models for RAG functionality

use crate::database::models::RAGEngineSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RAG provider model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagProvider {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub built_in: bool,
    pub proxy_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RAG instance model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagInstance {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub user_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_active: bool,
    pub engine_type: String,
    pub engine_settings: RAGEngineSettings,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub age_graph_name: Option<String>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
/// RAG instance file model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagInstanceFile {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub processing_status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_error: Option<String>,
    pub rag_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
/// Simple vector document model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleVectorDocument {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub token_count: i32,
    pub embedding: Option<Vec<f32>>, // VECTOR type from database
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simple graph entity model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGraphEntity {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub name: String,
    pub entity_type: Option<String>,
    pub description: Option<String>,
    pub importance_score: f32,
    pub extraction_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simple graph relationship model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGraphRelationship {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub relationship_type: String,
    pub description: Option<String>,
    pub weight: f32,
    pub extraction_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simple graph chunk model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGraphChunk {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub token_count: i32,
    pub entities: serde_json::Value,      // Array of entity names
    pub relationships: serde_json::Value, // Array of relationship descriptions
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simple graph community model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGraphCommunity {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub community_id: i32,
    pub entity_ids: serde_json::Value, // Array of entity UUIDs
    pub summary: Option<String>,
    pub importance_score: f32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RAG processing pipeline model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagProcessingPipeline {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub pipeline_stage: String,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Apache AGE graph model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeGraph {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub graph_name: String,
    pub status: String,
    pub node_count: i64,
    pub edge_count: i64,
    pub last_updated: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Apache AGE query cache model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeQueryCache {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub query_hash: String,
    pub query_type: String,
    pub query_params: serde_json::Value,
    pub result_data: serde_json::Value,
    pub hit_count: i32,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Helper structures for API responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagInstanceWithFiles {
    #[serde(flatten)]
    pub instance: RagInstance,
    pub files: Vec<RagInstanceFile>,
    pub processing_pipeline: Vec<RagProcessingPipeline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityWithRelationships {
    #[serde(flatten)]
    pub entity: SimpleGraphEntity,
    pub relationships: Vec<SimpleGraphRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkWithEmbedding {
    #[serde(flatten)]
    pub chunk: SimpleVectorDocument,
    pub similarity_score: Option<f32>,
}

/// Database query builders for complex operations

impl RagInstance {
    pub fn get_engine_settings(&self) -> Result<serde_json::Value, serde_json::Error> {
        match self.engine_type.as_str() {
            "simple_vector" => {
                if let Some(ref settings) = self.engine_settings.simple_vector {
                    serde_json::to_value(settings)
                } else {
                    Ok(serde_json::json!({}))
                }
            }
            "simple_graph" => {
                if let Some(ref settings) = self.engine_settings.simple_graph {
                    serde_json::to_value(settings)
                } else {
                    Ok(serde_json::json!({}))
                }
            }
            _ => Ok(serde_json::json!({})),
        }
    }

    pub fn set_engine_settings(
        &mut self,
        settings: serde_json::Value,
    ) -> Result<(), serde_json::Error> {
        match self.engine_type.as_str() {
            "simple_vector" => {
                if settings.is_null() || settings == serde_json::json!({}) {
                    self.engine_settings.simple_vector = None;
                } else {
                    self.engine_settings.simple_vector = Some(serde_json::from_value(settings)?);
                }
                Ok(())
            }
            "simple_graph" => {
                if settings.is_null() || settings == serde_json::json!({}) {
                    self.engine_settings.simple_graph = None;
                } else {
                    self.engine_settings.simple_graph = Some(serde_json::from_value(settings)?);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl SimpleVectorDocument {
    // Note: No longer need to format embedding for DB - pgvector handles Vec<f32> directly
    pub fn get_embedding_dimensions(&self) -> Option<usize> {
        self.embedding.as_ref().map(|vec| vec.len())
    }
}

impl SimpleGraphChunk {
    pub fn get_entity_names(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.entities.clone())
    }

    pub fn get_relationship_descriptions(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.relationships.clone())
    }

    pub fn set_entity_names(&mut self, names: Vec<String>) -> Result<(), serde_json::Error> {
        self.entities = serde_json::to_value(names)?;
        Ok(())
    }

    pub fn set_relationship_descriptions(
        &mut self,
        descriptions: Vec<String>,
    ) -> Result<(), serde_json::Error> {
        self.relationships = serde_json::to_value(descriptions)?;
        Ok(())
    }
}

impl SimpleGraphCommunity {
    pub fn get_entity_ids(&self) -> Result<Vec<Uuid>, serde_json::Error> {
        serde_json::from_value(self.entity_ids.clone())
    }

    pub fn set_entity_ids(&mut self, ids: Vec<Uuid>) -> Result<(), serde_json::Error> {
        self.entity_ids = serde_json::to_value(ids)?;
        Ok(())
    }
}
