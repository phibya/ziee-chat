use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

// Main unified structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub project_id: Option<Uuid>,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Conversation {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Conversation {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            title: row.try_get("title")?,
            project_id: row.try_get("project_id")?,
            assistant_id: row.try_get("assistant_id")?,
            model_id: row.try_get("model_id")?,
            active_branch_id: row.try_get("active_branch_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<Vec<MessageMetadata>>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Message {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Message {
            id: row.try_get("id")?,
            conversation_id: row.try_get("conversation_id")?,
            role: row.try_get("role")?,
            content: row.try_get("content")?,
            originated_from_id: row.try_get("originated_from_id")?,
            edit_count: row.try_get("edit_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            metadata: None, // This is loaded separately via joins when needed
        })
    }
}

// Branch structure for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Branch {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Branch {
            id: row.try_get("id")?,
            conversation_id: row.try_get("conversation_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// MessageBranch model that includes is_clone information from branch_messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBranch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_clone: bool,
}

impl FromRow<'_, sqlx::postgres::PgRow> for MessageBranch {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(MessageBranch {
            id: row.try_get("id")?,
            conversation_id: row.try_get("conversation_id")?,
            created_at: row.try_get("created_at")?,
            is_clone: row.try_get("is_clone")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub id: Uuid,
    pub message_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for MessageMetadata {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(MessageMetadata {
            id: row.try_get("id")?,
            message_id: row.try_get("message_id")?,
            key: row.try_get("key")?,
            value: row.try_get("value")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::postgres::PgRow> for ConversationMetadata {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ConversationMetadata {
            id: row.try_get("id")?,
            conversation_id: row.try_get("conversation_id")?,
            key: row.try_get("key")?,
            value: row.try_get("value")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    pub project_id: Option<Uuid>,
    pub assistant_id: Uuid,
    pub model_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub role: String,
    pub model_id: Uuid,
    pub file_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageResponse {
    pub message: Message,
    pub content_changed: bool,
    pub conversation_history: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationSummary>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: String,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub conversation: Conversation,
}

// AI Provider related structs moved from ai/core/providers.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReference {
    pub file_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
}

impl FileReference {
    pub fn is_image(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| mt.starts_with("image/"))
            .unwrap_or(false)
    }
    
    pub fn is_pdf(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| mt == "application/pdf")
            .unwrap_or(false)
    }
    
    pub fn is_text(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| mt.starts_with("text/"))
            .unwrap_or(false)
    }
    
    pub fn is_spreadsheet(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| {
                mt == "application/vnd.ms-excel" 
                || mt == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                || mt == "application/vnd.oasis.opendocument.spreadsheet"
                || mt == "text/csv"
            })
            .unwrap_or(false)
    }
    
    pub fn is_document(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| {
                mt == "application/msword"
                || mt == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                || mt == "application/vnd.ms-powerpoint"
                || mt == "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                || mt == "application/vnd.oasis.opendocument.text"
                || mt == "application/vnd.oasis.opendocument.presentation"
                || mt == "application/rtf"
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    FileReference(FileReference),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text(text)
    }
}

impl MessageContent {
    pub fn text(content: &str) -> Self {
        MessageContent::Text(content.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

impl ChatMessage {
    pub fn text(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: MessageContent::Text(content.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model_name: String,
    pub model_id: Uuid,
    pub provider_id: Uuid,
    pub stream: bool,
    pub parameters: Option<crate::database::models::model::ModelParameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderChatResponse {
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    pub content: Option<String>,
    pub finish_reason: Option<String>,
}