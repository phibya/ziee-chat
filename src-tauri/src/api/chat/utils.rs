//! Chat utility functions for constructing message arrays for AI provider requests
//!
//! This module provides convenient functions to build ChatMessage arrays for different scenarios:
//! - Full conversations with history and file attachments
//! - Simple requests with just system instructions and user input
//! - Message editing and branching scenarios
//! - Single message construction for specialized tasks
//!
//! All functions handle file attachments, assistant instructions, and conversation history
//! according to the patterns established in the main chat API.

use uuid::Uuid;

use crate::ai::core::{ChatMessage, ContentPart, MessageContent};
use crate::database::models::MessageContentData;
use crate::database::queries::{
    assistants::get_assistant_by_id,
    chat::get_conversation_messages,
};

use super::ChatMessageRequest;

/// Build messages array for a chat request with conversation history and file attachments
pub async fn build_chat_messages(
    request: &ChatMessageRequest,
    user_id: Uuid,
) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut messages = Vec::new();

    // Add assistant instructions as system message if available
    if let Ok(Some(assistant)) = get_assistant_by_id(request.assistant_id, Some(user_id)).await {
        if let Some(instructions) = assistant.instructions {
            if !instructions.trim().is_empty() {
                messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: MessageContent::Text(instructions),
                });
            }
        }
    }

    // Add conversation history
    match get_conversation_messages(request.conversation_id, user_id).await {
        Ok(conversation_messages) => {
            for msg in conversation_messages {
                // Process each content item in the message
                for content_item in &msg.contents {
                    match &content_item.content {
                        MessageContentData::Text { text } => {
                            messages.push(ChatMessage {
                                role: msg.role.clone(),
                                content: MessageContent::Text(text.clone()),
                            });
                        }
                        MessageContentData::ToolCall { tool_name, server_id: _, arguments, call_id } => {
                            // Tool calls should be sent to AI provider so tool_result has corresponding tool_use
                            messages.push(ChatMessage {
                                role: msg.role.clone(),
                                content: MessageContent::Multimodal(vec![
                                    ContentPart::ToolUse {
                                        id: call_id.clone(),
                                        name: tool_name.clone(),
                                        input: arguments.clone(),
                                    }
                                ]),
                            });
                        }
                        MessageContentData::ToolResult { call_id, result, success, error_message } => {
                            // Tool results should be sent with role "tool"
                            let output = if *success {
                                serde_json::to_string_pretty(result).unwrap_or_else(|_| result.to_string())
                            } else {
                                error_message.clone().unwrap_or_else(|| "Tool execution failed".to_string())
                            };

                            messages.push(ChatMessage {
                                role: "tool".to_string(),
                                content: MessageContent::Multimodal(vec![
                                    ContentPart::ToolResult {
                                        call_id: call_id.clone(),
                                        output,
                                    }
                                ]),
                            });
                        }
                        // Skip other content types (ToolCallPendingApproval, etc.)
                        // as they are internal to our system and not sent to the AI provider
                        _ => {}
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load conversation history: {}", e);
            // Continue without history rather than failing completely
        }
    }

    Ok(messages)
}

/// Create a single user message (useful for simple requests like title generation)
pub fn build_single_user_message(content: String) -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: "user".to_string(),
        content: MessageContent::Text(content),
    }]
}

// ===================================================================
// Tool Calling Utilities
// ===================================================================

use crate::database::models::chat::ToolDefinition;
use crate::database::models::mcp_tool::MCPTool;
use crate::database::queries::mcp_tools;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Structure to track which tools are enabled for a conversation
/// This will be used to build tool definitions from MCP tools
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnabledMCPTool {
    pub server_id: Uuid,
    pub name: String,
}

/// Convert an MCP tool to an AI provider tool definition format
pub fn mcp_tool_to_definition(mcp_tool: &MCPTool) -> ToolDefinition {
    ToolDefinition {
        name: mcp_tool.tool_name.clone(),
        description: mcp_tool.tool_description.clone(),
        input_schema: mcp_tool.input_schema.clone(),
    }
}

/// Convert a list of enabled MCP tools to tool definitions for AI providers
/// This function fetches the full MCP tool data from the database and converts it
/// to the ToolDefinition format that AI providers understand
pub async fn build_tool_definitions(
    enabled_tools: &[EnabledMCPTool],
) -> Result<Vec<ToolDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    let mut definitions = Vec::new();

    for enabled_tool in enabled_tools {
        // Fetch MCP tool from database
        match mcp_tools::get_tool_by_server_and_name(enabled_tool.server_id, &enabled_tool.name)
            .await
        {
            Ok(Some(mcp_tool)) => {
                definitions.push(mcp_tool_to_definition(&mcp_tool));
            }
            Ok(None) => {
                eprintln!(
                    "Tool not found: {} on server {}",
                    enabled_tool.name, enabled_tool.server_id
                );
            }
            Err(e) => {
                eprintln!("Error fetching tool: {}", e);
            }
        }
    }

    Ok(definitions)
}
