use axum::response::sse::{Event, KeepAlive};
use axum::{
    debug_handler,
    extract::Path,
    http::StatusCode,
    response::Sse,
    Extension, Json,
};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::ai::SimplifiedChatRequest;
use crate::api::errors::{ApiResult, AppError, ErrorCode};
use crate::api::middleware::AuthenticatedUser;
use crate::database::models::{EditMessageRequest, Message, MessageContentData, MessageContentType, SaveMessageRequest, UpdateConversationRequest};
use crate::database::queries::{
    chat,
    models::{get_model_by_id, get_provider_by_model_id},
};
use crate::utils::chat::{build_chat_messages, build_single_user_message, build_tool_definitions, EnabledMCPTool};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub model_id: Uuid,
    pub assistant_id: Uuid,
    pub file_ids: Option<Vec<Uuid>>, // Optional file attachments
    pub enabled_tools: Option<Vec<EnabledMCPTool>>, // Optional MCP tools to send to AI
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StreamChunkData {
    pub delta: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StreamCompleteData {
    pub message_id: String,
    pub conversation_id: String,
    pub role: String,
    pub originated_from_id: String,
    pub edit_count: i32,
    pub created_at: String,
    pub updated_at: String,
    pub total_tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StreamErrorData {
    pub error: String,
    pub code: String,
}

// SSE connected data for chat streaming
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SSEChatStreamConnectedData {
    pub message: Option<String>,
}

// SSE data for tool call pending approval
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolCallPendingApprovalData {
    pub message_id: Uuid,
    pub tool_name: String,
    pub server_id: Uuid,
    pub description: Option<String>,
    pub arguments: serde_json::Value,
}

// SSE data for tool call complete
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolCallCompleteData {
    pub call_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

// SSE event types for chat streaming
crate::sse_event_enum! {
    #[derive(Debug, Clone, Serialize, JsonSchema)]
    pub enum SSEChatStreamEvent {
        Connected(SSEChatStreamConnectedData),
        Start(String),
        Chunk(StreamChunkData),
        Complete(StreamCompleteData),
        Error(StreamErrorData),
        EditedMessage(Message),
        CreatedBranch(crate::database::models::MessageBranch),
        ToolCallPendingApproval(ToolCallPendingApprovalData),
        ToolCallComplete(ToolCallCompleteData),
    }
}

/// Result from streaming AI response
struct StreamAIResult {
    message_id: Uuid,
    tool_call_request: Option<ToolCallRequest>,
}

/// Common streaming function for AI responses
/// Returns the saved message ID and any tool call request from the AI
async fn stream_ai_response(
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
    let ai_model = match crate::ai::model_manager::model_factory::create_ai_model(request.model_id).await {
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
        let _ = generate_and_update_conversation_title(conversation_id, user_id, &model)
            .await;
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

    // Call AI model with streaming
    match ai_model.chat_stream(SimplifiedChatRequest {
        messages,
        stream: true,
        tools,
    }).await {
        Ok(mut stream) => {
            let mut full_content = String::new();
            let mut tool_use_option: Option<crate::database::models::ToolUse> = None;

            // Process the stream
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(content) = &chunk.content {
                            full_content.push_str(content);

                            // Send chunk to client
                            let chunk_event = SSEChatStreamEvent::Chunk(StreamChunkData {
                                delta: content.to_string(),
                                message_id: None,
                            });
                            let _ = tx.send(Ok(chunk_event.into()));
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

            // Save or append the assistant message content
            let message_id = if let Some(existing_message_id) = last_assistant_message_id {
                // Resuming from previous message - append content
                match chat::append_text_content_to_message(existing_message_id, full_content.clone()).await {
                    Ok(_) => existing_message_id,
                    Err(e) => {
                        let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                            error: format!("Error appending to assistant message: {}", e),
                            code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                        });
                        let _ = tx.send(Ok(error_event.into()));
                        return Err(e.into());
                    }
                }
            } else {
                // New message - save complete message
                let assistant_message_req = SaveMessageRequest {
                    conversation_id: request.conversation_id,
                    content: full_content.clone(),
                    role: "assistant".to_string(),
                    model_id: request.model_id,
                    file_ids: None, // Assistant messages don't have file attachments
                };

                match chat::save_message(assistant_message_req, user_id, active_branch_id).await {
                    Ok(assistant_message) => assistant_message.id,
                    Err(e) => {
                        let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                            error: format!("Error saving assistant message: {}", e),
                            code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                        });
                        let _ = tx.send(Ok(error_event.into()));
                        return Err(e.into());
                    }
                }
            };

            // NOTE: Complete event is sent by the caller (send_message_stream)
            // after the tool approval loop completes, not here

            // Extract tool call request if present
            let tool_call_request = if let Some(tool_use) = tool_use_option {
                // Match the tool_name against enabled_tools to get server_id
                if let Some(enabled_tools) = &request.enabled_tools {
                    let matching_tool = enabled_tools.iter().find(|t| t.name == tool_use.name);
                    if let Some(tool) = matching_tool {
                        Some(ToolCallRequest {
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

/// Send a message with AI provider integration using SSE streaming
/// Implements main loop pattern with tool approval flow
#[debug_handler]
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async AI interaction
    tokio::spawn(async move {
        // Send initial connected event
        let connected_event = SSEChatStreamEvent::Connected(SSEChatStreamConnectedData {
            message: Some("Connected to chat stream".to_string()),
        });
        let _ = tx.send(Ok(connected_event.into()));

        // Send start event
        let start_event = SSEChatStreamEvent::Start("Chat stream started".to_string());
        let _ = tx.send(Ok(start_event.into()));

        // Get active branch ID upfront
        let _active_branch_id = match chat::get_conversation_by_id(request.conversation_id, auth_user.user.id).await {
            Ok(Some(conversation)) => conversation.active_branch_id,
            Ok(None) => {
                send_error(&tx, "Conversation not found".to_string(), ErrorCode::ResourceNotFound).await;
                return;
            }
            Err(e) => {
                send_error(&tx, format!("Error getting conversation: {}", e), ErrorCode::SystemDatabaseError).await;
                return;
            }
        };

        // ============================================
        // MAIN LOOP: Max 5 iterations
        // ============================================
        const MAX_ITERATIONS: usize = 5;
        let mut iteration = 0;
        let mut last_assistant_message_id: Option<Uuid> = None;

        while iteration < MAX_ITERATIONS {
            iteration += 1;

            // ----------------------------------------
            // 1. Check last message for pending approval
            // ----------------------------------------
            let messages = match chat::get_conversation_messages(request.conversation_id, auth_user.user.id).await {
                Ok(msgs) => msgs,
                Err(e) => {
                    send_error(&tx, format!("Failed to load messages: {}", e), ErrorCode::SystemDatabaseError).await;
                    return;
                }
            };

            let last_message = messages.last();
            let needs_approval = if let Some(msg) = last_message {
                msg.role == "assistant"
                    && msg.contents.iter().any(|c| c.content_type == MessageContentType::ToolCallPendingApproval)
            } else {
                false
            };

            if needs_approval {
                let last_msg = last_message.unwrap();

                // Find the pending approval content
                let pending_content = last_msg
                    .contents
                    .iter()
                    .find(|c| c.content_type == MessageContentType::ToolCallPendingApproval);

                if let Some(content) = pending_content {
                    // Try to parse the pending approval data
                    let tool_data: Result<MessageContentData, _> = serde_json::from_value(
                        serde_json::to_value(&content.content).unwrap_or_default()
                    );

                    if let Ok(MessageContentData::ToolCallPendingApproval { tool_name, server_id, arguments }) = tool_data {
                        // Check if approved
                        let is_approved = match chat::check_tool_approval(
                            request.conversation_id,
                            server_id,
                            &tool_name,
                        ).await {
                            Ok(approved) => approved,
                            Err(e) => {
                                send_error(&tx, format!("Failed to check approval: {}", e), ErrorCode::SystemDatabaseError).await;
                                return;
                            }
                        };

                        if !is_approved {
                            // Send approval request event and EXIT
                            let approval_event = SSEChatStreamEvent::ToolCallPendingApproval(ToolCallPendingApprovalData {
                                message_id: last_msg.id,
                                tool_name: tool_name.clone(),
                                server_id,
                                description: None, // TODO: Get from MCP server
                                arguments: arguments.clone(),
                            });
                            let _ = tx.send(Ok(approval_event.into()));
                            return;
                        }

                        // Approved! Execute tool
                        last_assistant_message_id = Some(last_msg.id);

                        if let Err(e) = execute_tool_and_save_result(
                            last_msg.id,
                            server_id,
                            &tool_name,
                            &arguments,
                            &tx,
                        ).await {
                            send_error(&tx, format!("Tool execution failed: {}", e), ErrorCode::SystemInternalError).await;
                            return;
                        }

                        // Continue to next iteration
                        continue;
                    } else {
                        send_error(&tx, "Invalid pending approval data".to_string(), ErrorCode::SystemInternalError).await;
                        return;
                    }
                }
            } else if iteration == 1 {
                // First iteration, not resuming - save user message
                let user_message_req = SaveMessageRequest {
                    conversation_id: request.conversation_id,
                    content: request.content.clone(),
                    role: "user".to_string(),
                    model_id: request.model_id,
                    file_ids: request.file_ids.clone(),
                };

                if let Err(e) = chat::save_message(user_message_req, auth_user.user.id, None).await {
                    send_error(&tx, format!("Failed to save user message: {}", e), ErrorCode::SystemDatabaseError).await;
                    return;
                }
            }

            // ----------------------------------------
            // 2. Stream AI response (saves message and returns result)
            // ----------------------------------------
            let result = match stream_ai_response(tx.clone(), request.clone(), auth_user.user.id, last_assistant_message_id).await {
                Ok(result) => result,
                Err(_) => {
                    // Error already sent by stream_ai_response
                    return;
                }
            };

            last_assistant_message_id = Some(result.message_id);

            // ----------------------------------------
            // 3. Check if AI requests tool call
            // ----------------------------------------
            if let Some(tool_request) = result.tool_call_request {
                // Save ToolCallPendingApproval to DB
                let pending_content = MessageContentData::ToolCallPendingApproval {
                    tool_name: tool_request.tool_name.clone(),
                    server_id: tool_request.server_id,
                    arguments: tool_request.arguments.clone(),
                };

                if let Err(e) = chat::save_pending_tool_approval_content(
                    result.message_id,
                    pending_content,
                ).await {
                    send_error(&tx, format!("Failed to save pending approval: {}", e), ErrorCode::SystemDatabaseError).await;
                    return;
                }

                // Continue loop - approval will be checked in next iteration
                continue;
            }

            // No tool call - we're done
            break;
        }

        if iteration >= MAX_ITERATIONS {
            send_error(&tx, "Maximum tool call iterations reached".to_string(), ErrorCode::SystemInternalError).await;
            return;
        }

        // Send complete event
        if let Some(final_message_id) = last_assistant_message_id {
            // Register model access for auto-unload tracking
            crate::ai::register_model_access(&request.model_id).await;

            // Query the final message from database to get accurate data
            match chat::get_message_by_id(final_message_id, auth_user.user.id).await {
                Ok(Some(message)) => {
                    let complete_event = SSEChatStreamEvent::Complete(StreamCompleteData {
                        message_id: message.id.to_string(),
                        conversation_id: message.conversation_id.to_string(),
                        role: message.role,
                        originated_from_id: message.originated_from_id.to_string(),
                        edit_count: message.edit_count,
                        created_at: message.created_at.to_rfc3339(),
                        updated_at: message.updated_at.to_rfc3339(),
                        total_tokens: None, // TODO: Add token tracking
                    });
                    let _ = tx.send(Ok(complete_event.into()));
                }
                Ok(None) => {
                    send_error(&tx, "Final message not found".to_string(), ErrorCode::ResourceNotFound).await;
                }
                Err(e) => {
                    send_error(&tx, format!("Error getting final message: {}", e), ErrorCode::SystemDatabaseError).await;
                }
            }
        }
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);

    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}

/// Edit a message with streaming response (creates a new branch)
#[debug_handler]
pub async fn edit_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<ChatMessageRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async message editing and AI interaction
    tokio::spawn(async move {
        // Send initial connected event
        let connected_event = SSEChatStreamEvent::Connected(SSEChatStreamConnectedData {
            message: Some("Connected to edit stream".to_string()),
        });
        let _ = tx.send(Ok(connected_event.into()));

        // Send start event
        let start_event = SSEChatStreamEvent::Start("Edit stream started".to_string());
        let _ = tx.send(Ok(start_event.into()));

        let edit_message = EditMessageRequest {
            content: request.content.clone(),
            file_ids: request.file_ids.clone(),
        };

        // Edit the message first
        match chat::edit_message(message_id, edit_message, auth_user.user.id).await {
            Ok(Some(edit_response)) => {
                // send the edited message as a data event
                let edited_message_event = SSEChatStreamEvent::EditedMessage(edit_response.message);
                let _ = tx.send(Ok(edited_message_event.into()));
                //send the created branch as a data event
                let created_branch_event = SSEChatStreamEvent::CreatedBranch(edit_response.branch);
                let _ = tx.send(Ok(created_branch_event.into()));
            }
            Ok(None) => {
                let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                    error: "Message not found".to_string(),
                    code: ErrorCode::ResourceNotFound.as_str().to_string(),
                });
                let _ = tx.send(Ok(error_event.into()));
                return;
            }
            Err(e) => {
                let error_event = SSEChatStreamEvent::Error(StreamErrorData {
                    error: format!("Error editing message: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                });
                let _ = tx.send(Ok(error_event.into()));
                return;
            }
        }

        let _ = stream_ai_response(tx, request, auth_user.user.id, None).await;
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);
    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}

/// Get message branches for a specific message (all branches containing messages with same originated_from_id)
#[debug_handler]
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> ApiResult<Json<Vec<crate::database::models::MessageBranch>>> {
    match chat::get_message_branches(message_id, auth_user.user.id).await {
        Ok(branches) => Ok((StatusCode::OK, Json(branches))),
        Err(e) => {
            eprintln!("Error getting message branches: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Get messages for a conversation with specific branch
#[debug_handler]
pub async fn get_conversation_messages_by_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Vec<Message>>> {
    match chat::get_conversation_messages_by_branch(conversation_id, branch_id, auth_user.user.id)
        .await
    {
        Ok(messages) => Ok((StatusCode::OK, Json(messages))),
        Err(e) => {
            eprintln!("Error getting messages for branch: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Generate and update conversation title using AI model
async fn generate_and_update_conversation_title(
    conversation_id: Uuid,
    user_id: Uuid,
    model: &crate::database::models::Model,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the first user message from the conversation
    let messages = chat::get_conversation_messages(conversation_id, user_id).await?;

    // Find the first user message
    let first_user_message = messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.get_text_content());

    if let Some(user_content) = first_user_message {
        // Create a title generation prompt
        let title_prompt = format!(
      "Generate a concise, descriptive title (maximum 6 words) for a conversation that starts with this message: \"{}\"\n\nRespond with only the title, no quotes or additional text.",
      user_content.chars().take(200).collect::<String>()
    );

        let chat_messages = build_single_user_message(title_prompt);

        // Create AI model instance
        let ai_model = crate::ai::model_manager::model_factory::create_ai_model(model.id).await?;

        // Call AI model to generate title
        // Note: Title-specific parameters would ideally be configured in the model instance
        match ai_model.chat(SimplifiedChatRequest {
            messages: chat_messages,
            stream: false,
            tools: None, // Don't use tools for title generation
        }).await {
            Ok(response) => {
                let generated_title = response.content.trim().to_string();

                // Clean up the title (remove quotes, limit length)
                let clean_title = generated_title
                    .trim_matches('"')
                    .trim_matches('\'')
                    .chars()
                    .take(50)
                    .collect::<String>();

                // Update the conversation title
                let update_request = UpdateConversationRequest {
                    title: Some(clean_title),
                    assistant_id: None,
                    model_id: None,
                };

                if let Err(e) =
                    chat::update_conversation(conversation_id, update_request, user_id).await
                {
                    eprintln!("Error updating conversation title: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error generating title with AI: {}", e);
                // Fallback to simple title generation
                if let Err(e) = chat::auto_update_conversation_title(conversation_id, user_id).await
                {
                    eprintln!("Error with fallback title generation: {}", e);
                }
            }
        }
    } else {
        // No user message found, use fallback
        if let Err(e) = chat::auto_update_conversation_title(conversation_id, user_id).await {
            eprintln!("Error with fallback title generation: {}", e);
        }
    }

    Ok(())
}

// ============================================
// Tool Approval Helper Functions
// ============================================

/// Execute MCP tool and save result to DB (MOCK EXECUTION FOR NOW)
async fn execute_tool_and_save_result(
    message_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
    arguments: &serde_json::Value,
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // MOCK EXECUTION - Return mock result
    // TODO: Replace with actual MCP tool execution
    let call_id = Uuid::new_v4().to_string();
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

    // Save to database using query function
    chat::save_tool_result_content(message_id, result_content).await?;

    // Send SSE event
    let event = SSEChatStreamEvent::ToolCallComplete(ToolCallCompleteData {
        call_id,
        success: true,
        result: Some(mock_result),
        error_message: None,
    });
    let _ = tx.send(Ok(event.into()));

    Ok(())
}

/// Send error SSE event
async fn send_error(
    tx: &tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
    error_message: String,
    error_code: ErrorCode,
) {
    let error_event = SSEChatStreamEvent::Error(StreamErrorData {
        error: error_message,
        code: error_code.as_str().to_string(),
    });
    let _ = tx.send(Ok(error_event.into()));
}

/// Struct to hold tool call request from AI
struct ToolCallRequest {
    server_id: Uuid,
    tool_name: String,
    arguments: serde_json::Value,
}