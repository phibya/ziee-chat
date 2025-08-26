use axum::{
    debug_handler,
    extract::{Path, Query},
    Extension, Json,
};
use uuid::Uuid;

use crate::api::{errors::{ApiResult, AppError}, middleware::auth::AuthenticatedUser};
use crate::database::{
    models::{  RAGInstanceFile, RAGInstanceFilesQuery},
    queries::{
        rag_instance_files::{
            list_rag_instance_files,
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
