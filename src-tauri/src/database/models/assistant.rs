use super::model::ModelParameters;
use crate::database::types::JsonOption;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Assistant {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: JsonOption<ModelParameters>,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response structures for assistants
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAssistantRequest {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<ModelParameters>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateAssistantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<ModelParameters>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssistantListResponse {
    pub assistants: Vec<Assistant>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
