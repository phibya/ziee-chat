//! Tool approval and execution handling

use axum::response::sse::Event;
use std::convert::Infallible;
use uuid::Uuid;

use crate::api::errors::ErrorCode;
use crate::database::models::{MessageContentData, MessageContentType};
use crate::database::queries::chat;

use super::helpers::send_error;
use super::types::{
    NewMessageContentData, SSEChatStreamEvent, ToolCallData, ToolCallPendingApprovalData,
    ToolCallRequest, ToolResultData,
};

/// Check if the last message needs approval and handle it
/// Returns (needs_approval, should_continue_loop)
pub(super) async fn check_and_handle_pending_approval(
    conversation_id: Uuid,
    user_id: Uuid,
    last_assistant_message_id: &mut Option<Uuid>,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> Result<(bool, bool), Box<dyn std::error::Error + Send + Sync>> {
    let messages = chat::get_conversation_messages(conversation_id, user_id).await?;

    let last_message = messages.last();
    let needs_approval = if let Some(msg) = last_message {
        if msg.role != "assistant" {
            false
        } else {
            // Sort contents by sequence_order and check if the LAST one is ToolCallPendingApproval
            let mut sorted_contents = msg.contents.clone();
            sorted_contents.sort_by_key(|c| c.sequence_order);

            sorted_contents
                .last()
                .map(|c| c.content_type == MessageContentType::ToolCallPendingApproval)
                .unwrap_or(false)
        }
    } else {
        false
    };

    if !needs_approval {
        return Ok((false, true)); // No approval needed, continue
    }

    let last_msg = last_message.unwrap();

    // Sort contents and get the last one (which should be ToolCallPendingApproval)
    let mut sorted_contents = last_msg.contents.clone();
    sorted_contents.sort_by_key(|c| c.sequence_order);
    let pending_content = sorted_contents.last();

    if let Some(content) = pending_content {
        // Try to parse the pending approval data
        let tool_data: Result<MessageContentData, _> = serde_json::from_value(
            serde_json::to_value(&content.content).unwrap_or_default(),
        );

        if let Ok(MessageContentData::ToolCallPendingApproval {
            tool_name,
            server_id,
            arguments,
            is_approved: _,
        }) = tool_data
        {
            // Check if approved
            let is_approved =
                chat::check_tool_approval(conversation_id, server_id, &tool_name).await?;

            if !is_approved {
                // Send approval request event and EXIT
                let approval_event =
                    SSEChatStreamEvent::ToolCallPendingApproval(ToolCallPendingApprovalData {
                        message_content_id: content.id,
                        message_id: last_msg.id,
                        tool_name: tool_name.clone(),
                        server_id,
                        arguments: arguments.clone(),
                    });
                let _ = tx.send(Ok(approval_event.into()));
                return Ok((true, false)); // Needs approval, stop loop
            }

            // Approved! Execute tool
            *last_assistant_message_id = Some(last_msg.id);

            if let Err(e) =
                execute_tool_and_save_result(last_msg.id, server_id, &tool_name, &arguments, tx)
                    .await
            {
                send_error(
                    tx,
                    format!("Tool execution failed: {}", e),
                    ErrorCode::SystemInternalError,
                )
                .await;
                return Ok((true, false)); // Don't continue loop
            }

            // Continue to next iteration
            return Ok((true, true));
        } else {
            send_error(
                tx,
                "Invalid pending approval data".to_string(),
                ErrorCode::SystemInternalError,
            )
            .await;
            return Ok((true, false));
        }
    }

    Ok((false, true))
}

/// Handle tool call request from AI by saving pending approval and sending events
/// Returns true if tool request was handled, false otherwise
pub(super) async fn handle_tool_request(
    tool_request: ToolCallRequest,
    message_id: Uuid,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> bool {
    // Save ToolCallPendingApproval to DB
    let pending_content = MessageContentData::ToolCallPendingApproval {
        tool_name: tool_request.tool_name.clone(),
        server_id: tool_request.server_id,
        arguments: tool_request.arguments.clone(),
        is_approved: None,
    };

    match chat::save_pending_tool_approval_content(message_id, pending_content).await {
        Ok(message_content_id) => {
            // Send NewMessageContent event
            let new_content_event = SSEChatStreamEvent::NewMessageContent(NewMessageContentData {
                message_content_id,
                message_id,
            });
            let _ = tx.send(Ok(new_content_event.into()));

            // Send ToolCallPendingApproval event (for UI modal) - BEFORE ToolCall
            let approval_event =
                SSEChatStreamEvent::ToolCallPendingApproval(ToolCallPendingApprovalData {
                    message_content_id,
                    message_id,
                    tool_name: tool_request.tool_name,
                    server_id: tool_request.server_id,
                    arguments: tool_request.arguments,
                });
            let _ = tx.send(Ok(approval_event.into()));

            // NOTE: ToolCall event is sent AFTER approval in execute_tool_and_save_result
            true
        }
        Err(e) => {
            send_error(
                tx,
                format!("Failed to save pending approval: {}", e),
                ErrorCode::SystemDatabaseError,
            )
            .await;
            false
        }
    }
}

/// Execute MCP tool and save result to DB (MOCK EXECUTION FOR NOW)
async fn execute_tool_and_save_result(
    message_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
    arguments: &serde_json::Value,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate call_id for this tool execution
    let call_id = Uuid::new_v4().to_string();

    // Save ToolCall content to database
    let tool_call_content = MessageContentData::ToolCall {
        tool_name: tool_name.to_string(),
        server_id,
        arguments: arguments.clone(),
        call_id: call_id.clone(),
    };

    let tool_call_content_id =
        chat::save_tool_call_content(message_id, tool_call_content).await?;

    // Send NewMessageContent event for ToolCall
    let new_content_event = SSEChatStreamEvent::NewMessageContent(NewMessageContentData {
        message_content_id: tool_call_content_id,
        message_id,
    });
    let _ = tx.send(Ok(new_content_event.into()));

    // Send ToolCall event (tool is approved and being executed)
    let tool_call_event = SSEChatStreamEvent::ToolCall(ToolCallData {
        message_content_id: tool_call_content_id,
        message_id,
        tool_name: tool_name.to_string(),
        server_id,
        arguments: arguments.clone(),
        call_id: call_id.clone(),
    });
    let _ = tx.send(Ok(tool_call_event.into()));

    // MOCK EXECUTION - Return mock result
    // TODO: Replace with actual MCP tool execution
    let mock_result = serde_json::json!({
        "status": "success",
        "message": format!("Mock execution of tool: {}", tool_name),
        "tool_name": tool_name,
        "server_id": server_id,
        "arguments": arguments,
        "note": "This is a mock result. Actual MCP execution will be implemented later."
    });

    // Save ToolResult content to message
    let result_content = MessageContentData::ToolResult {
        call_id: call_id.clone(),
        result: mock_result.clone(),
        success: true,
        error_message: None,
    };

    // Save to database using query function and get the content_id
    let message_content_id = chat::save_tool_result_content(message_id, result_content).await?;

    // Send NewMessageContent event
    let new_content_event = SSEChatStreamEvent::NewMessageContent(NewMessageContentData {
        message_content_id,
        message_id,
    });
    let _ = tx.send(Ok(new_content_event.into()));

    // Send ToolResult event
    let event = SSEChatStreamEvent::ToolResult(ToolResultData {
        message_content_id,
        message_id,
        call_id,
        result: mock_result,
        success: true,
        error_message: None,
    });
    let _ = tx.send(Ok(event.into()));

    Ok(())
}
