use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Main Project structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProjectDetailResponse {
    pub project: Project,
    pub conversations: Vec<super::chat::Conversation>,
}
