use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::errors::{ApiResult, ApiResult2, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        CreateRAGDatabaseRequest, CreateRAGProviderRequest,
        DownloadRAGDatabaseFromRepositoryRequest, RAGDatabase, RAGProvider,
        RAGProviderListResponse, RAGRepositoryConnectionTestResponse, UpdateRAGDatabaseRequest,
        UpdateRAGProviderRequest,
    },
    queries::{rag_providers, rag_repositories},
};
use crate::types::PaginationQuery;


// RAG Provider endpoints
#[debug_handler]
pub async fn list_rag_providers(
    Extension(_user): Extension<AuthenticatedUser>,
    Query(pagination): Query<PaginationQuery>,
) -> ApiResult2<Json<RAGProviderListResponse>> {
    let response = rag_providers::list_rag_providers(pagination.page, pagination.per_page).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(response)))
}

#[debug_handler]
pub async fn get_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult2<Json<RAGProvider>> {
    let provider = rag_providers::get_rag_provider_by_id(provider_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG provider")))?;

    Ok((StatusCode::OK, Json(provider)))
}

#[debug_handler]
pub async fn create_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGProviderRequest>,
) -> ApiResult2<Json<RAGProvider>> {
    let provider = rag_providers::create_rag_provider(request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(provider)))
}

#[debug_handler]
pub async fn update_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<UpdateRAGProviderRequest>,
) -> ApiResult2<Json<RAGProvider>> {
    let provider = rag_providers::update_rag_provider(provider_id, request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(provider)))
}

#[debug_handler]
pub async fn delete_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    rag_providers::delete_rag_provider(provider_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn clone_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult2<Json<RAGProvider>> {
    let provider = rag_providers::clone_rag_provider(provider_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(provider)))
}

// RAG Database endpoints
#[debug_handler]
pub async fn list_rag_provider_databases(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult2<Json<Vec<RAGDatabase>>> {
    let databases = rag_providers::list_rag_databases_by_provider(provider_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(databases)))
}

#[debug_handler]
pub async fn add_database_to_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<CreateRAGDatabaseRequest>,
) -> ApiResult2<Json<RAGDatabase>> {
    let database = rag_providers::create_rag_database(provider_id, request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(database)))
}

#[debug_handler]
pub async fn get_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<Json<RAGDatabase>> {
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG database")))?;

    Ok((StatusCode::OK, Json(database)))
}

#[debug_handler]
pub async fn update_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
    Json(request): Json<UpdateRAGDatabaseRequest>,
) -> ApiResult2<Json<RAGDatabase>> {
    let database = rag_providers::update_rag_database(database_id, request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::OK, Json(database)))
}

#[debug_handler]
pub async fn delete_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    rag_providers::delete_rag_database(database_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn start_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    // First check if the database belongs to a local provider
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG database")))?;

    let provider = rag_providers::get_rag_provider_by_id(database.provider_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG provider")))?;

    if provider.provider_type.as_str() != "local" {
        return Err((StatusCode::BAD_REQUEST, AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Only local RAG providers support start/stop operations",
        )));
    }

    rag_providers::set_rag_database_active(database_id, true).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn stop_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    // First check if the database belongs to a local provider
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG database")))?;

    let provider = rag_providers::get_rag_provider_by_id(database.provider_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("RAG provider")))?;

    if provider.provider_type.as_str() != "local" {
        return Err((StatusCode::BAD_REQUEST, AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Only local RAG providers support start/stop operations",
        )));
    }

    rag_providers::set_rag_database_active(database_id, false).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn enable_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    rag_providers::set_rag_database_enabled(database_id, true).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}

#[debug_handler]
pub async fn disable_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    rag_providers::set_rag_database_enabled(database_id, false).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database operation failed")))?;
    Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
}
