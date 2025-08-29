use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RAGRepository {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub enabled: bool,
    pub requires_auth: bool,
    pub auth_token: Option<String>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RAGRepository {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(RAGRepository {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            url: row.try_get("url")?,
            enabled: row.try_get("enabled")?,
            requires_auth: row.try_get("requires_auth")?,
            auth_token: row.try_get("auth_token")?,
            priority: row.try_get("priority")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRAGRepositoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub enabled: Option<bool>,
    pub requires_auth: Option<bool>,
    pub auth_token: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRAGRepositoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub enabled: Option<bool>,
    pub requires_auth: Option<bool>,
    pub auth_token: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGRepositoryListResponse {
    pub repositories: Vec<RAGRepository>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGRepositoryConnectionTestResponse {
    pub success: bool,
    pub message: String,
    pub available_databases_count: Option<i32>,
}
