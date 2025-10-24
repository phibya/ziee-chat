#![allow(dead_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum AuthProviderType {
    Local,
    Ldap,
    OAuth2,
    Oidc,
    Saml,
}

impl AsRef<str> for AuthProviderType {
    fn as_ref(&self) -> &str {
        match self {
            AuthProviderType::Local => "local",
            AuthProviderType::Ldap => "ldap",
            AuthProviderType::OAuth2 => "oauth2",
            AuthProviderType::Oidc => "oidc",
            AuthProviderType::Saml => "saml",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthProvider {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub priority: i32,
    pub config: serde_json::Value,
    pub mapping_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuthProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub priority: i32,
    pub config: serde_json::Value,
    pub mapping_rules: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateAuthProviderRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub config: Option<serde_json::Value>,
    pub mapping_rules: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthProviderListResponse {
    pub providers: Vec<AuthProvider>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OAuthLoginResponse {
    pub redirect_url: String,
    pub session_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginWithProviderRequest {
    pub provider_id: Uuid,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscoverProvidersRequest {
    pub username_or_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderDiscoveryResult {
    pub user_exists: bool,
    pub requires_password: bool,
    pub requires_redirect: bool,
    pub available_providers: Vec<ProviderOption>,
    pub recommended_provider_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProviderOption {
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_type: String,
    pub requires_password: bool,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}
