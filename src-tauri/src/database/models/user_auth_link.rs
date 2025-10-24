#![allow(dead_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserAuthLink {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_id: Uuid,
    pub external_id: String,
    pub external_username: Option<String>,
    pub external_email: Option<String>,
    pub external_metadata: serde_json::Value,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserAuthLinkRequest {
    pub user_id: Uuid,
    pub provider_id: Uuid,
    pub external_id: String,
    pub external_username: Option<String>,
    pub external_email: Option<String>,
    pub external_metadata: serde_json::Value,
}
