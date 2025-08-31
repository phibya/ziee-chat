use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct RepositoryAuthConfig {
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub auth_test_api_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub auth_type: String, // none, api_key, basic_auth, bearer_token
    pub auth_config: Option<RepositoryAuthConfig>,
    pub enabled: bool,
    pub built_in: bool, // true for built-in repositories like Hugging Face
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub url: String,
    pub auth_type: String,
    pub auth_config: Option<RepositoryAuthConfig>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub auth_type: Option<String>,
    pub auth_config: Option<RepositoryAuthConfig>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryListResponse {
    pub repositories: Vec<Repository>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestRepositoryConnectionRequest {
    pub name: String,
    pub url: String,
    pub auth_type: String,
    pub auth_config: Option<RepositoryAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestRepositoryConnectionResponse {
    pub success: bool,
    pub message: String,
}
