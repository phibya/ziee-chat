use axum::response::sse::{Event, KeepAlive};
use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    response::Sse,
    Extension, Json,
};
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

use crate::ai::core::ChatRequest;
use crate::api::errors::{ApiResult2, AppError, ErrorCode};
use crate::api::middleware::AuthenticatedUser;
use crate::database::models::EditMessageRequest;
use crate::database::{
    models::{
        Conversation, ConversationListResponse, CreateConversationRequest, Message,
        SaveMessageRequest, UpdateConversationRequest,
    },
    queries::{
        assistants::get_assistant_by_id,
        chat,
        models::{get_model_by_id, get_provider_by_model_id},
    },
};
use crate::types::ConversationPaginationQuery;
use crate::utils::chat::{build_chat_messages, build_single_user_message};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchQuery {
    q: String,
    page: Option<i32>,
    per_page: Option<i32>,
    project_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChatMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub model_id: Uuid,
    pub assistant_id: Uuid,
    pub file_ids: Option<Vec<Uuid>>, // Optional file attachments
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchBranchRequest {
    pub branch_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub r#type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct OperationSuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StreamChunkData {
    pub delta: String,
    pub message_id: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct StreamErrorData {
    pub error: String,
    pub code: String,
}

/// Create a new conversation
#[debug_handler]
pub async fn create_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateConversationRequest>,
) -> ApiResult2<Json<Conversation>> {
    match chat::create_conversation(request, auth_user.user.id).await {
        Ok(conversation) => Ok((StatusCode::OK, Json(conversation))),
        Err(e) => {
            eprintln!("Error creating conversation: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Get conversation by ID (without messages)
#[debug_handler]
pub async fn get_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
) -> ApiResult2<Json<Conversation>> {
    match chat::get_conversation_by_id(conversation_id, auth_user.user.id).await {
        Ok(Some(conversation)) => Ok((StatusCode::OK, Json(conversation))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Conversation"))),
        Err(e) => {
            eprintln!("Error getting conversation: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// List conversations for the authenticated user
#[debug_handler]
pub async fn list_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<ConversationPaginationQuery>,
) -> ApiResult2<Json<ConversationListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    let project_id = params
        .project_id
        .as_deref()
        .map(|s| Uuid::parse_str(s).ok())
        .flatten();
    match chat::list_conversations(auth_user.user.id, page, per_page, project_id).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error listing conversations: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Update conversation
#[debug_handler]
pub async fn update_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<UpdateConversationRequest>,
) -> ApiResult2<Json<Conversation>> {
    match chat::update_conversation(conversation_id, request, auth_user.user.id).await {
        Ok(Some(conversation)) => Ok((StatusCode::OK, Json(conversation))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("Conversation"))),
        Err(e) => {
            eprintln!("Error updating conversation: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Delete conversation
#[debug_handler]
pub async fn delete_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    match chat::delete_conversation(conversation_id, auth_user.user.id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("Conversation"))),
        Err(e) => {
            eprintln!("Error deleting conversation: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Common streaming function for AI responses
async fn stream_ai_response(
    tx: tokio::sync::mpsc::UnboundedSender<Result<Event, Infallible>>,
    request: ChatMessageRequest,
    user_id: Uuid,
) {
    // IMPORTANT: Capture the conversation's active branch immediately to prevent
    // race conditions if the user switches branches during streaming
    let active_branch_id =
        match chat::get_conversation_by_id(request.conversation_id, user_id).await {
            Ok(Some(conversation)) => conversation.active_branch_id,
            Ok(None) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: "Conversation not found".to_string(),
                        code: ErrorCode::ResourceNotFound.as_str().to_string(),
                    })
                    .unwrap_or_default(),
                )));
                return;
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: format!("Error getting conversation: {}", e),
                        code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                    })
                    .unwrap_or_default(),
                )));
                return;
            }
        };

    // Get the model provider configuration directly from model_id
    let provider = match get_provider_by_model_id(request.model_id).await {
        Ok(Some(provider)) => {
            println!("DEBUG: Found provider: {:?}", provider.name);
            provider
        }
        Ok(None) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: "Model or provider not found".to_string(),
                    code: ErrorCode::ResourceModelNotFound.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }
        Err(e) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error getting model provider: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }
    };

    // Check if provider is enabled
    if !provider.enabled {
        let _ = tx.send(Ok(Event::default().event("error").data(
            &serde_json::to_string(&StreamErrorData {
                error: "Provider is disabled".to_string(),
                code: ErrorCode::ResourceProviderDisabled.as_str().to_string(),
            })
            .unwrap_or_default(),
        )));
        return;
    }

    // Get the model to get the actual model name
    let model = match get_model_by_id(request.model_id).await {
        Ok(Some(model)) => {
            println!("DEBUG: Found model: {:?}", model.name);
            model
        }
        Ok(None) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: "Model not found".to_string(),
                    code: ErrorCode::ResourceModelNotFound.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }
        Err(e) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error getting model: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }
    };

    // Build chat messages for AI provider using utility function
    let messages = match build_chat_messages(&request, user_id).await {
        Ok(messages) => messages,
        Err(e) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error building chat messages: {}", e),
                    code: ErrorCode::SystemInternalError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }
    };

    // Get assistant parameters for model configuration
    let assistant_params = if let Ok(Some(assistant)) =
        get_assistant_by_id(request.assistant_id, Some(user_id)).await
    {
        assistant.parameters.clone()
    } else {
        None
    };

    // Check if this is a new conversation (count messages before moving them)
    // Count messages excluding system messages (assistant instructions)
    let user_and_assistant_messages = messages.iter().filter(|m| m.role != "system").count();

    // Create AI provider with model ID for Candle providers
    let ai_provider =
        match crate::ai::model_manager::create_ai_provider_with_model_id(&provider, Some(request.model_id)).await {
            Ok(provider) => provider,
            Err(e) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: format!("Error creating AI provider: {}", e),
                        code: ErrorCode::SystemInternalError.as_str().to_string(),
                    })
                    .unwrap_or_default(),
                )));
                return;
            }
        };

    // Check if provider supports streaming
    if !ai_provider.supports_streaming() {
        let _ = tx.send(Ok(Event::default().event("error").data(
            &serde_json::to_string(&StreamErrorData {
                error: "Provider does not support streaming responses".to_string(),
                code: ErrorCode::SystemInternalError.as_str().to_string(),
            })
            .unwrap_or_default(),
        )));
        return;
    }

    // Merge parameters from model and assistant configurations
    let (temperature, max_tokens, top_p, frequency_penalty, presence_penalty) =
        merge_parameters(&model.parameters, &assistant_params);

    // Create ModelParameters from the merged values
    let parameters = crate::database::models::model::ModelParameters {
        temperature,
        max_tokens,
        top_p,
        frequency_penalty,
        presence_penalty,
        ..Default::default()
    };

    // Create chat request
    let chat_request = ChatRequest {
        messages,
        model_name: model.name.clone(),
        model_id: model.id,
        provider_id: provider.id,
        stream: true,
        parameters: Some(parameters),
    };

    // If there's only 1 message (the user message we just added), this is a new conversation
    // Generate title before streaming the response
    if user_and_assistant_messages == 1 {
        let conversation_id = request.conversation_id;

        // Wait for title generation to complete before streaming the response
        let _ = generate_and_update_conversation_title(conversation_id, user_id, &provider, &model)
            .await;
    }

    // Call AI provider with streaming
    match ai_provider.chat_stream(chat_request).await {
        Ok(mut stream) => {
            let mut full_content = String::new();

            // Process the stream
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(content) = chunk.content {
                            full_content.push_str(&content);

                            // Send chunk to client
                            let _ = tx.send(Ok(Event::default().event("chunk").data(
                                &serde_json::to_string(&StreamChunkData {
                                    delta: content,
                                    message_id: None,
                                })
                                .unwrap_or_default(),
                            )));
                        }

                        // Check if streaming is complete
                        if chunk.finish_reason.is_some() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Ok(Event::default().event("error").data(
                            &serde_json::to_string(&StreamErrorData {
                                error: format!("Streaming error: {}", e),
                                code: ErrorCode::SystemStreamingError.as_str().to_string(),
                            })
                            .unwrap_or_default(),
                        )));
                        return;
                    }
                }
            }

            // Save the complete assistant message
            let assistant_message_req = SaveMessageRequest {
                conversation_id: request.conversation_id,
                content: full_content.clone(),
                role: "assistant".to_string(),
                model_id: request.model_id,
                file_ids: None, // Assistant messages don't have file attachments
            };

            match chat::save_message(assistant_message_req, user_id, Some(active_branch_id)).await {
                Ok(assistant_message) => {
                    // Register model access for auto-unload tracking on successful completion
                    crate::ai::register_model_access(&request.model_id).await;

                    // Send completion event
                    let _ = tx.send(Ok(Event::default().event("complete").data(
                        &serde_json::to_string(&StreamCompleteData {
                            message_id: assistant_message.id.to_string(),
                            conversation_id: request.conversation_id.to_string(),
                            role: assistant_message.role.clone(),
                            originated_from_id: assistant_message.originated_from_id.to_string(),
                            edit_count: assistant_message.edit_count,
                            created_at: assistant_message.created_at.to_rfc3339(),
                            updated_at: assistant_message.updated_at.to_rfc3339(),
                            total_tokens: None, // Token usage not available in streaming mode
                        })
                        .unwrap_or_default(),
                    )));
                }
                Err(e) => {
                    let _ = tx.send(Ok(Event::default().event("error").data(
                        &serde_json::to_string(&StreamErrorData {
                            error: format!("Error saving assistant message: {}", e),
                            code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                        })
                        .unwrap_or_default(),
                    )));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error calling AI provider: {}", e),
                    code: ErrorCode::SystemExternalServiceError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
        }
    }
}

/// Send a message with AI provider integration using SSE streaming
#[debug_handler]
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> ApiResult2<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async AI interaction
    tokio::spawn(async move {
        // Send initial event
        let _ = tx.send(Ok(Event::default().data("start")));

        // First save the user message
        let user_message_req = SaveMessageRequest {
            conversation_id: request.conversation_id,
            content: request.content.clone(),
            role: "user".to_string(),
            model_id: request.model_id,
            file_ids: request.file_ids.clone(),
        };

        if let Err(e) = chat::save_message(user_message_req, auth_user.user.id, None).await {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error saving user message: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
        }

        stream_ai_response(tx, request, auth_user.user.id).await;
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
) -> ApiResult2<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async message editing and AI interaction
    tokio::spawn(async move {
        // Send initial event
        let _ = tx.send(Ok(Event::default().data("start")));
        let edit_message = EditMessageRequest {
            content: request.content.clone(),
            file_ids: request.file_ids.clone(),
        };

        // Edit the message first
        match chat::edit_message(message_id, edit_message, auth_user.user.id).await {
            Ok(Some(edit_response)) => {
                // send the edited message as a data event
                let _ = tx.send(Ok(Event::default().event("edited-message").data(
                    &serde_json::to_string(&edit_response.message).unwrap_or_default(),
                )));
                //send the created branch as a data event
                let _ = tx.send(Ok(Event::default().event("created-branch").data(
                    &serde_json::to_string(&edit_response.branch).unwrap_or_default(),
                )));
            }
            Ok(None) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: "Message not found".to_string(),
                        code: ErrorCode::ResourceNotFound.as_str().to_string(),
                    })
                    .unwrap_or_default(),
                )));
                return;
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: format!("Error editing message: {}", e),
                        code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                    })
                    .unwrap_or_default(),
                )));
                return;
            }
        }

        stream_ai_response(tx, request, auth_user.user.id).await;
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);
    Ok((
        StatusCode::OK,
        Sse::new(stream).keep_alive(KeepAlive::default()),
    ))
}


/// Switch to a different branch for a conversation
#[debug_handler]
pub async fn switch_conversation_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<SwitchBranchRequest>,
) -> ApiResult2<Json<OperationSuccessResponse>> {
    match chat::switch_conversation_branch(conversation_id, request.branch_id, auth_user.user.id)
        .await
    {
        Ok(true) => Ok((
            StatusCode::OK,
            Json(OperationSuccessResponse {
                success: true,
                message: "Branch switched successfully".to_string(),
            }),
        )),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Conversation branch"),
        )),
        Err(e) => {
            eprintln!("Error switching conversation branch: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Get message branches for a specific message (all branches containing messages with same originated_from_id)
#[debug_handler]
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> ApiResult2<Json<Vec<crate::database::models::MessageBranch>>> {
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

/// Search conversations
#[debug_handler]
pub async fn search_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<SearchQuery>,
) -> ApiResult2<Json<ConversationListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    let project_id = params
        .project_id
        .as_deref()
        .map(|s| Uuid::parse_str(s).ok())
        .flatten();
    match chat::search_conversations(auth_user.user.id, &params.q, page, per_page, project_id).await
    {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error searching conversations: {}", e);
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
) -> ApiResult2<Json<Vec<Message>>> {
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

/// Generate and update conversation title using AI provider
async fn generate_and_update_conversation_title(
    conversation_id: Uuid,
    user_id: Uuid,
    provider: &crate::database::models::Provider,
    model: &crate::database::models::Model,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get the first user message from the conversation
    let messages = chat::get_conversation_messages(conversation_id, user_id).await?;

    // Find the first user message
    let first_user_message = messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.content.clone());

    if let Some(user_content) = first_user_message {
        // Create a title generation prompt
        let title_prompt = format!(
      "Generate a concise, descriptive title (maximum 6 words) for a conversation that starts with this message: \"{}\"\n\nRespond with only the title, no quotes or additional text.",
      user_content.chars().take(200).collect::<String>()
    );

        let chat_messages = build_single_user_message(title_prompt);

        // Create AI provider instance
        let ai_provider = crate::ai::model_manager::create_ai_provider_with_model_id(provider, Some(model.id)).await?;

        // For title generation, use specific parameters optimized for titles
        // Don't use assistant parameters as this is an internal system function
        let title_parameters = crate::database::models::model::ModelParameters {
            temperature: Some(0.3), // Lower temperature for more consistent titles
            max_tokens: Some(20),   // Short titles only
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            ..Default::default()
        };

        let chat_request = ChatRequest {
            messages: chat_messages,
            model_name: model.name.clone(),
            model_id: model.id,
            provider_id: provider.id,
            stream: false,
            parameters: Some(title_parameters),
        };

        // Call AI provider to generate title
        match ai_provider.chat(chat_request).await {
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

/// Merge model and assistant parameters with assistant parameters taking priority
/// Only include parameters that are actually defined (not null)
fn merge_parameters(
    model_params: &Option<crate::database::models::ModelParameters>,
    assistant_params: &Option<crate::database::models::ModelParameters>,
) -> (
    Option<f32>, // temperature
    Option<u32>, // max_tokens
    Option<f32>, // top_p
    Option<f32>, // frequency_penalty
    Option<f32>, // presence_penalty
) {
    let mut temperature = None;
    let mut max_tokens = None;
    let mut top_p = None;
    let mut frequency_penalty = None;
    let mut presence_penalty = None;

    // First, extract from model parameters
    if let Some(model_params) = model_params {
        if let Some(temp) = model_params.temperature {
            temperature = Some(temp);
        }
        if let Some(m_tokens) = model_params.max_tokens {
            max_tokens = Some(m_tokens);
        }
        if let Some(top_p_val) = model_params.top_p {
            top_p = Some(top_p_val);
        }
        if let Some(freq_pen) = model_params.frequency_penalty {
            frequency_penalty = Some(freq_pen);
        }
        if let Some(pres_pen) = model_params.presence_penalty {
            presence_penalty = Some(pres_pen);
        }
    }

    // Then, override with assistant parameters (higher priority)
    if let Some(assistant_params) = assistant_params {
        if let Some(temp) = assistant_params.temperature {
            temperature = Some(temp);
        }
        if let Some(max_tok) = assistant_params.max_tokens {
            max_tokens = Some(max_tok);
        }
        if let Some(top_p_val) = assistant_params.top_p {
            top_p = Some(top_p_val);
        }
        if let Some(freq_pen) = assistant_params.frequency_penalty {
            frequency_penalty = Some(freq_pen);
        }
        if let Some(pres_pen) = assistant_params.presence_penalty {
            presence_penalty = Some(pres_pen);
        }
    }

    (
        temperature,
        max_tokens,
        top_p,
        frequency_penalty,
        presence_penalty,
    )
}
