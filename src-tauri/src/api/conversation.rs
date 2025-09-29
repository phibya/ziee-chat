use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::api::types::ConversationPaginationQuery;
use crate::database::{
    models::{
        Conversation, ConversationListResponse, CreateConversationRequest,
        UpdateConversationRequest,
    },
    queries::chat,
};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchQuery {
    q: String,
    page: Option<i32>,
    per_page: Option<i32>,
    project_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct OperationSuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchBranchRequest {
    pub branch_id: Uuid,
}

/// Create a new conversation
#[debug_handler]
pub async fn create_conversation(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateConversationRequest>,
) -> ApiResult<Json<Conversation>> {
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
) -> ApiResult<Json<Conversation>> {
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
) -> ApiResult<Json<ConversationListResponse>> {
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
) -> ApiResult<Json<Conversation>> {
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
) -> ApiResult<StatusCode> {
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

/// Switch to a different branch for a conversation
#[debug_handler]
pub async fn switch_conversation_branch(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<SwitchBranchRequest>,
) -> ApiResult<Json<OperationSuccessResponse>> {
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

/// Search conversations
#[debug_handler]
pub async fn search_conversations(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<Json<ConversationListResponse>> {
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