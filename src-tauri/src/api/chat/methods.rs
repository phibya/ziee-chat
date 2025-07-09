use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        Conversation, ConversationListResponse, CreateConversationRequest, 
        UpdateConversationRequest, Message, SendMessageRequest, EditMessageRequest,
        ConversationSummary,
    },
    queries::chat,
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
            let response = serde_json::json!({
                "conversation": conversation,
                "messages": []
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

/// Send a message (simplified version)
pub async fn send_message_stream(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ChatMessageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return a placeholder response
    // The full streaming implementation will be added later
    Ok(Json(serde_json::json!({
        "message": "Chat functionality coming soon",
        "conversation_id": request.conversation_id,
        "content": request.content
    })))
}

/// Edit a message (creates a new branch)
pub async fn edit_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<EditMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    match chat::edit_message(message_id, request, auth_user.user.id).await {
        Ok(Some(message)) => Ok(Json(message)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error editing message: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Switch to a different branch (placeholder)
pub async fn switch_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Placeholder implementation
    Ok(StatusCode::NO_CONTENT)
}

/// Get message with branches (placeholder)
pub async fn get_message_branches(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Placeholder implementation
    Ok(Json(serde_json::json!({
        "message_id": message_id,
        "branches": []
    })))
}