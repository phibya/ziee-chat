use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        CreateRAGRepositoryRequest,
        RAGRepository, RAGRepositoryConnectionTestResponse, RAGRepositoryListResponse,
        UpdateRAGRepositoryRequest,
    },
    queries::rag_repositories,
};
use crate::types::PaginationQuery;

// RAG Repository endpoints
#[debug_handler]
pub async fn list_rag_repositories(
    Extension(_user): Extension<AuthenticatedUser>,
    Query(pagination): Query<PaginationQuery>,
) -> ApiResult<Json<RAGRepositoryListResponse>> {
    let response = rag_repositories::list_rag_repositories(pagination.page, pagination.per_page)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;
    Ok((StatusCode::OK, Json(response)))
}

#[debug_handler]
pub async fn get_rag_repository(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
) -> ApiResult<Json<RAGRepository>> {
    let repository = rag_repositories::get_rag_repository_by_id(repository_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG repository")))?;

    Ok((StatusCode::OK, Json(repository)))
}

#[debug_handler]
pub async fn create_rag_repository(
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGRepositoryRequest>,
) -> ApiResult<Json<RAGRepository>> {
    let repository = rag_repositories::create_rag_repository(request)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;
    Ok((StatusCode::OK, Json(repository)))
}

#[debug_handler]
pub async fn update_rag_repository(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
    Json(request): Json<UpdateRAGRepositoryRequest>,
) -> ApiResult<Json<RAGRepository>> {
    let repository = rag_repositories::update_rag_repository(repository_id, request)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;
    Ok((StatusCode::OK, Json(repository)))
}

#[debug_handler]
pub async fn delete_rag_repository(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_repositories::delete_rag_repository(repository_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn test_rag_repository_connection(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(repository_id): Path<Uuid>,
) -> ApiResult<Json<RAGRepositoryConnectionTestResponse>> {
    let _repository = rag_repositories::get_rag_repository_by_id(repository_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG repository")))?;

    // TODO: Implement actual connection test logic
    // For now, we'll return a mock response
    let response = RAGRepositoryConnectionTestResponse {
        success: true,
        message: "Connection test successful".to_string(),
        available_databases_count: Some(0),
    };

    Ok((StatusCode::OK, Json(response)))
}

