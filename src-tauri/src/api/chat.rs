use axum::response::sse::{Event, KeepAlive};
use axum::{
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

use crate::ai::{
    anthropic::AnthropicProvider,
    openai::OpenAIProvider,
    providers::{AIProvider, ChatMessage, ChatRequest, ProxyConfig},
};
use crate::api::errors::ErrorCode;
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        Conversation, ConversationListResponse, CreateConversationRequest, EditMessageRequest,
        Message, SendMessageRequest, UpdateConversationRequest,
    },
    queries::{
        assistants::get_assistant_by_id,
        chat,
        model_providers::{get_model_by_id, get_model_provider_by_id},
    },
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    page: Option<i32>,
    per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub parent_id: Option<Uuid>,
    pub model_provider_id: Uuid,
    pub model_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub r#type: String,
    pub data: serde_json::Value,
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
    pub total_tokens: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct StreamErrorData {
    pub error: String,
    pub code: String,
}

/// Create a new conversation
pub async fn create_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    match chat::create_conversation(request, auth_user.user.id).await {
        Ok(conversation) => Ok(Json(conversation)),
        Err(e) => {
            eprintln!("Error creating conversation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get conversation by ID with messages
pub async fn get_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match chat::get_conversation_by_id(conversation_id, auth_user.user.id).await {
        Ok(Some(conversation)) => {
            // Get messages for this conversation
            let messages =
                match chat::get_conversation_messages(conversation_id, auth_user.user.id).await {
                    Ok(messages) => messages,
                    Err(e) => {
                        eprintln!("Error getting messages: {}", e);
                        vec![]
                    }
                };

            let response = serde_json::json!({
                "conversation": conversation,
                "messages": messages
            });
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting conversation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List conversations for the authenticated user
pub async fn list_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<ConversationListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match chat::list_conversations(auth_user.user.id, page, per_page).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error listing conversations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update conversation
pub async fn update_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<UpdateConversationRequest>,
) -> Result<Json<Conversation>, StatusCode> {
    match chat::update_conversation(conversation_id, request, auth_user.user.id).await {
        Ok(Some(conversation)) => Ok(Json(conversation)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating conversation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete conversation
pub async fn delete_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match chat::delete_conversation(conversation_id, auth_user.user.id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting conversation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Send a message with AI provider integration using SSE streaming
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Create a channel for streaming events
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to handle the async AI interaction
    tokio::spawn(async move {
        // Send initial event
        let _ = tx.send(Ok(Event::default().data("start")));

        // Get the model provider configuration
        let provider = match get_model_provider_by_id(request.model_provider_id).await {
            Ok(Some(provider)) => {
                println!("DEBUG: Found provider: {:?}", provider.name);
                provider
            }
            Ok(None) => {
                let _ = tx.send(Ok(Event::default().event("error").data(
                    &serde_json::to_string(&StreamErrorData {
                        error: "Provider not found".to_string(),
                        code: ErrorCode::ResourceProviderNotFound.as_str().to_string(),
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

        // Get the assistant for instructions if available
        let conversation =
            match chat::get_conversation_by_id(request.conversation_id, auth_user.user.id).await {
                Ok(Some(conv)) => conv,
                Ok(None) => {
                    let _ = tx.send(Ok(Event::default().event("error").data(
                        &serde_json::to_string(&StreamErrorData {
                            error: "Conversation not found".to_string(),
                            code: ErrorCode::ResourceConversationNotFound.as_str().to_string(),
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

        // Build chat messages for AI provider
        let mut messages = Vec::new();

        // Add system message from assistant if available
        if let Some(assistant_id) = conversation.assistant_id {
            if let Ok(Some(assistant)) =
                get_assistant_by_id(assistant_id, Some(auth_user.user.id)).await
            {
                if let Some(instructions) = assistant.instructions {
                    if !instructions.trim().is_empty() {
                        messages.push(ChatMessage {
                            role: "system".to_string(),
                            content: instructions,
                        });
                    }
                }
            }
        }

        // Get conversation history and add it to messages
        match chat::get_conversation_messages(request.conversation_id, auth_user.user.id).await {
            Ok(conversation_messages) => {
                // Add each message from the conversation history
                for msg in conversation_messages {
                    messages.push(ChatMessage {
                        role: msg.role,
                        content: msg.content,
                    });
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not load conversation history: {}", e);
                // Continue without history rather than failing completely
            }
        }

        // Add the current user's message
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: request.content.clone(),
        });

        // Create AI provider
        let ai_provider = match create_ai_provider(&provider) {
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

        // Create chat request
        let chat_request = ChatRequest {
            messages,
            model: model.name.clone(),
            stream: true,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            top_p: Some(0.95),
            frequency_penalty: None,
            presence_penalty: None,
        };

        // First save the user message
        let user_message_req = SendMessageRequest {
            conversation_id: request.conversation_id,
            content: request.content.clone(),
            role: "user".to_string(),
            parent_id: request.parent_id,
            model_provider_id: request.model_provider_id,
            model_id: request.model_id,
        };

        if let Err(e) = chat::send_message(user_message_req, auth_user.user.id).await {
            let _ = tx.send(Ok(Event::default().event("error").data(
                &serde_json::to_string(&StreamErrorData {
                    error: format!("Error saving user message: {}", e),
                    code: ErrorCode::SystemDatabaseError.as_str().to_string(),
                })
                .unwrap_or_default(),
            )));
            return;
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
                let assistant_message_req = SendMessageRequest {
                    conversation_id: request.conversation_id,
                    content: full_content.clone(),
                    role: "assistant".to_string(),
                    parent_id: None,
                    model_provider_id: request.model_provider_id,
                    model_id: request.model_id,
                };

                match chat::send_message(assistant_message_req, auth_user.user.id).await {
                    Ok(assistant_message) => {
                        // Update conversation title if this is a new conversation (only 1 message - the user message)
                        let message_count = match chat::count_conversation_messages(
                            request.conversation_id,
                            auth_user.user.id,
                        )
                        .await
                        {
                            Ok(count) => count,
                            Err(_) => 0,
                        };

                        // If there's only 1 message (the user message we just saved), this is a new conversation
                        if message_count == 1 {
                            let _ = generate_and_update_conversation_title(
                                request.conversation_id,
                                auth_user.user.id,
                                &provider,
                                &model,
                            )
                            .await;
                        }

                        // Send completion event
                        let _ = tx.send(Ok(Event::default().event("complete").data(
                            &serde_json::to_string(&StreamCompleteData {
                                message_id: assistant_message.id.to_string(),
                                conversation_id: request.conversation_id.to_string(),
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
    });

    // Convert the receiver to a stream and return as SSE
    let stream = UnboundedReceiverStream::new(rx);

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Helper function to get AI response for an edited message
async fn get_ai_response_for_edited_message(
    conversation_history: Vec<Message>,
    edited_content: String,
    conversation: Conversation,
    provider_id: Uuid,
    model_id: Uuid,
    user_id: Uuid,
) -> Result<(), StatusCode> {
    // Get the model provider configuration
    let provider = match get_model_provider_by_id(provider_id).await {
        Ok(Some(provider)) => provider,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Get the model
    let model = match get_model_by_id(model_id).await {
        Ok(Some(model)) => model,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Build chat messages for AI provider
    let mut messages = Vec::new();

    // Add system message from assistant if available
    if let Some(assistant_id) = conversation.assistant_id {
        if let Ok(Some(assistant)) = get_assistant_by_id(assistant_id, Some(user_id)).await {
            if let Some(instructions) = assistant.instructions {
                if !instructions.trim().is_empty() {
                    messages.push(ChatMessage {
                        role: "system".to_string(),
                        content: instructions,
                    });
                }
            }
        }
    }

    // Add conversation history
    for msg in conversation_history {
        messages.push(ChatMessage {
            role: msg.role,
            content: msg.content,
        });
    }

    // Add the edited user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: edited_content,
    });

    // Create AI provider instance
    let ai_provider =
        create_ai_provider(&provider).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create chat request
    let chat_request = ChatRequest {
        messages,
        model: model.name.clone(),
        stream: true,
        temperature: Some(0.7),
        max_tokens: Some(4096),
        top_p: Some(0.95),
        frequency_penalty: None,
        presence_penalty: None,
    };

    // Call AI provider and get response
    match ai_provider.chat(chat_request).await {
        Ok(response) => {
            // Store only the assistant response (user message already stored by edit_message)
            let assistant_message_req = SendMessageRequest {
                conversation_id: conversation.id,
                content: response.content,
                role: "assistant".to_string(),
                parent_id: None,
                model_provider_id: provider_id,
                model_id: model_id,
            };

            if let Err(e) = chat::send_message(assistant_message_req, user_id).await {
                eprintln!("Error saving assistant message: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Error calling AI provider: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Edit a message (creates a new branch)
pub async fn edit_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    match chat::edit_message(message_id, request.clone(), auth_user.user.id).await {
        Ok(Some(edit_response)) => {
            // If content changed, automatically send the edited message to AI and get response
            if edit_response.content_changed {
                // Get conversation details to get the model provider and model IDs
                if let Ok(Some(conversation)) = chat::get_conversation_by_id(
                    edit_response.message.conversation_id,
                    auth_user.user.id,
                )
                .await
                {
                    // Only proceed if the conversation has model provider and model configured
                    if let (Some(provider_id), Some(model_id)) =
                        (conversation.model_provider_id, conversation.model_id)
                    {
                        // Get AI response for the edited message (don't store user message again)
                        if let Err(e) = get_ai_response_for_edited_message(
                            edit_response.conversation_history,
                            edit_response.message.content.clone(),
                            conversation,
                            provider_id,
                            model_id,
                            auth_user.user.id,
                        )
                        .await
                        {
                            eprintln!(
                                "Warning: Failed to get AI response for edited message: {:?}",
                                e
                            );
                        }
                    }
                }
            }

            Ok(Json(edit_response.message))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error editing message: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Switch to a different branch
pub async fn switch_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Message>, StatusCode> {
    match chat::switch_message_branch(message_id, auth_user.user.id).await {
        Ok(Some(message)) => Ok(Json(message)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error switching branch: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get message branches for a specific message (all messages with same originated_from_id)
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    use sqlx::Row;

    // Get the database pool
    let pool = match crate::database::queries::get_database_pool() {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Error getting database pool: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the message to find its conversation and created_at
    let message_row = match sqlx::query(
        r#"
        SELECT m.*, c.user_id as conversation_user_id
        FROM messages m
        JOIN conversations c ON m.conversation_id = c.id
        WHERE m.id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting message: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let conversation_user_id: Uuid = message_row.get("conversation_user_id");
    if conversation_user_id != auth_user.user.id {
        return Err(StatusCode::NOT_FOUND);
    }

    let conversation_id: Uuid = message_row.get("conversation_id");
    let created_at: chrono::DateTime<chrono::Utc> = message_row.get("created_at");

    match chat::get_message_branches(conversation_id, created_at, auth_user.user.id).await {
        Ok(branches) => Ok(Json(branches)),
        Err(e) => {
            eprintln!("Error getting message branches: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Search conversations
pub async fn search_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ConversationListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match chat::search_conversations(auth_user.user.id, &params.q, page, per_page).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error searching conversations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Helper function to create proxy configuration from model provider settings
fn create_proxy_config(
    proxy_settings: &crate::database::models::ModelProviderProxySettings,
) -> Option<ProxyConfig> {
    if proxy_settings.enabled {
        Some(ProxyConfig {
            enabled: proxy_settings.enabled,
            url: proxy_settings.url.clone(),
            username: if proxy_settings.username.is_empty() {
                None
            } else {
                Some(proxy_settings.username.clone())
            },
            password: if proxy_settings.password.is_empty() {
                None
            } else {
                Some(proxy_settings.password.clone())
            },
            no_proxy: proxy_settings
                .no_proxy
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            ignore_ssl_certificates: proxy_settings.ignore_ssl_certificates,
        })
    } else {
        None
    }
}

/// Helper function to create AI provider instances
fn create_ai_provider(
    provider: &crate::database::models::ModelProvider,
) -> Result<Box<dyn AIProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let proxy_config = provider
        .proxy_settings
        .as_ref()
        .and_then(create_proxy_config);

    match provider.provider_type.as_str() {
        "openai" => {
            let openai_provider = OpenAIProvider::new(
                provider.api_key.as_ref().unwrap_or(&String::new()).clone(),
                provider.base_url.clone(),
                proxy_config,
            )?;
            Ok(Box::new(openai_provider))
        }
        "anthropic" => {
            let anthropic_provider = AnthropicProvider::new(
                provider.api_key.as_ref().unwrap_or(&String::new()).clone(),
                provider.base_url.clone(),
                proxy_config,
            )?;
            Ok(Box::new(anthropic_provider))
        }
        _ => Err(format!("Unsupported provider type: {}", provider.provider_type).into()),
    }
}

/// Clear all chat history for the authenticated user
pub async fn clear_all_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match chat::delete_all_conversations(auth_user.user.id).await {
        Ok(deleted_count) => Ok(Json(serde_json::json!({
            "deleted_count": deleted_count,
            "message": "All conversations deleted successfully"
        }))),
        Err(e) => {
            eprintln!("Error clearing all conversations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate and update conversation title using AI provider
async fn generate_and_update_conversation_title(
    conversation_id: Uuid,
    user_id: Uuid,
    provider: &crate::database::models::ModelProvider,
    model: &crate::database::models::ModelProviderModel,
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

        let chat_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: title_prompt,
        }];

        // Create AI provider instance
        let ai_provider = create_ai_provider(provider)?;

        // Create chat request for title generation
        let chat_request = ChatRequest {
            messages: chat_messages,
            model: model.name.clone(),
            stream: false,
            temperature: Some(0.3), // Lower temperature for more consistent titles
            max_tokens: Some(20),   // Short titles only
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
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
                    model_provider_id: None,
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
