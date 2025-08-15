use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
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

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

// RAG Provider endpoints
pub async fn list_rag_providers(
    Extension(_user): Extension<AuthenticatedUser>,
    Query(pagination): Query<PaginationQuery>,
) -> ApiResult<Json<RAGProviderListResponse>> {
    let response = rag_providers::list_rag_providers(pagination.page, pagination.per_page).await?;
    Ok(Json(response))
}

pub async fn get_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::get_rag_provider_by_id(provider_id)
        .await?
        .ok_or(AppError::not_found("RAG provider"))?;

    Ok(Json(provider))
}

pub async fn create_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateRAGProviderRequest>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::create_rag_provider(request).await?;
    Ok(Json(provider))
}

pub async fn update_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<UpdateRAGProviderRequest>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::update_rag_provider(provider_id, request).await?;
    Ok(Json(provider))
}

pub async fn delete_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_providers::delete_rag_provider(provider_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clone_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<RAGProvider>> {
    let provider = rag_providers::clone_rag_provider(provider_id).await?;
    Ok(Json(provider))
}

// RAG Database endpoints
pub async fn list_rag_provider_databases(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
) -> ApiResult<Json<Vec<RAGDatabase>>> {
    let databases = rag_providers::list_rag_databases_by_provider(provider_id).await?;
    Ok(Json(databases))
}

pub async fn add_database_to_rag_provider(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(provider_id): Path<Uuid>,
    Json(request): Json<CreateRAGDatabaseRequest>,
) -> ApiResult<Json<RAGDatabase>> {
    let database = rag_providers::create_rag_database(provider_id, request).await?;
    Ok(Json(database))
}

pub async fn get_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<Json<RAGDatabase>> {
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await?
        .ok_or(AppError::not_found("RAG database"))?;

    Ok(Json(database))
}

pub async fn update_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
    Json(request): Json<UpdateRAGDatabaseRequest>,
) -> ApiResult<Json<RAGDatabase>> {
    let database = rag_providers::update_rag_database(database_id, request).await?;
    Ok(Json(database))
}

pub async fn delete_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_providers::delete_rag_database(database_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn start_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // First check if the database belongs to a local provider
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await?
        .ok_or(AppError::not_found("RAG database"))?;

    let provider = rag_providers::get_rag_provider_by_id(database.provider_id)
        .await?
        .ok_or(AppError::not_found("RAG provider"))?;

    if provider.provider_type != "local" {
        return Err(AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Only local RAG providers support start/stop operations",
        ));
    }

    rag_providers::set_rag_database_active(database_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn stop_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // First check if the database belongs to a local provider
    let database = rag_providers::get_rag_database_by_id(database_id)
        .await?
        .ok_or(AppError::not_found("RAG database"))?;

    let provider = rag_providers::get_rag_provider_by_id(database.provider_id)
        .await?
        .ok_or(AppError::not_found("RAG provider"))?;

    if provider.provider_type != "local" {
        return Err(AppError::new(
            crate::api::errors::ErrorCode::ValidInvalidInput,
            "Only local RAG providers support start/stop operations",
        ));
    }

    rag_providers::set_rag_database_active(database_id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn enable_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_providers::set_rag_database_enabled(database_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn disable_rag_database(
    Extension(_user): Extension<AuthenticatedUser>,
    Path(database_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    rag_providers::set_rag_database_enabled(database_id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}
