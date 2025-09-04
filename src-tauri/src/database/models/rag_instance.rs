use crate::ai::rag::engines::settings::*;
use crate::database::macros::{impl_json_from, impl_string_to_enum};
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

// Implement JSON conversion for RAGEngineSettings using macro
impl_json_from!(RAGEngineSettings);

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
    pub status: RAGInstanceStatus,
    pub error_code: RAGInstanceErrorCode,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum RAGInstanceStatus {
    None,
    Indexing,
    Finished,
    Error,
}

impl RAGInstanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGInstanceStatus::None => "none",
            RAGInstanceStatus::Indexing => "indexing",
            RAGInstanceStatus::Finished => "finished",
            RAGInstanceStatus::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(RAGInstanceStatus::None),
            "indexing" => Some(RAGInstanceStatus::Indexing),
            "finished" => Some(RAGInstanceStatus::Finished),
            "error" => Some(RAGInstanceStatus::Error),
            _ => None,
        }
    }
}

// Implement string to enum conversion for RAGInstanceStatus
impl_string_to_enum!(RAGInstanceStatus);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum RAGInstanceErrorCode {
    None,
    EmbeddingModelNotConfig,
    EmbeddingModelNotFound,
    LlmModelNotConfig,
    LlmModelNotFound,
    ProviderConnectionFailed,
    ProviderNotFound,
    RagInstanceNotFound,
    IndexingFailed,
    FileProcessingFailed,
    DatabaseError,
    ConfigurationError,
}

impl RAGInstanceErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RAGInstanceErrorCode::None => "none",
            RAGInstanceErrorCode::EmbeddingModelNotConfig => "embedding_model_not_config",
            RAGInstanceErrorCode::EmbeddingModelNotFound => "embedding_model_not_found",
            RAGInstanceErrorCode::LlmModelNotConfig => "llm_model_not_config",
            RAGInstanceErrorCode::LlmModelNotFound => "llm_model_not_found",
            RAGInstanceErrorCode::ProviderConnectionFailed => "provider_connection_failed",
            RAGInstanceErrorCode::ProviderNotFound => "provider_not_found",
            RAGInstanceErrorCode::RagInstanceNotFound => "rag_instance_not_found",
            RAGInstanceErrorCode::IndexingFailed => "indexing_failed",
            RAGInstanceErrorCode::FileProcessingFailed => "file_processing_failed",
            RAGInstanceErrorCode::DatabaseError => "database_error",
            RAGInstanceErrorCode::ConfigurationError => "configuration_error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(RAGInstanceErrorCode::None),
            "embedding_model_not_config" => Some(RAGInstanceErrorCode::EmbeddingModelNotConfig),
            "embedding_model_not_found" => Some(RAGInstanceErrorCode::EmbeddingModelNotFound),
            "llm_model_not_config" => Some(RAGInstanceErrorCode::LlmModelNotConfig),
            "llm_model_not_found" => Some(RAGInstanceErrorCode::LlmModelNotFound),
            "provider_connection_failed" => Some(RAGInstanceErrorCode::ProviderConnectionFailed),
            "provider_not_found" => Some(RAGInstanceErrorCode::ProviderNotFound),
            "rag_instance_not_found" => Some(RAGInstanceErrorCode::RagInstanceNotFound),
            "indexing_failed" => Some(RAGInstanceErrorCode::IndexingFailed),
            "file_processing_failed" => Some(RAGInstanceErrorCode::FileProcessingFailed),
            "database_error" => Some(RAGInstanceErrorCode::DatabaseError),
            "configuration_error" => Some(RAGInstanceErrorCode::ConfigurationError),
            _ => None,
        }
    }
}

// Implement string to enum conversion for RAGInstanceErrorCode
impl_string_to_enum!(RAGInstanceErrorCode);

// Implement Display for RAGInstanceErrorCode
impl std::fmt::Display for RAGInstanceErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
    pub is_active: Option<bool>,
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
