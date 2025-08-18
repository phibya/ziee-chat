use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::proxy::ProxySettings;

// Re-export ProxySettings as RAGProviderProxySettings for compatibility
pub type RAGProviderProxySettings = ProxySettings;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

impl RAGProviderType {
    pub fn from_str(s: &str) -> RAGProviderType {
        match s {
            "local" => RAGProviderType::Local,
            "lightrag" => RAGProviderType::LightRAG,
            "ragstack" => RAGProviderType::RAGStack,
            "chroma" => RAGProviderType::Chroma,
            "weaviate" => RAGProviderType::Weaviate,
            "pinecone" => RAGProviderType::Pinecone,
            "custom" => RAGProviderType::Custom,
            _ => RAGProviderType::Custom, // fallback to custom for unknown types
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RAGProviderType::Local => "local",
            RAGProviderType::LightRAG => "lightrag",
            RAGProviderType::RAGStack => "ragstack",
            RAGProviderType::Chroma => "chroma",
            RAGProviderType::Weaviate => "weaviate",
            RAGProviderType::Pinecone => "pinecone",
            RAGProviderType::Custom => "custom",
        }
    }
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
    pub proxy_settings: Option<ProxySettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for RAGProvider {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let proxy_settings_json: serde_json::Value = row.try_get("proxy_settings")?;
        let proxy_settings = if proxy_settings_json.is_null() {
            None
        } else {
            Some(serde_json::from_value(proxy_settings_json).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "proxy_settings".into(),
                    source: Box::new(e),
                }
            })?)
        };

        let provider_type_str: String = row.try_get("provider_type")?;
        let provider_type = RAGProviderType::from_str(&provider_type_str);

        Ok(RAGProvider {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            provider_type,
            enabled: row.try_get("enabled")?,
            api_key: row.try_get("api_key")?,
            base_url: row.try_get("base_url")?,
            built_in: row.try_get("built_in")?,
            proxy_settings,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRAGProviderRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRAGProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub proxy_settings: Option<ProxySettings>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RAGProviderListResponse {
    pub providers: Vec<RAGProvider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
