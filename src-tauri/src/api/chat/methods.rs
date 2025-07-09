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

/// Get conversation by ID
pub async fn get_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Conversation>, StatusCode> {
    match chat::get_conversation_by_id(conversation_id, auth_user.user.id).await {
        Ok(Some(conversation)) => Ok(Json(conversation)),
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

/// Send a message in a conversation
pub async fn send_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<Message>, StatusCode> {
    match chat::send_message(request, auth_user.user.id).await {
        Ok(message) => Ok(Json(message)),
        Err(e) => {
            eprintln!("Error sending message: {}", e);
            match e {
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
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