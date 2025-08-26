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
    models::{CreateSystemRAGInstanceRequest, UpdateRAGInstanceRequest, RAGInstance, RAGInstanceListResponse},
    queries::rag_instances::{
        create_system_rag_instance, list_system_rag_instances, 
        get_rag_instance, update_rag_instance, delete_rag_instance
    },
};

/// Create system RAG instance (admin only)
#[debug_handler]
pub async fn create_system_rag_instance_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateSystemRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    let instance = create_system_rag_instance(request).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::CREATED, Json(instance)))
}

/// List system RAG instances (admin only)
#[debug_handler]
pub async fn list_system_rag_instances_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<RAGInstanceListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 items

    let response = list_system_rag_instances(page, per_page).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Get system RAG instance by ID (admin only)
#[debug_handler]
pub async fn get_system_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<Json<RAGInstance>> {
    let instance = get_rag_instance(instance_id, auth_user.user.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    match instance {
        Some(instance) if instance.is_system => Ok((StatusCode::OK, Json(instance))),
        Some(_) => Err((StatusCode::FORBIDDEN, AppError::forbidden("Not a system instance"))),
        None => Err((StatusCode::NOT_FOUND, AppError::not_found("RAG instance"))),
    }
}

/// Update system RAG instance (admin only)
#[debug_handler]
pub async fn update_system_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<UpdateRAGInstanceRequest>,
) -> ApiResult<Json<RAGInstance>> {
    // First check if instance exists and is a system instance
    let existing = get_rag_instance(instance_id, auth_user.user.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    match existing {
        Some(instance) if instance.is_system => {
            let updated_instance = update_rag_instance(instance_id, request).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
            
            match updated_instance {
                Some(instance) => Ok((StatusCode::OK, Json(instance))),
                None => Err((StatusCode::NOT_FOUND, AppError::not_found("RAG instance"))),
            }
        },
        Some(_) => Err((StatusCode::FORBIDDEN, AppError::forbidden("Not a system instance"))),
        None => Err((StatusCode::NOT_FOUND, AppError::not_found("RAG instance"))),
    }
}

/// Delete system RAG instance (admin only)
#[debug_handler]
pub async fn delete_system_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // First check if instance exists and is a system instance
    let existing = get_rag_instance(instance_id, auth_user.user.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    match existing {
        Some(instance) if instance.is_system => {
            let success = delete_rag_instance(instance_id).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
            
            if success {
                Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
            } else {
                Ok((StatusCode::NOT_FOUND, StatusCode::NOT_FOUND))
            }
        },
        Some(_) => Err((StatusCode::FORBIDDEN, AppError::forbidden("Not a system instance"))),
        None => Err((StatusCode::NOT_FOUND, AppError::not_found("RAG instance"))),
    }
}