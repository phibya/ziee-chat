use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult2, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{Assistant, AssistantListResponse, CreateAssistantRequest, UpdateAssistantRequest},
    queries::assistants,
};
use crate::types::PaginationQuery;


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
#[debug_handler]
pub async fn create_template_assistant(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(mut request): Json<CreateAssistantRequest>,
) -> ApiResult2<Json<Assistant>> {
    // Only admins can create template assistants
    request.is_template = Some(true);
    match assistants::create_assistant(request, Some(auth_user.user.id)).await {
        Ok(assistant) => Ok((StatusCode::OK, Json(assistant))),
        Err(e) => {
            eprintln!("Error creating template assistant: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to create template assistant")
            ))
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
#[debug_handler]
pub async fn get_assistant_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> ApiResult2<Json<Assistant>> {
    match assistants::get_assistant_by_id(assistant_id, None).await {
        Ok(Some(assistant)) => Ok((StatusCode::OK, Json(assistant))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Assistant")
        )),
        Err(e) => {
            eprintln!("Error getting assistant (admin): {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get assistant")
            ))
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
#[debug_handler]
pub async fn list_assistants_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult2<Json<AssistantListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match assistants::list_assistants(page, per_page, None, true).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error listing assistants (admin): {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to list assistants")
            ))
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
#[debug_handler]
pub async fn update_assistant_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
    Json(request): Json<UpdateAssistantRequest>,
) -> ApiResult2<Json<Assistant>> {
    match assistants::update_assistant(assistant_id, request, None, true).await {
        Ok(Some(assistant)) => Ok((StatusCode::OK, Json(assistant))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Assistant")
        )),
        Err(e) => {
            eprintln!("Error updating assistant (admin): {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update assistant")
            ))
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
#[debug_handler]
pub async fn delete_assistant_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(assistant_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    match assistants::delete_assistant(assistant_id, None, true).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Assistant")
        )),
        Err(e) => {
            eprintln!("Error deleting assistant (admin): {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete assistant")
            ))
        }
    }
}

/// Get default assistant
pub async fn get_default_assistant(
    Extension(_auth_user): Extension<AuthenticatedUser>,
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
