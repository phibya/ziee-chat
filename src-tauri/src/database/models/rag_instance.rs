use crate::ai::rag::engines::settings::*;
use crate::database::macros::impl_string_to_enum;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Engine-specific settings for RAG instance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, sqlx::Decode)]
pub struct RAGEngineSettings {
    /// Simple vector RAG engine settings
    pub simple_vector: Option<RAGSimpleVectorEngineSettings>,
    /// Simple graph RAG engine settings  
    pub simple_graph: Option<RAGSimpleGraphEngineSettings>,
}

// Implement JSON conversion for RAGEngineSettings
impl From<serde_json::Value> for RAGEngineSettings {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }
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
    pub engine_settings: RAGEngineSettings,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub age_graph_name: Option<String>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum RAGEngineType {
    #[serde(rename = "simple_vector")]
    RagSimpleVector,
    #[serde(rename = "simple_graph")]
    RagSimpleGraph,
}

impl RAGEngineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGEngineType::RagSimpleVector => "simple_vector",
            RAGEngineType::RagSimpleGraph => "simple_graph",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "simple_vector" => Some(RAGEngineType::RagSimpleVector),
            "simple_graph" => Some(RAGEngineType::RagSimpleGraph),
            _ => None,
        }
    }
}

// Implement string to enum conversion for RAGEngineType
impl_string_to_enum!(RAGEngineType);

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
    pub engine_settings: Option<RAGEngineSettings>,
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
    pub engine_settings: Option<RAGEngineSettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRAGInstanceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub embedding_model_id: Option<Uuid>,
    pub llm_model_id: Option<Uuid>,
    pub parameters: Option<serde_json::Value>,
    pub engine_settings: Option<RAGEngineSettings>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum RAGProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl RAGProcessingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGProcessingStatus::Pending => "pending",
            RAGProcessingStatus::Processing => "processing",
            RAGProcessingStatus::Completed => "completed",
            RAGProcessingStatus::Failed => "failed",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(RAGProcessingStatus::Pending),
            "processing" => Some(RAGProcessingStatus::Processing),
            "completed" => Some(RAGProcessingStatus::Completed),
            "failed" => Some(RAGProcessingStatus::Failed),
            _ => None,
        }
    }
}

// Implement string to enum conversion for RAGProcessingStatus
impl_string_to_enum!(RAGProcessingStatus);

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
