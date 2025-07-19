use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Project structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDocumentDb {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub content_text: Option<String>,
    pub upload_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectConversationDb {
    pub id: Uuid,
    pub project_id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// API structures for projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub document_count: Option<i64>,
    pub conversation_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub content_text: Option<String>,
    pub upload_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConversation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub conversation_id: Uuid,
    pub conversation: Option<super::chat::Conversation>,
    pub created_at: DateTime<Utc>,
}

// Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
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
    pub documents: Vec<ProjectDocument>,
    pub conversations: Vec<ProjectConversation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadDocumentResponse {
    pub document: ProjectDocument,
    pub upload_url: Option<String>, // For future file upload handling
}

impl Project {
    pub fn from_db(
        project_db: ProjectDb,
        document_count: Option<i64>,
        conversation_count: Option<i64>,
    ) -> Self {
        Self {
            id: project_db.id,
            user_id: project_db.user_id,
            name: project_db.name,
            description: project_db.description,
            is_private: project_db.is_private,
            document_count,
            conversation_count,
            created_at: project_db.created_at,
            updated_at: project_db.updated_at,
        }
    }
}

impl ProjectDocument {
    pub fn from_db(document_db: ProjectDocumentDb) -> Self {
        Self {
            id: document_db.id,
            project_id: document_db.project_id,
            file_name: document_db.file_name,
            file_path: document_db.file_path,
            file_size: document_db.file_size,
            mime_type: document_db.mime_type,
            content_text: document_db.content_text,
            upload_status: document_db.upload_status,
            created_at: document_db.created_at,
            updated_at: document_db.updated_at,
        }
    }
}

impl ProjectConversation {
    pub fn from_db(pc_db: ProjectConversationDb, conversation: Option<super::chat::Conversation>) -> Self {
        Self {
            id: pc_db.id,
            project_id: pc_db.project_id,
            conversation_id: pc_db.conversation_id,
            conversation,
            created_at: pc_db.created_at,
        }
    }
}