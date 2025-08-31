use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
