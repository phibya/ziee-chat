use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

// API structures for model providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderProxySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub no_proxy: String,
    #[serde(default)]
    pub ignore_ssl_certificates: bool,
    #[serde(default)]
    pub proxy_ssl: bool,
    #[serde(default)]
    pub proxy_host_ssl: bool,
    #[serde(default)]
    pub peer_ssl: bool,
    #[serde(default)]
    pub host_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub is_default: bool,
    pub proxy_settings: Option<ProviderProxySettings>,
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

        Ok(Provider {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            provider_type: row.try_get("provider_type")?,
            enabled: row.try_get("enabled")?,
            api_key: row.try_get("api_key")?,
            base_url: row.try_get("base_url")?,
            is_default: row.try_get("is_default")?,
            proxy_settings,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProviderRequest {
    pub name: String,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub proxy_settings: Option<ProviderProxySettings>,
    #[serde(rename = "type")]
    pub provider_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub proxy_settings: Option<ProviderProxySettings>,
}

// Device detection structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: i32, // Device index (0, 1, 2, etc.)
    pub name: String,
    pub device_type: String,       // cpu, cuda, metal
    pub memory_total: Option<u64>, // Total memory in bytes
    pub memory_free: Option<u64>,  // Free memory in bytes
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableDevicesResponse {
    pub devices: Vec<DeviceInfo>,
    pub default_device_type: String,
    pub supports_multi_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProviderProxyResponse {
    pub success: bool,
    pub message: String,
}
