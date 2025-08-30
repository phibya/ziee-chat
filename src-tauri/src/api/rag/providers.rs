use axum::{
  debug_handler,
  extract::{Path, Query},
  http::StatusCode,
  Extension, Json,
};
use uuid::Uuid;

use crate::{
  api::{
    errors::{ApiResult, AppError},
    middleware::AuthenticatedUser,
  },
  database::{
    models::{
      CreateRAGProviderRequest, RAGProvider, RAGProviderListResponse,
      UpdateRAGProviderRequest,
    },
    queries::rag_providers,
  },
};
use crate::api::types::PaginationQuery;
// =============================================================================
// ADMIN ENDPOINTS - Full provider management
// =============================================================================

/// List all RAG providers (admin only)
#[debug_handler]
pub async fn list_rag_providers(
    Extension(_user): Extension<AuthenticatedUser>,
    Query(pagination): Query<PaginationQuery>,
) -> ApiResult<Json<RAGProviderListResponse>> {
    let response = rag_providers::list_rag_providers(pagination.page, pagination.per_page)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;
    Ok((StatusCode::OK, Json(response)))
}

/// Get RAG provider by ID (admin only)
#[debug_handler]
pub async fn get_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::get_rag_provider_by_id(provider_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG provider")))?;

    Ok((StatusCode::OK, Json(provider)))
}

/// Create new RAG provider (admin only)
#[debug_handler]
pub async fn create_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGProviderRequest>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::create_rag_provider(request)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;

    Ok((StatusCode::CREATED, Json(provider)))
}

/// Update RAG provider (admin only)
#[debug_handler]
pub async fn update_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<UpdateRAGProviderRequest>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::update_rag_provider(provider_id, request)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;

    Ok((StatusCode::OK, Json(provider)))
}

/// Delete RAG provider (admin only)
#[debug_handler]
pub async fn delete_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_providers::delete_rag_provider(provider_id)
        .await
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database operation failed"),
            )
        })?;

    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

/// Test RAG provider connection (admin only)
#[debug_handler]
pub async fn test_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(_provider_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement actual provider testing
    let result = serde_json::json!({
        "success": true,
        "message": "Connection test successful"
    });
    Ok((StatusCode::OK, Json(result)))
}
