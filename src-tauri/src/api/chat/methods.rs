use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use futures_util::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai::{
    anthropic::AnthropicProvider,
    openai::OpenAIProvider,
    providers::{AIProvider, ChatMessage, ChatRequest, ProxyConfig},
};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        Conversation, ConversationListResponse, CreateConversationRequest, EditMessageRequest,
        Message, SendMessageRequest, UpdateConversationRequest,
    },
    queries::{assistants::get_assistant_by_id, chat, model_providers::{get_model_provider_by_id, get_model_by_id}},
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

/// Send a message with AI provider integration
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!(
        "DEBUG: send_message_stream called with request: {:?}",
        request
    );
    // Get the model provider configuration
    let provider = match get_model_provider_by_id(request.model_provider_id).await {
        Ok(Some(provider)) => {
            println!("DEBUG: Found provider: {:?}", provider.name);
            provider
        }
        Ok(None) => {
            println!(
                "DEBUG: Provider not found for ID: {}",
                request.model_provider_id
            );
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            eprintln!("Error getting model provider: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if provider is enabled
    if !provider.enabled {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get the model to get the actual model name
    let model = match get_model_by_id(request.model_id).await {
        Ok(Some(model)) => {
            println!("DEBUG: Found model: {:?}", model.name);
            model
        },
        Ok(None) => {
            println!("DEBUG: Model not found for ID: {}", request.model_id);
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            eprintln!("Error getting model: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the assistant for instructions if available
    let conversation =
        match chat::get_conversation_by_id(request.conversation_id, auth_user.user.id).await {
            Ok(Some(conv)) => conv,
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error getting conversation: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
    let ai_provider: Box<dyn AIProvider> = match provider.provider_type.as_str() {
        "openai" => {
            let proxy_config = if let Some(proxy_settings) = &provider.proxy_settings {
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
            };

            match OpenAIProvider::new(
                provider.api_key.unwrap_or_default(),
                provider.base_url,
                proxy_config,
            ) {
                Ok(provider) => Box::new(provider),
                Err(e) => {
                    eprintln!("Error creating OpenAI provider: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        "anthropic" => {
            let proxy_config = if let Some(proxy_settings) = &provider.proxy_settings {
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
            };

            match AnthropicProvider::new(
                provider.api_key.unwrap_or_default(),
                provider.base_url,
                proxy_config,
            ) {
                Ok(provider) => Box::new(provider),
                Err(e) => {
                    eprintln!("Error creating Anthropic provider: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        _ => {
            eprintln!("Unsupported provider type: {}", provider.provider_type);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create chat request
    let chat_request = ChatRequest {
        messages,
        model: model.name.clone(), // The name field should contain the provider model ID (e.g., "claude-3-sonnet-20240229")
        stream: true,                        // Enable streaming
        temperature: Some(0.7),
        max_tokens: Some(4096),
        top_p: Some(0.95),
        frequency_penalty: None,
        presence_penalty: None,
    };

    // Call AI provider
    match ai_provider.chat(chat_request).await {
        Ok(response) => {
            // Create user message first
            let user_message_req = SendMessageRequest {
                conversation_id: request.conversation_id,
                content: request.content.clone(),
                role: "user".to_string(),
                parent_id: request.parent_id,
                model_provider_id: request.model_provider_id,
                model_id: request.model_id,
            };

            let _user_message = match chat::send_message(user_message_req, auth_user.user.id).await
            {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error saving user message: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Create assistant message
            let assistant_message_req = SendMessageRequest {
                conversation_id: request.conversation_id,
                content: response.content.clone(),
                role: "assistant".to_string(),
                parent_id: None, // Assistant message is a response to user message
                model_provider_id: request.model_provider_id,
                model_id: request.model_id,
            };

            match chat::send_message(assistant_message_req, auth_user.user.id).await {
                Ok(_assistant_message) => {
                    // Update conversation title if it's still "New Conversation"
                    if conversation.title == "New Conversation" {
                        let _ = chat::auto_update_conversation_title(
                            request.conversation_id,
                            auth_user.user.id,
                        )
                        .await;
                    }
                    
                    Ok(Json(serde_json::json!({
                        "content": response.content,
                        "finish_reason": response.finish_reason,
                        "usage": response.usage
                    })))
                }
                Err(e) => {
                    eprintln!("Error saving assistant message: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error calling AI provider: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
    let ai_provider_result: Result<Box<dyn AIProvider>, _> = match provider.provider_type.as_str() {
        "openai" => {
            let proxy_config = provider.proxy_settings.as_ref().map(|proxy_settings| {
                ProxyConfig {
                    enabled: proxy_settings.enabled,
                    url: proxy_settings.url.clone(),
                    username: Some(proxy_settings.username.clone()),
                    password: Some(proxy_settings.password.clone()),
                    no_proxy: proxy_settings.no_proxy
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                    ignore_ssl_certificates: proxy_settings.ignore_ssl_certificates,
                }
            });

            OpenAIProvider::new(
                provider.api_key.unwrap_or_default(),
                provider.base_url,
                proxy_config,
            ).map(|p| Box::new(p) as Box<dyn AIProvider>)
        }
        "anthropic" => {
            let proxy_config = provider.proxy_settings.as_ref().map(|proxy_settings| {
                ProxyConfig {
                    enabled: proxy_settings.enabled,
                    url: proxy_settings.url.clone(),
                    username: Some(proxy_settings.username.clone()),
                    password: Some(proxy_settings.password.clone()),
                    no_proxy: proxy_settings.no_proxy
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                    ignore_ssl_certificates: proxy_settings.ignore_ssl_certificates,
                }
            });

            AnthropicProvider::new(
                provider.api_key.unwrap_or_default(),
                provider.base_url,
                proxy_config,
            ).map(|p| Box::new(p) as Box<dyn AIProvider>)
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let ai_provider = ai_provider_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
                    auth_user.user.id
                ).await {
                    // Only proceed if the conversation has model provider and model configured
                    if let (Some(provider_id), Some(model_id)) = (conversation.model_provider_id, conversation.model_id) {
                        // Get AI response for the edited message (don't store user message again)
                        if let Err(e) = get_ai_response_for_edited_message(
                            edit_response.conversation_history,
                            edit_response.message.content.clone(),
                            conversation,
                            provider_id,
                            model_id,
                            auth_user.user.id
                        ).await {
                            eprintln!("Warning: Failed to get AI response for edited message: {:?}", e);
                        }
                    }
                }
            }

            Ok(Json(edit_response.message))
        },
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

/// Get message branches for a specific conversation position
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((conversation_id, timestamp)): Path<(Uuid, String)>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // Parse the timestamp
    let created_at = match chrono::DateTime::parse_from_rfc3339(&timestamp) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
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
