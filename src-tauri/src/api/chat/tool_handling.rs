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
                // Not approved yet - stop and wait for approval
                // Note: The approval event was already sent when the tool was first requested
                // by handle_tool_request(), so we don't send it again here
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
    conversation_id: Uuid,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> bool {
    // Check if tool is already approved in the database
    let is_already_approved = match chat::check_tool_approval(conversation_id, tool_request.server_id, &tool_request.tool_name).await {
        Ok(approved) => approved,
        Err(e) => {
            eprintln!("Warning: Failed to check tool approval: {}", e);
            false // Proceed with normal flow on error
        }
    };

    // Save ToolCallPendingApproval to DB (always save, regardless of approval status)
    let pending_content = MessageContentData::ToolCallPendingApproval {
        tool_name: tool_request.tool_name.clone(),
        server_id: tool_request.server_id,
        arguments: tool_request.arguments.clone(),
        is_approved: None,
    };

    match chat::save_pending_tool_approval_content(message_id, pending_content).await {
        Ok(message_content_id) => {
            // Only send approval events if tool is not already approved
            if !is_already_approved {
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
            }

            // NOTE: ToolCall event is sent AFTER approval in execute_tool_and_save_result
            // Tool execution happens in the next iteration via check_and_handle_pending_approval
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

    // Execute tool via MCP
    let start_time = std::time::Instant::now();

    let execution_result = crate::mcp::tool_executor::execute_mcp_tool(
        server_id,
        tool_name.to_string(),
        arguments.clone(),
    )
    .await;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    // Handle result
    let (result, success, error_message) = match execution_result {
        Ok(exec_result) => {
            if exec_result.success {
                tracing::info!(
                    "Tool '{}' executed successfully in {}ms",
                    tool_name,
                    exec_result.duration_ms
                );
                (exec_result.result, true, None)
            } else {
                tracing::error!(
                    "Tool '{}' execution failed: {:?}",
                    tool_name,
                    exec_result.error_message
                );
                (None, false, exec_result.error_message)
            }
        }
        Err(e) => {
            tracing::error!("Tool '{}' execution error: {}", tool_name, e);
            (
                None,
                false,
                Some(format!("Tool execution failed: {}", e)),
            )
        }
    };

    // Prepare result value for both database and SSE event
    let result_value = result.clone().unwrap_or_else(|| {
        serde_json::json!({
            "error": "execution_failed",
            "duration_ms": duration_ms
        })
    });

    // Save ToolResult content to message
    let result_content = MessageContentData::ToolResult {
        call_id: call_id.clone(),
        result: result_value.clone(),
        success,
        error_message: error_message.clone(),
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
        result: result_value,
        success,
        error_message,
    });
    let _ = tx.send(Ok(event.into()));

    Ok(())
}
