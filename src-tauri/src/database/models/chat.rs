use crate::database::macros::{impl_json_option_from, make_transparent};
use crate::database::models::File;
use crate::database::types::JsonOption;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Main unified structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

make_transparent!(
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct MessageFiles(Vec<File>)
);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub originated_from_id: Uuid, // ID of the original message this was edited from
    pub edit_count: i32,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: JsonOption<Vec<MessageMetadata>>,
    pub files: MessageFiles,
}

// Branch structure for proper branching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// MessageBranch model that includes is_clone information from branch_messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessageBranch {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_clone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
pub struct MessageMetadata {
    pub id: Uuid,
    pub message_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// Implement JSON conversion for Vec<MessageMetadata>
impl_json_option_from!(Vec<MessageMetadata>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateConversationRequest {
    pub title: String,
    pub project_id: Option<Uuid>,
    pub assistant_id: Uuid,
    pub model_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub assistant_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub role: String,
    pub model_id: Uuid,
    pub file_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
    pub file_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageResponse {
    pub message: Message,
    pub branch: MessageBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationSummary>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
