use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGDatabaseCapabilities {
    pub semantic_search: Option<bool>,
    pub hybrid_search: Option<bool>,
    pub metadata_filtering: Option<bool>,
    pub similarity_threshold: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGDatabase {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_active: bool,
    pub collection_name: Option<String>,
    pub embedding_model: Option<String>,
    pub chunk_size: i32,
    pub chunk_overlap: i32,
    pub capabilities: Option<RAGDatabaseCapabilities>,
    pub settings: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RAGDatabase {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let capabilities_json: serde_json::Value = row.try_get("capabilities")?;
        let capabilities = if capabilities_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(capabilities_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "capabilities".into(),
                    source: Box::new(e),
                }
            })?)
        };

        let settings_json: serde_json::Value = row.try_get("settings")?;
        let settings = if settings_json.is_null() {
            None
        } else {
            Some(settings_json)
        };

        Ok(RAGDatabase {
            id: row.try_get("id")?,
            provider_id: row.try_get("provider_id")?,
            name: row.try_get("name")?,
            alias: row.try_get("alias")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            is_active: row.try_get("is_active")?,
            collection_name: row.try_get("collection_name")?,
            embedding_model: row.try_get("embedding_model")?,
            chunk_size: row.try_get("chunk_size")?,
            chunk_overlap: row.try_get("chunk_overlap")?,
            capabilities,
            settings,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRAGDatabaseRequest {
    pub name: String,
    pub alias: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub collection_name: Option<String>,
    pub embedding_model: Option<String>,
    pub chunk_size: Option<i32>,
    pub chunk_overlap: Option<i32>,
    pub capabilities: Option<RAGDatabaseCapabilities>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRAGDatabaseRequest {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub collection_name: Option<String>,
    pub embedding_model: Option<String>,
    pub chunk_size: Option<i32>,
    pub chunk_overlap: Option<i32>,
    pub capabilities: Option<RAGDatabaseCapabilities>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RAGDatabaseListResponse {
    pub databases: Vec<RAGDatabase>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}