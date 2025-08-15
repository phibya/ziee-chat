use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

// Main Project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Project {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Project {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            instruction: row.try_get("instruction")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ProjectConversation struct removed - using conversations.project_id field instead

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetailResponse {
    pub project: Project,
    pub conversations: Vec<super::chat::Conversation>,
}
