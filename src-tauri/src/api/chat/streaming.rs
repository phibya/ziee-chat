//! Core streaming logic for chat messages with AI provider integration

use axum::response::sse::Event;
use futures_util::StreamExt;
use std::convert::Infallible;
use uuid::Uuid;

use crate::ai::SimplifiedChatRequest;
use crate::api::errors::ErrorCode;
use crate::database::models::SaveMessageRequest;
use crate::database::queries::{
    chat,
    models::{get_model_by_id, get_provider_by_model_id},
};
use super::utils::{build_chat_messages, build_tool_definitions};

use super::helpers::{generate_and_update_conversation_title, send_error};
use super::tool_handling::{check_and_handle_pending_approval, handle_tool_request};
use super::types::{
    ChatMessageRequest, CompleteData, MaxIterationReachedData, MessageContentChunkData,
    NewAssistantMessageData, NewMessageContentData, NewUserMessageData, SSEChatStreamEvent,
    StreamAIResult, StreamErrorData,
};

/// Stream AI response and save to database
///
/// This function handles the interaction with the AI model, including:
/// - Fetching conversation and model details
/// - Building chat messages
/// - Creating AI model instance
/// - Streaming response chunks
/// - Detecting tool use requests
/// - Saving content to database
pub(super) async fn stream_ai_response(
    tx: tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
    request: ChatMessageRequest,
    user_id: Uuid,
    last_assistant_message_id: Option<Uuid>,
) -> Result<StreamAIResult, Box<dyn std::error::Error + Send + Sync>> {
    // IMPORTANT: Capture the conversation's active branch immediately to prevent
    // race conditions if the user switches branches during streaming
    let active_branch_id =
        match chat::get_conversation_by_id(request.conversation_id, user_id).await {
            Ok(Some(conversation)) => conversation.active_branch_id,
            Ok(None) => {
                let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                    error: "Conversation not found".to_string(),
                    code: ErrorCode::ResourceNotFound.as_str().to_string(),
                });
                let _ = tx.send(Ok(error_event.into()));
                return Err("Conversation not found".into());
            }
            Err(e) => {
                let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                    error: format!("Error getting conversation: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                });
                let _ = tx.send(Ok(error_event.into()));
                return Err(e.into());
            }
        };

    // If resuming from an existing message, load enabled_tools from its metadata
    let mut request = request;
    if request.message_id.is_some() && request.enabled_tools.is_none() {
        if let Some(message_id) = request.message_id {
            // Load the message to get its metadata
            if let Ok(messages) = chat::get_conversation_messages(request.conversation_id, user_id).await {
                if let Some(message) = messages.iter().find(|m| m.id == message_id) {
                    if let Some(metadata) = &message.metadata {
                        request.enabled_tools = metadata.enabled_tools.clone();
                    }
                }
            }
        }
    }

    // Get the model provider configuration directly from model_id
    let provider = match get_provider_by_model_id(request.model_id).await {
        Ok(Some(provider)) => {
            println!("DEBUG: Found provider: {:?}", provider.name);
            provider
        }
        Ok(None) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: "Model or provider not found".to_string(),
                code: ErrorCode::ResourceModelNotFound.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err("Model or provider not found".into());
        }
        Err(e) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: format!("Error getting model provider: {}", e),
                code: ErrorCode::SystemDatabaseError.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err(e.into());
        }
    };

    // Check if provider is enabled
    if !provider.enabled {
        let error_event = SSEChatStreamEvent::Error(StreamErrorData {
            error: "Provider is disabled".to_string(),
            code: ErrorCode::ResourceProviderDisabled.as_str().to_string(),
        });
        let _ = tx.send(Ok(error_event.into()));
        return Err("Provider is disabled".into());
    }

    // Get the model to get the actual model name
    let model = match get_model_by_id(request.model_id).await {
        Ok(Some(model)) => {
            println!("DEBUG: Found model: {:?}", model.name);
            model
        }
        Ok(None) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: "Model not found".to_string(),
                code: ErrorCode::ResourceModelNotFound.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err("Model not found".into());
        }
        Err(e) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: format!("Error getting model: {}", e),
                code: ErrorCode::SystemDatabaseError.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err(e.into());
        }
    };

    // Build chat messages for AI provider using utility function
    let mut messages = match build_chat_messages(&request, user_id).await {
        Ok(messages) => messages,
        Err(e) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: format!("Error building chat messages: {}", e),
                code: ErrorCode::SystemInternalError.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err(e.into());
        }
    };

    // If resuming from a previous assistant message, remove the last user message
    // since we're continuing from where the assistant left off
    if last_assistant_message_id.is_some() {
        // Remove the last message if it's a user message
        if let Some(last_msg) = messages.last() {
            if last_msg.role == "user" {
                messages.pop();
            }
        }
    }

    // Check if this is a new conversation (count messages before moving them)
    // Count messages excluding system messages (assistant instructions)
    let user_and_assistant_messages = messages.iter().filter(|m| m.role != "system").count();

    // Create AI model instance using the new simplified API
    let ai_model =
        match crate::ai::model_manager::model_factory::create_ai_model(request.model_id).await {
            Ok(model) => model,
            Err(e) => {
                let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                    error: format!("Error creating AI model: {}", e),
                    code: ErrorCode::SystemInternalError.as_str().to_string(),
                });
                let _ = tx.send(Ok(error_event.into()));
                return Err(e);
            }
        };

    // Check if model supports streaming
    if !ai_model.supports_streaming() {
        let error_event = SSEChatStreamEvent::Error(StreamErrorData {
            error: "Provider does not support streaming responses".to_string(),
            code: ErrorCode::SystemInternalError.as_str().to_string(),
        });
        let _ = tx.send(Ok(error_event.into()));
        return Err("Provider does not support streaming responses".into());
    }

    // If there's only 1 message (the user message we just added), this is a new conversation
    // Generate title before streaming the response
    if user_and_assistant_messages == 1 {
        let conversation_id = request.conversation_id;

        // Wait for title generation to complete before streaming the response
        let _ = generate_and_update_conversation_title(conversation_id, user_id, &model, &tx).await;
    }

    // Build tool definitions from enabled_tools if provided
    let tools = if let Some(enabled_tools) = &request.enabled_tools {
        match build_tool_definitions(enabled_tools).await {
            Ok(defs) => Some(defs),
            Err(e) => {
                eprintln!("Warning: Failed to build tool definitions: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Create or get the assistant message ID BEFORE streaming
    let message_id = if let Some(existing_message_id) = last_assistant_message_id {
        // Resuming from previous message
        existing_message_id
    } else {
        // New message - create empty message first
        let assistant_message_req = SaveMessageRequest {
            conversation_id: request.conversation_id,
            content: String::new(), // Empty content initially
            role: "assistant".to_string(),
            model_id: request.model_id,
            file_ids: None,
            enabled_tools: request.enabled_tools.clone(),
        };

        match chat::save_message(assistant_message_req, user_id, active_branch_id).await {
            Ok(assistant_message) => {
                let asst_msg_id = assistant_message.id;

                // Send NewAssistantMessage event
                let new_asst_msg_event =
                    SSEChatStreamEvent::NewAssistantMessage(NewAssistantMessageData {
                        message_id: asst_msg_id,
                    });
                let _ = tx.send(Ok(new_asst_msg_event.into()));

                asst_msg_id
            }
            Err(e) => {
                send_error(
                    &tx,
                    format!("Error creating assistant message: {}", e),
                    ErrorCode::SystemDatabaseError,
                )
                .await;
                return Err(e.into());
            }
        }
    };

    // Call AI model with streaming
    match ai_model
        .chat_stream(SimplifiedChatRequest {
            messages,
            stream: true,
            tools,
        })
        .await
    {
        Ok(mut stream) => {
            let mut full_content = String::new();
            let mut tool_use_option: Option<crate::database::models::ToolUse> = None;
            let mut message_content_id: Option<Uuid> = None;

            // Process the stream
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(content) = &chunk.content {
                            full_content.push_str(content);

                            // Create message_content_id and send NewMessageContent on first chunk
                            if message_content_id.is_none() {
                                let content_id = Uuid::new_v4();
                                message_content_id = Some(content_id);

                                // Send NewMessageContent event BEFORE first chunk
                                let new_content_event =
                                    SSEChatStreamEvent::NewMessageContent(NewMessageContentData {
                                        message_content_id: content_id,
                                        message_id,
                                    });
                                let _ = tx.send(Ok(new_content_event.into()));
                            }

                            // Send chunk to client with content_id
                            if let Some(content_id) = message_content_id {
                                let chunk_event = SSEChatStreamEvent::MessageContentChunk(
                                    MessageContentChunkData {
                                        message_content_id: content_id,
                                        delta: content.to_string(),
                                    },
                                );
                                let _ = tx.send(Ok(chunk_event.into()));
                            }
                        }

                        // Check for tool use
                        if let Some(tool_use) = chunk.tool_use {
                            tool_use_option = Some(tool_use);
                        }

                        // Check if streaming is complete
                        if chunk.finish_reason.is_some() {
                            break;
                        }
                    }
                    Err(e) => {
                        let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                            error: format!("Streaming error: {}", e),
                            code: ErrorCode::SystemStreamingError.as_str().to_string(),
                        });
                        let _ = tx.send(Ok(error_event.into()));
                        return Err(e.into());
                    }
                }
            }

            // Save the text content to the message
            if !full_content.is_empty() {
                match chat::append_text_content_to_message(message_id, full_content.clone()).await {
                    Ok(_) => {}
                    Err(e) => {
                        send_error(
                            &tx,
                            format!("Error saving text content: {}", e),
                            ErrorCode::SystemDatabaseError,
                        )
                        .await;
                        return Err(e.into());
                    }
                }
            }

            // NOTE: Complete event is sent by the caller (send_message_stream)
            // after the tool approval loop completes, not here

            // Extract tool call request if present
            let tool_call_request = if let Some(tool_use) = tool_use_option {
                // Match the tool_name against enabled_tools to get server_id
                if let Some(enabled_tools) = &request.enabled_tools {
                    let matching_tool = enabled_tools.iter().find(|t| t.name == tool_use.name);
                    if let Some(tool) = matching_tool {
                        Some(super::types::ToolCallRequest {
                            server_id: tool.server_id,
                            tool_name: tool_use.name,
                            arguments: tool_use.input,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            return Ok(StreamAIResult {
                message_id,
                tool_call_request,
            });
        }
        Err(e) => {
            let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                error: format!("Error calling AI provider: {}", e),
                code: ErrorCode::SystemExternalServiceError.as_str().to_string(),
            });
            let _ = tx.send(Ok(error_event.into()));
            return Err(e.into());
        }
    }
}

/// Execute the main message streaming loop with tool approval support
///
/// This is the shared core logic for both send_message_stream and edit_message_stream.
/// It handles the iterative tool approval flow, AI response streaming, and tool execution.
///
/// Parameters:
/// - tx: SSE event channel for sending events to the client
/// - request: Chat message request containing conversation_id, content, model_id, etc.
/// - user_id: User ID for database operations
/// - should_create_user_message: Whether to create a new user message (true for send, false for edit)
/// - resume_from_message_id: Optional message ID to resume from (used when continuing after approval)
///
/// Flow:
/// 1. Main loop (MAX_ITERATIONS times):
///    - Check for pending approval (if resuming)
///    - Create user message (if should_create_user_message and not resuming)
///    - Stream AI response
///    - Handle tool request (if any)
/// 2. Send MaxIterationReached event (if max iterations reached)
/// 3. Register model access
/// 4. Send Complete event
///
/// Returns: Ok(()) on success, Err on failure
pub(super) async fn execute_message_stream_loop(
    tx: tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
    request: ChatMessageRequest,
    user_id: Uuid,
    should_create_user_message: bool,
    resume_from_message_id: Option<Uuid>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const MAX_ITERATIONS: usize = 5;
    let mut iteration = 0;
    let mut last_assistant_message_id: Option<Uuid> = resume_from_message_id;
    let is_resuming = resume_from_message_id.is_some();

    while iteration < MAX_ITERATIONS {
        iteration += 1;

        // ----------------------------------------
        // 1. Check last message for pending approval (if resuming or after first iteration)
        // ----------------------------------------
        if is_resuming || iteration > 1 {
            match check_and_handle_pending_approval(
                request.conversation_id,
                user_id,
                &mut last_assistant_message_id,
                &tx,
            )
            .await
            {
                Ok((_, should_continue)) => {
                    if !should_continue {
                        return Ok(()); // Stop if we need to wait for approval or had an error
                    }
                }
                Err(e) => {
                    send_error(
                        &tx,
                        format!("Failed to check approval: {}", e),
                        ErrorCode::SystemDatabaseError,
                    )
                    .await;
                    return Err(e);
                }
            }
        }

        // ----------------------------------------
        // 2. Save user message on first iteration if needed
        // ----------------------------------------
        if iteration == 1 && should_create_user_message && !is_resuming {
            let user_message_req = SaveMessageRequest {
                conversation_id: request.conversation_id,
                content: request.content.clone(),
                role: "user".to_string(),
                model_id: request.model_id,
                file_ids: request.file_ids.clone(),
                enabled_tools: request.enabled_tools.clone(),
            };

            match chat::save_message(user_message_req, user_id, None).await {
                Ok(user_message) => {
                    // Send NewUserMessage event
                    let new_user_message_event =
                        SSEChatStreamEvent::NewUserMessage(NewUserMessageData {
                            message_id: user_message.id,
                        });
                    let _ = tx.send(Ok(new_user_message_event.into()));
                }
                Err(e) => {
                    send_error(
                        &tx,
                        format!("Failed to save user message: {}", e),
                        ErrorCode::SystemDatabaseError,
                    )
                    .await;
                    return Err(e.into());
                }
            }
        }

        // ----------------------------------------
        // 3. Stream AI response (saves message and returns result)
        // ----------------------------------------
        let result = match stream_ai_response(tx.clone(), request.clone(), user_id, last_assistant_message_id).await {
            Ok(result) => result,
            Err(e) => {
                // Error already sent by stream_ai_response
                return Err(e);
            }
        };

        last_assistant_message_id = Some(result.message_id);

        // ----------------------------------------
        // 4. Check if AI requests tool call
        // ----------------------------------------
        if let Some(tool_request) = result.tool_call_request {
            if !handle_tool_request(tool_request, result.message_id, request.conversation_id, &tx).await {
                // Failed to handle tool request, error already sent
                return Err("Failed to handle tool request".into());
            }

            // Continue loop - approval will be checked in next iteration
            continue;
        }

        // No tool call - we're done
        break;
    }

    if iteration >= MAX_ITERATIONS {
        // Send MaxIterationReached event
        let max_iteration_event =
            SSEChatStreamEvent::MaxIterationReached(MaxIterationReachedData {
                iteration: iteration as i32,
            });
        let _ = tx.send(Ok(max_iteration_event.into()));
        // Continue to send Complete event below
    }

    // Send complete event
    if last_assistant_message_id.is_some() {
        // Register model access for auto-unload tracking
        crate::ai::register_model_access(&request.model_id).await;

        // Send Complete event (no data)
        let complete_event = SSEChatStreamEvent::Complete(CompleteData {});
        let _ = tx.send(Ok(complete_event.into()));
    }

    Ok(())
}
