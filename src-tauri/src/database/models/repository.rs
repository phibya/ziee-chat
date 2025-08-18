use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryAuthConfig {
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub auth_test_api_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl FromRow<'_, sqlx::postgres::PgRow> for Repository {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let auth_config_json: serde_json::Value = row.try_get("auth_config")?;
        let auth_config = if auth_config_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(auth_config_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "auth_config".into(),
                    source: Box::new(e),
                }
            })?)
        };

        Ok(Repository {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            url: row.try_get("url")?,
            auth_type: row.try_get("auth_type")?,
            auth_config,
            enabled: row.try_get("enabled")?,
            built_in: row.try_get("built_in")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRepositoryConnectionResponse {
    pub success: bool,
    pub message: String,
}
