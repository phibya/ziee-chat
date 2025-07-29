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
    pub is_private: bool,
    pub conversation_count: Option<i64>,
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
            is_private: row.try_get("is_private")?,
            conversation_count: row.try_get("conversation_count").ok(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConversation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub conversation_id: Uuid,
    pub conversation: Option<super::chat::Conversation>,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for ProjectConversation {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ProjectConversation {
            id: row.try_get("id")?,
            project_id: row.try_get("project_id")?,
            conversation_id: row.try_get("conversation_id")?,
            conversation: None, // This is loaded separately via joins when needed
            created_at: row.try_get("created_at")?,
        })
    }
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub instruction: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instruction: Option<String>,
    pub is_private: Option<bool>,
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
    pub conversations: Vec<ProjectConversation>,
}

