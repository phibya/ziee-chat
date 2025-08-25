use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::{errors::{ApiResult, AppError}, middleware::auth::AuthenticatedUser};
use crate::database::{
    models::{AddFilesToRAGInstanceRequest, AddFilesToRAGInstanceResponse, RAGInstanceFile, RAGInstanceFilesQuery},
    queries::{
        rag_instance_files::{
            add_files_to_rag_instance, list_rag_instance_files, remove_file_from_rag_instance,
        },
        rag_instances::validate_rag_instance_access,
    },
};

/// List files in RAG instance
#[debug_handler]
pub async fn list_rag_instance_files_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Query(params): Query<RAGInstanceFilesQuery>,
) -> ApiResult<Json<Vec<RAGInstanceFile>>> {
    // Check if user has access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((axum::http::StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 items

    let files = list_rag_instance_files(
        instance_id,
        page,
        per_page,
        params.status_filter,
    ).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    Ok((axum::http::StatusCode::OK, Json(files)))
}

/// Add files to RAG instance
#[debug_handler]
pub async fn add_files_to_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Json(request): Json<AddFilesToRAGInstanceRequest>,
) -> ApiResult<Json<AddFilesToRAGInstanceResponse>> {
    // Check if user owns this instance (require ownership to add files)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((axum::http::StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    let response = add_files_to_rag_instance(instance_id, request.file_ids).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Remove file from RAG instance
#[debug_handler]
pub async fn remove_file_from_rag_instance_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((instance_id, file_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    // Check if user owns this instance (require ownership to remove files)
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, true).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((axum::http::StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    let success = remove_file_from_rag_instance(instance_id, file_id).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    
    if success {
        Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
    } else {
        Ok((StatusCode::NOT_FOUND, StatusCode::NOT_FOUND))
    }
}