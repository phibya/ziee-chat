use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;
use crate::ai::rag::engines::settings::*;

/// Engine-specific settings for RAG instance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct RagEngineSettings {
    /// Simple vector RAG engine settings
    pub simple_vector: Option<RAGSimpleVectorEngineSettings>,
    /// Simple graph RAG engine settings  
    pub simple_graph: Option<RAGSimpleGraphEngineSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstance {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub user_id: Option<Uuid>, // null for system instances
    pub project_id: Option<Uuid>,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_active: bool,
    pub is_system: bool,
    pub engine_type: RAGEngineType,
    pub engine_settings: RagEngineSettings,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub age_graph_name: Option<String>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RAGInstance {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let engine_type_str: String = row.try_get("engine_type")?;
        let engine_type = RAGEngineType::from_str(&engine_type_str);

        Ok(RAGInstance {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            user_id: row.try_get("user_id")?,
            project_id: row.try_get("project_id")?,
            name: row.try_get("name")?,
            alias: row.try_get("alias")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            is_active: row.try_get("is_active")?,
            is_system: row.try_get("is_system").unwrap_or(false),
            engine_type,
            engine_settings: row.try_get::<serde_json::Value, _>("engine_settings")
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RAGEngineType {
    #[serde(rename = "simple_vector")]
    RagSimpleVector,
    #[serde(rename = "simple_graph")]
    RagSimpleGraph,
}

impl RAGEngineType {
    pub fn from_str(s: &str) -> RAGEngineType {
        match s {
            "simple_vector" => RAGEngineType::RagSimpleVector,
            "simple_graph" => RAGEngineType::RagSimpleGraph,
            _ => RAGEngineType::RagSimpleVector, // fallback to vector for unknown types
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RAGEngineType::RagSimpleVector => "simple_vector",
            RAGEngineType::RagSimpleGraph => "simple_graph",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRAGInstanceRequest {
    pub provider_id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub engine_type: RAGEngineType,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub parameters: Option<serde_json::Value>,
    pub engine_settings: Option<RagEngineSettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSystemRAGInstanceRequest {
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub engine_type: RAGEngineType,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub parameters: Option<serde_json::Value>,
    pub engine_settings: Option<RagEngineSettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRAGInstanceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub parameters: Option<serde_json::Value>,
    pub engine_settings: Option<RagEngineSettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceListResponse {
    pub instances: Vec<RAGInstance>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// RAG Instance File Models
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceFile {
    pub id: Uuid,
    pub rag_instance_id: Uuid,
    pub file_id: Uuid,
    pub filename: String,
    pub processing_status: RAGProcessingStatus,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_error: Option<String>,
    pub rag_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RAGInstanceFile {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let status_str: String = row.try_get("processing_status")?;
        let processing_status = RAGProcessingStatus::from_str(&status_str);

        Ok(RAGInstanceFile {
            id: row.try_get("id")?,
            rag_instance_id: row.try_get("rag_instance_id")?,
            file_id: row.try_get("file_id")?,
            filename: row.try_get("filename")?,
            processing_status,
            processed_at: row.try_get("processed_at")?,
            processing_error: row.try_get("processing_error")?,
            rag_metadata: row.try_get("rag_metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RAGProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl RAGProcessingStatus {
    pub fn from_str(s: &str) -> RAGProcessingStatus {
        match s {
            "pending" => RAGProcessingStatus::Pending,
            "processing" => RAGProcessingStatus::Processing,
            "completed" => RAGProcessingStatus::Completed,
            "failed" => RAGProcessingStatus::Failed,
            _ => RAGProcessingStatus::Pending, // fallback to pending for unknown types
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RAGProcessingStatus::Pending => "pending",
            RAGProcessingStatus::Processing => "processing",
            RAGProcessingStatus::Completed => "completed",
            RAGProcessingStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddFilesToRAGInstanceRequest {
    pub file_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddFilesToRAGInstanceResponse {
    pub added_files: Vec<RAGInstanceFile>,
    pub errors: Vec<RAGFileError>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGFileError {
    pub file_id: Uuid,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceFilesQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status_filter: Option<RAGProcessingStatus>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGInstanceFilesListResponse {
    pub files: Vec<RAGInstanceFile>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}