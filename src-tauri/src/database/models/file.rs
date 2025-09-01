use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::macros::impl_json_option_from;

// Implement JSON conversion for Vec<File>
impl_json_option_from!(Vec<File>);

// Main File structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub project_id: Option<Uuid>,
    pub thumbnail_count: i32,
    pub page_count: i32,
    pub processing_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFile {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_id: Uuid,
    pub file: Option<File>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFile {
    pub id: Uuid,
    pub file_id: Uuid,
    pub provider_id: Uuid,
    pub provider_file_id: Option<String>,
    pub provider_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileRequest {
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadFileResponse {
    pub file: File,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileListResponse {
    pub files: Vec<File>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileListParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
}

// Processing structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub text_content: Option<String>,
    pub metadata: serde_json::Value,
    pub thumbnail_count: i32,
    pub page_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreateData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub project_id: Option<Uuid>,
    pub thumbnail_count: i32,
    pub page_count: i32,
    pub processing_metadata: serde_json::Value,
}
