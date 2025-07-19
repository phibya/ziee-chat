use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Assistants structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssistantDb {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub is_template: bool,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    pub name: String,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub is_template: Option<bool>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub parameters: Option<serde_json::Value>,
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

// Chat structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub originated_from_id: Option<Uuid>, // ID of the original message this was edited from
    pub edit_count: Option<i32>,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Branch structures for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BranchDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageMetadataDb {
    pub id: Uuid,
    pub message_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationMetadataDb {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub active_branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

// Branch API model for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// MessageBranch API model that includes is_clone information from branch_messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBranch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_clone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
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