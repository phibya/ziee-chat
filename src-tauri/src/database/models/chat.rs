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

// Content type enum for structured message content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "lowercase")]
pub enum MessageContentType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "tool_call")]
    ToolCall,
    #[serde(rename = "tool_result")]
    ToolResult,
    #[serde(rename = "file_attachment")]
    FileAttachment,
    #[serde(rename = "error")]
    Error,
}

// Content data enum for different content types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum MessageContentData {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_call")]
    ToolCall {
        tool_name: String,
        server_id: Uuid,
        arguments: serde_json::Value,
        call_id: String,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        call_id: String,
        result: serde_json::Value,
        success: bool,
        error_message: Option<String>,
    },

    #[serde(rename = "file_attachment")]
    FileAttachment {
        file_id: Uuid,
        filename: String,
        file_type: Option<String>,
    },

    #[serde(rename = "error")]
    Error {
        error_type: String,
        message: String,
        details: Option<serde_json::Value>,
    },
}

impl MessageContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageContentType::Text => "text",
            MessageContentType::ToolCall => "tool_call",
            MessageContentType::ToolResult => "tool_result",
            MessageContentType::FileAttachment => "file_attachment",
            MessageContentType::Error => "error",
        }
    }
}

impl std::fmt::Display for MessageContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Database row struct for message content queries
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageContentRow {
    pub id: Uuid,
    pub message_id: Uuid,
    pub content_type: MessageContentType,
    pub content: serde_json::Value,
    pub sequence_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Simple content structure
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessageContentItem {
    pub id: Uuid,
    pub message_id: Uuid,
    pub content_type: MessageContentType,
    pub content: MessageContentData,
    pub sequence_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MessageContentRow> for MessageContentItem {
    fn from(row: MessageContentRow) -> Self {
        let content = match row.content_type {
            MessageContentType::Text => {
                if let Some(text) = row.content.get("text").and_then(|v| v.as_str()) {
                    MessageContentData::Text { text: text.to_string() }
                } else {
                    MessageContentData::Text { text: String::new() }
                }
            }
            MessageContentType::ToolCall => {
                // For now, serialize the entire JSON as the content
                // This will be properly implemented when MCP is added
                MessageContentData::Text { text: row.content.to_string() }
            }
            MessageContentType::ToolResult => {
                MessageContentData::Text { text: row.content.to_string() }
            }
            MessageContentType::FileAttachment => {
                MessageContentData::Text { text: row.content.to_string() }
            }
            MessageContentType::Error => {
                MessageContentData::Text { text: row.content.to_string() }
            }
        };

        Self {
            id: row.id,
            message_id: row.message_id,
            content_type: row.content_type,
            content,
            sequence_order: row.sequence_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// Database row struct for queries (without complex fields)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub originated_from_id: Uuid,
    pub edit_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub originated_from_id: Uuid, // ID of the original message this was edited from
    pub edit_count: i32,          // Number of times this message lineage has been edited
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: JsonOption<Vec<MessageMetadata>>,
    pub files: MessageFiles,
    pub contents: Vec<MessageContentItem>, // NEW: structured content
}

impl Message {
    // Simple helper to get text content
    pub fn get_text_content(&self) -> String {
        self.contents
            .iter()
            .filter(|c| matches!(c.content_type, MessageContentType::Text))
            .filter_map(|c| match &c.content {
                MessageContentData::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
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
