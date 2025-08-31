use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub id: i32,
    pub key: String,
    pub value: Value,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
