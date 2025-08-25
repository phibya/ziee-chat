use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::{errors::{ApiResult, AppError}, middleware::auth::AuthenticatedUser};
use crate::types::PaginationQuery;
use crate::database::{
    models::{
        AssignRAGProviderToGroupRequest, UpdateGroupRAGProviderRequest, UserGroupRAGProviderResponse,
    },
    queries::user_group_rag_providers::{
        assign_rag_provider_to_group, list_user_group_rag_provider_relationships,
        remove_rag_provider_from_group, update_group_rag_provider_permissions,
    },
};

/// Assign RAG provider to user group
#[debug_handler]
pub async fn assign_rag_provider_to_group_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignRAGProviderToGroupRequest>,
) -> ApiResult<Json<UserGroupRAGProviderResponse>> {
    let response = assign_rag_provider_to_group(request).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

/// Update RAG provider permissions for user group
#[debug_handler]
pub async fn update_group_rag_provider_permissions_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((group_id, provider_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateGroupRAGProviderRequest>,
) -> ApiResult<Json<UserGroupRAGProviderResponse>> {
    let response = update_group_rag_provider_permissions(group_id, provider_id, request).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Remove RAG provider from user group
#[debug_handler]
pub async fn remove_rag_provider_from_group_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((group_id, provider_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let success = remove_rag_provider_from_group(group_id, provider_id).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    if success {
        Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
    } else {
        Ok((StatusCode::NOT_FOUND, StatusCode::NOT_FOUND))
    }
}

/// List all user group RAG provider relationships
#[debug_handler]
pub async fn list_user_group_rag_provider_relationships_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(_params): Query<PaginationQuery>,
) -> ApiResult<Json<Vec<UserGroupRAGProviderResponse>>> {
    let relationships = list_user_group_rag_provider_relationships().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::OK, Json(relationships)))
}