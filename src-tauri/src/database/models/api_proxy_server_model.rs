use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelServerEntry {
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProxyServerModel {
    pub id: Uuid,
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for ApiProxyServerModel {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ApiProxyServerModel {
            id: row.try_get("id")?,
            model_id: row.try_get("model_id")?,
            alias_id: row.try_get("alias_id")?,
            enabled: row.try_get("enabled")?,
            is_default: row.try_get("is_default")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProxyServerTrustedHost {
    pub id: Uuid,
    pub host: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for ApiProxyServerTrustedHost {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ApiProxyServerTrustedHost {
            id: row.try_get("id")?,
            host: row.try_get("host")?,
            description: row.try_get("description")?,
            enabled: row.try_get("enabled")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiProxyServerModelRequest {
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApiProxyServerModelRequest {
    pub alias_id: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTrustedHostRequest {
    pub host: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTrustedHostRequest {
    pub host: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProxyServerConfig {
    pub port: u16,
    pub address: String,
    pub prefix: String,
    pub api_key: String,
    pub allow_cors: bool,
    pub log_level: String,
    pub autostart_on_startup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiProxyServerStatus {
    pub running: bool,
    pub active_models: usize,
    pub server_url: Option<String>,
}