use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{model::DeviceType, proxy::ProxySettings};
use crate::database::macros::impl_string_to_enum;
use crate::database::types::JsonOption;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
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
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(ProviderType::Local),
            "openai" => Some(ProviderType::OpenAI),
            "anthropic" => Some(ProviderType::Anthropic),
            "groq" => Some(ProviderType::Groq),
            "gemini" => Some(ProviderType::Gemini),
            "mistral" => Some(ProviderType::Mistral),
            "deepseek" => Some(ProviderType::DeepSeek),
            "huggingface" => Some(ProviderType::Huggingface),
            "custom" => Some(ProviderType::Custom),
            _ => Some(ProviderType::Custom), // fallback to custom for unknown types
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

// Implement string to enum conversion for SQLx
impl_string_to_enum!(ProviderType);

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
    pub proxy_settings: JsonOption<ProxySettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub device_type: DeviceType,
    pub memory_total: Option<u64>, // Total memory in bytes
    pub memory_free: Option<u64>,  // Free memory in bytes
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AvailableDevicesResponse {
    pub devices: Vec<DeviceInfo>,
    pub default_device_type: DeviceType,
    pub supports_multi_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
