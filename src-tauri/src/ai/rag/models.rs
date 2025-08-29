// Database models for RAG functionality

use crate::database::models::RagEngineSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
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

impl FromRow<'_, sqlx::postgres::PgRow> for RagProvider {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(RagProvider {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            provider_type: row.try_get("provider_type")?,
            enabled: row.try_get("enabled")?,
            api_key: row.try_get("api_key")?,
            base_url: row.try_get("base_url")?,
            built_in: row.try_get("built_in")?,
            proxy_settings: row.try_get("proxy_settings")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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
    pub engine_settings: RagEngineSettings,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub age_graph_name: Option<String>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RagInstance {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(RagInstance {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            user_id: row.try_get("user_id")?,
            project_id: row.try_get("project_id")?,
            name: row.try_get("name")?,
            alias: row.try_get("alias")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            is_active: row.try_get("is_active")?,
            engine_type: row.try_get("engine_type")?,
            engine_settings: row
                .try_get::<serde_json::Value, _>("engine_settings")
                .map(|v| serde_json::from_value(v).unwrap_or_default())
                .unwrap_or_default(),
            embedding_model_id: row.try_get("embedding_model_id")?,
            llm_model_id: row.try_get("llm_model_id")?,
            age_graph_name: row.try_get("age_graph_name")?,
            parameters: row.try_get("parameters")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for RagInstanceFile {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(RagInstanceFile {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            file_id: row.try_get("file_id")?,
            processing_status: row.try_get("processing_status")?,
            processed_at: row.try_get("processed_at")?,
            processing_error: row.try_get("processing_error")?,
            rag_metadata: row.try_get("rag_metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for SimpleVectorDocument {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Handle HALFVEC/VECTOR type conversion - pgvector automatically handles both
        let embedding: Option<Vec<f32>> = row.try_get::<Option<Vec<f32>>, _>("embedding")?;

        Ok(SimpleVectorDocument {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            file_id: row.try_get("file_id")?,
            chunk_index: row.try_get("chunk_index")?,
            content: row.try_get("content")?,
            content_hash: row.try_get("content_hash")?,
            token_count: row.try_get("token_count")?,
            embedding,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for SimpleGraphEntity {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SimpleGraphEntity {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            name: row.try_get("name")?,
            entity_type: row.try_get("entity_type")?,
            description: row.try_get("description")?,
            importance_score: row.try_get("importance_score")?,
            extraction_metadata: row.try_get("extraction_metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for SimpleGraphRelationship {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SimpleGraphRelationship {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            source_entity_id: row.try_get("source_entity_id")?,
            target_entity_id: row.try_get("target_entity_id")?,
            relationship_type: row.try_get("relationship_type")?,
            description: row.try_get("description")?,
            weight: row.try_get("weight")?,
            extraction_metadata: row.try_get("extraction_metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for SimpleGraphChunk {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SimpleGraphChunk {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            file_id: row.try_get("file_id")?,
            chunk_index: row.try_get("chunk_index")?,
            content: row.try_get("content")?,
            content_hash: row.try_get("content_hash")?,
            token_count: row.try_get("token_count")?,
            entities: row.try_get("entities")?,
            relationships: row.try_get("relationships")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for SimpleGraphCommunity {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(SimpleGraphCommunity {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            community_id: row.try_get("community_id")?,
            entity_ids: row.try_get("entity_ids")?,
            summary: row.try_get("summary")?,
            importance_score: row.try_get("importance_score")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// RAG processing pipeline model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagProcessingPipeline {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub pipeline_stage: String,
    pub status: String,
    pub progress_percentage: i32,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RagProcessingPipeline {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(RagProcessingPipeline {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            file_id: row.try_get("file_id")?,
            pipeline_stage: row.try_get("pipeline_stage")?,
            status: row.try_get("status")?,
            progress_percentage: row.try_get("progress_percentage")?,
            error_message: row.try_get("error_message")?,
            metadata: row.try_get("metadata")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for AgeGraph {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(AgeGraph {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            graph_name: row.try_get("graph_name")?,
            status: row.try_get("status")?,
            node_count: row.try_get("node_count")?,
            edge_count: row.try_get("edge_count")?,
            last_updated: row.try_get("last_updated")?,
            metadata: row.try_get("metadata")?,
            created_at: row.try_get("created_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for AgeQueryCache {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(AgeQueryCache {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            query_hash: row.try_get("query_hash")?,
            query_type: row.try_get("query_type")?,
            query_params: row.try_get("query_params")?,
            result_data: row.try_get("result_data")?,
            hit_count: row.try_get("hit_count")?,
            last_accessed: row.try_get("last_accessed")?,
            expires_at: row.try_get("expires_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
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
