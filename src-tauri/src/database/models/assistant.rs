use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::model::ModelParameters;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<ModelParameters>,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Assistant {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        // Parse parameters JSON to ModelParameters
        let parameters_json: serde_json::Value = row.try_get("parameters")?;
        let parameters = if parameters_json.is_null() {
            None
        } else {
            // Deserialize JSON to ModelParameters
            match serde_json::from_value::<ModelParameters>(parameters_json) {
                Ok(params) => Some(params),
                Err(e) => {
                    eprintln!("Warning: Failed to parse assistant parameters: {}. Using default parameters.", e);
                    // Fallback to default parameters if parsing fails
                    Some(ModelParameters::default())
                }
            }
        };

        Ok(Assistant {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            instructions: row.try_get("instructions")?,
            parameters,
            created_by: row.try_get("created_by")?,
            is_template: row.try_get("is_template")?,
            is_default: row.try_get("is_default")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
// Request/Response structures for assistants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<ModelParameters>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<ModelParameters>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantListResponse {
    pub assistants: Vec<Assistant>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
