use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::proxy::ProxySettings;

// Re-export ProxySettings as ProviderProxySettings for backward compatibility
pub type ProviderProxySettings = ProxySettings;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Local,
    OpenAI,
    Anthropic,
    Groq,
    Gemini,
    Mistral,
    DeepSeek,
    Huggingface,
    Custom,
}

impl ProviderType {
    pub fn from_str(s: &str) -> ProviderType {
        match s {
            "local" => ProviderType::Local,
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "groq" => ProviderType::Groq,
            "gemini" => ProviderType::Gemini,
            "mistral" => ProviderType::Mistral,
            "deepseek" => ProviderType::DeepSeek,
            "huggingface" => ProviderType::Huggingface,
            "custom" => ProviderType::Custom,
            _ => ProviderType::Custom, // fallback to custom for unknown types
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Local => "local",
            ProviderType::OpenAI => "openai",
            ProviderType::Anthropic => "anthropic",
            ProviderType::Groq => "groq",
            ProviderType::Gemini => "gemini",
            ProviderType::Mistral => "mistral",
            ProviderType::DeepSeek => "deepseek",
            ProviderType::Huggingface => "huggingface",
            ProviderType::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub built_in: bool,
    pub proxy_settings: Option<ProxySettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Provider {
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
        let provider_type = ProviderType::from_str(&provider_type_str);

        Ok(Provider {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProviderRequest {
    pub name: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub proxy_settings: Option<ProxySettings>,
    #[serde(rename = "type")]
    pub provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub proxy_settings: Option<ProxySettings>,
}

// Device detection structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeviceInfo {
    pub id: i32, // Device index (0, 1, 2, etc.)
    pub name: String,
    pub device_type: String,       // cpu, cuda, metal
    pub memory_total: Option<u64>, // Total memory in bytes
    pub memory_free: Option<u64>,  // Free memory in bytes
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AvailableDevicesResponse {
    pub devices: Vec<DeviceInfo>,
    pub default_device_type: String,
    pub supports_multi_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
