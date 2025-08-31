use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::proxy::ProxySettings;
use super::user::UserGroup;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum RAGProviderType {
    Local,
    LightRAG,
    RAGStack,
    Chroma,
    Weaviate,
    Pinecone,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RAGProvider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: RAGProviderType,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub built_in: bool,
    pub can_user_create_instance: bool,
    pub proxy_settings: Option<ProxySettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRAGProviderRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub can_user_create_instance: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRAGProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub can_user_create_instance: Option<bool>,
    pub proxy_settings: Option<ProxySettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGProviderListResponse {
    pub providers: Vec<RAGProvider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// User Group RAG Provider Relationship Models
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserGroupRAGProvider {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserGroupRAGProviderResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub provider_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub provider: RAGProvider,
    pub group: UserGroup,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AssignRAGProviderToGroupRequest {
    pub group_id: Uuid,
    pub provider_id: Uuid,
}
