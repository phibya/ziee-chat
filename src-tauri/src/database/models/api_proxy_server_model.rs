use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelServerEntry {
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiProxyServerModel {
    pub id: Uuid,
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiProxyServerTrustedHost {
    pub id: Uuid,
    pub host: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateApiProxyServerModelRequest {
    pub model_id: Uuid,
    pub alias_id: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateApiProxyServerModelRequest {
    pub alias_id: Option<String>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateTrustedHostRequest {
    pub host: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTrustedHostRequest {
    pub host: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiProxyServerConfig {
    pub port: u16,
    pub address: String,
    pub prefix: String,
    pub api_key: String,
    pub allow_cors: bool,
    pub log_level: String,
    pub autostart_on_startup: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ApiProxyServerStatus {
    pub running: bool,
    pub active_models: i32,
    pub server_url: Option<String>,
}
