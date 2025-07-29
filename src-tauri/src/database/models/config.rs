use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Row};

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

impl FromRow<'_, sqlx::postgres::PgRow> for Configuration {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Configuration {
            id: row.try_get("id")?,
            key: row.try_get("key")?,
            value: row.try_get("value")?,
            description: row.try_get("description")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}