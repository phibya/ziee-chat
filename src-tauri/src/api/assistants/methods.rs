use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::AuthenticatedUser;
use crate::api::permissions;
use crate::database::{
    models::{Assistant, AssistantListResponse, CreateAssistantRequest, UpdateAssistantRequest},
    queries::assistants,
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

/// Create a new assistant
pub async fn create_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateAssistantRequest>,
) -> Result<Json<Assistant>, StatusCode> {
    // Users can create their own assistants
    match assistants::create_assistant(request, Some(auth_user.user.id)).await {
        Ok(assistant) => Ok(Json(assistant)),
        Err(e) => {
            eprintln!("Error creating assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new template assistant (admin only)
pub async fn create_template_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(mut request): Json<CreateAssistantRequest>,
) -> Result<Json<Assistant>, StatusCode> {
    // Only admins can create template assistants
    request.is_template = Some(true);
    match assistants::create_assistant(request, Some(auth_user.user.id)).await {
        Ok(assistant) => Ok(Json(assistant)),
        Err(e) => {
            eprintln!("Error creating template assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get assistant by ID
pub async fn get_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> Result<Json<Assistant>, StatusCode> {
    match assistants::get_assistant_by_id(assistant_id, Some(auth_user.user.id)).await {
        Ok(Some(assistant)) => Ok(Json(assistant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get assistant by ID (admin view)
pub async fn get_assistant_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> Result<Json<Assistant>, StatusCode> {
    match assistants::get_assistant_by_id(assistant_id, None).await {
        Ok(Some(assistant)) => Ok(Json(assistant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting assistant (admin): {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List assistants for user
pub async fn list_assistants(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<AssistantListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match assistants::list_assistants(page, per_page, Some(auth_user.user.id), false).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error listing assistants: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all assistants (admin view)
pub async fn list_assistants_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<AssistantListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match assistants::list_assistants(page, per_page, None, true).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error listing assistants (admin): {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update assistant
pub async fn update_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
    Json(request): Json<UpdateAssistantRequest>,
) -> Result<Json<Assistant>, StatusCode> {
    match assistants::update_assistant(assistant_id, request, Some(auth_user.user.id), false).await
    {
        Ok(Some(assistant)) => Ok(Json(assistant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update assistant (admin view)
pub async fn update_assistant_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
    Json(request): Json<UpdateAssistantRequest>,
) -> Result<Json<Assistant>, StatusCode> {
    match assistants::update_assistant(assistant_id, request, None, true).await {
        Ok(Some(assistant)) => Ok(Json(assistant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating assistant (admin): {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete assistant
pub async fn delete_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match assistants::delete_assistant(assistant_id, Some(auth_user.user.id), false).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete assistant (admin view)
pub async fn delete_assistant_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match assistants::delete_assistant(assistant_id, None, true).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting assistant (admin): {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get default assistant
pub async fn get_default_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Assistant>, StatusCode> {
    match assistants::get_default_assistant().await {
        Ok(Some(assistant)) => Ok(Json(assistant)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting default assistant: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
