use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

// Main File structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub project_id: Option<Uuid>,
    pub thumbnail_count: i32,
    pub processing_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for File {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(File {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            filename: row.try_get("filename")?,
            file_size: row.try_get("file_size")?,
            mime_type: row.try_get("mime_type")?,
            checksum: row.try_get("checksum")?,
            project_id: row.try_get("project_id")?,
            thumbnail_count: row.try_get("thumbnail_count")?,
            processing_metadata: row.try_get("processing_metadata")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFile {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_id: Uuid,
    pub file: Option<File>,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for MessageFile {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(MessageFile {
            id: row.try_get("id")?,
            message_id: row.try_get("message_id")?,
            file_id: row.try_get("file_id")?,
            file: None, // This is loaded separately via joins when needed
            created_at: row.try_get("created_at")?,
        })
    }
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

impl FromRow<'_, sqlx::postgres::PgRow> for ProviderFile {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ProviderFile {
            id: row.try_get("id")?,
            file_id: row.try_get("file_id")?,
            provider_id: row.try_get("provider_id")?,
            provider_file_id: row.try_get("provider_file_id")?,
            provider_metadata: row.try_get("provider_metadata")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileRequest {
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileResponse {
    pub file: File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<File>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
}

// Processing structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub text_content: Option<String>,
    pub base64_content: Option<String>,
    pub metadata: serde_json::Value,
    pub thumbnail_count: i32,
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
    pub processing_metadata: serde_json::Value,
}

