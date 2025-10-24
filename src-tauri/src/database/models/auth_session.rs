#![allow(dead_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthSession {
    pub id: Uuid,
    pub session_key: String,
    pub provider_id: Uuid,
    pub state: String,
    pub nonce: Option<String>,
    pub code_verifier: Option<String>,
    pub redirect_uri: Option<String>,
    pub metadata: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuthSessionRequest {
    pub session_key: String,
    pub provider_id: Uuid,
    pub state: String,
    pub nonce: Option<String>,
    pub code_verifier: Option<String>,
    pub redirect_uri: Option<String>,
    pub metadata: serde_json::Value,
    pub expires_at: DateTime<Utc>,
}
