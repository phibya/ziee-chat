//! Type definitions for chat SSE streaming

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{Message, MessageBranch};
use crate::utils::chat::EnabledMCPTool;

/// Request structure for sending/editing chat messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub model_id: Uuid,
    pub assistant_id: Uuid,
    pub file_ids: Option<Vec<Uuid>>,         // Optional file attachments
    pub enabled_tools: Option<Vec<EnabledMCPTool>>, // Optional MCP tools to send to AI
    pub message_id: Option<Uuid>,            // Optional message ID to resume from
}

// ============================================
// SSE Event Data Structures
// ============================================

/// Empty event data for Connected event
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ConnectedData {}

/// Empty event data for Complete event
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CompleteData {}

/// Error event data
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StreamErrorData {
    pub error: String,
    pub code: String,
}

// Message lifecycle events
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NewUserMessageData {
    pub message_id: Uuid,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NewAssistantMessageData {
    pub message_id: Uuid,
}

// Content streaming events
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NewMessageContentData {
    pub message_content_id: Uuid,
    pub message_id: Uuid,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MessageContentChunkData {
    pub message_content_id: Uuid,
    pub delta: String,
}

// Tool-related events
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolCallData {
    pub message_content_id: Uuid,
    pub message_id: Uuid,
    pub tool_name: String,
    pub server_id: Uuid,
    pub arguments: serde_json::Value,
    pub call_id: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolCallPendingApprovalData {
    pub message_content_id: Uuid,
    pub message_id: Uuid,
    pub tool_name: String,
    pub server_id: Uuid,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolResultData {
    pub message_content_id: Uuid,
    pub message_id: Uuid,
    pub call_id: String,
    pub result: serde_json::Value,
    pub success: bool,
    pub error_message: Option<String>,
}

// Other events
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TitleUpdatedData {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MaxIterationReachedData {
    pub iteration: i32,
}

// ============================================
// SSE Event Enum
// ============================================

// SSE event types for chat streaming
crate::sse_event_enum! {
    #[derive(Debug, Clone, Serialize, JsonSchema)]
    pub enum SSEChatStreamEvent {
        Connected(ConnectedData),
        NewUserMessage(NewUserMessageData),
        NewAssistantMessage(NewAssistantMessageData),
        NewMessageContent(NewMessageContentData),
        MessageContentChunk(MessageContentChunkData),
        ToolCall(ToolCallData),
        ToolCallPendingApproval(ToolCallPendingApprovalData),
        ToolResult(ToolResultData),
        TitleUpdated(TitleUpdatedData),
        MaxIterationReached(MaxIterationReachedData),
        Complete(CompleteData),
        Error(StreamErrorData),
        EditedMessage(Message),
        CreatedBranch(MessageBranch),
    }
}

// ============================================
// Internal Types
// ============================================

/// Result from streaming AI response
pub(super) struct StreamAIResult {
    pub message_id: Uuid,
    pub tool_call_request: Option<ToolCallRequest>,
}

/// Tool call request extracted from AI response
pub(super) struct ToolCallRequest {
    pub server_id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}
