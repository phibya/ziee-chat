use axum::{
    debug_handler,
    extract::{Extension, Multipart, Path, Query},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::{errors::ErrorCode, files::FileOperationSuccessResponse};
use crate::api::{
    errors::{ApiResult, AppError},
    middleware::auth::AuthenticatedUser,
};
use crate::database::{
    models::{
        file::{FileCreateData, UploadFileResponse},
        RAGInstanceFilesListResponse, RAGInstanceFilesQuery,
    },
    queries::{
        files,
        rag_instance_files::{
            add_file_to_rag_instance, list_rag_instance_files, remove_file_from_rag_instance,
        },
        rag_instances::validate_rag_instance_access,
    },
};
use crate::utils::file_storage::{extract_extension, get_mime_type_from_extension};
use crate::global::RAG_FILE_STORAGE;

/// List files in RAG instance
#[debug_handler]
pub async fn list_rag_instance_files_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    Query(params): Query<RAGInstanceFilesQuery>,
) -> ApiResult<Json<RAGInstanceFilesListResponse>> {
    // Check if user has access to this instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::from(e),
            )
        })?;
    if !has_access {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            AppError::forbidden("Access denied"),
        ));
    }

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(50).min(100); // Cap at 100 items

    let response = list_rag_instance_files(
        instance_id,
        page,
        per_page,
        params.status_filter,
        params.search,
    )
    .await
    .map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from(e),
        )
    })?;

    Ok((axum::http::StatusCode::OK, Json(response)))
}

/// Upload file to RAG instance
#[debug_handler]
pub async fn upload_rag_file_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(instance_id): Path<Uuid>,
    mut multipart: Multipart,
) -> ApiResult<Json<UploadFileResponse>> {
    // Validate user has access to RAG instance
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    // Extract multipart data
    let mut file_data = None;
    let mut filename = String::new();
    let mut file_size = 0u64;

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            AppError::new(ErrorCode::ValidInvalidInput, "Invalid multipart data"),
        )
    })? {
        let field_name = field.name().unwrap_or("");
        match field_name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        AppError::new(ErrorCode::ValidInvalidInput, "Failed to read file data"),
                    )
                })?;
                file_size = data.len() as u64;
                file_data = Some(data);
            }
            _ => continue,
        }
    }

    let file_data = file_data.ok_or((
        StatusCode::BAD_REQUEST,
        AppError::new(
            ErrorCode::ValidMissingRequiredField,
            "No file data provided",
        ),
    ))?;

    if filename.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                ErrorCode::ValidMissingRequiredField,
                "Filename cannot be empty",
            ),
        ));
    }

    match process_rag_file_upload(
        auth_user.user.id,
        instance_id,
        filename,
        file_data,
        file_size,
    )
    .await
    {
        Ok(response) => Ok((StatusCode::OK, response)),
        Err(status) => Err((
            status,
            AppError::internal_error("Failed to upload RAG file"),
        )),
    }
}

/// Process RAG file upload (no processing, just store original)
async fn process_rag_file_upload(
    user_id: Uuid,
    instance_id: Uuid,
    filename: String,
    file_data: bytes::Bytes,
    file_size: u64,
) -> Result<Json<UploadFileResponse>, StatusCode> {
    let file_id = Uuid::new_v4();
    let extension = extract_extension(&filename);
    let mime_type = get_mime_type_from_extension(&extension);

    // Save RAG file (original only, no processing)
    let file_path = RAG_FILE_STORAGE
        .save_rag_file(instance_id, file_id, &extension, &file_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculate checksum
    let checksum = RAG_FILE_STORAGE
        .calculate_checksum(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create file record in files table with RAG instance association
    let file_create_data = FileCreateData {
        id: file_id,
        user_id,
        filename,
        file_size: file_size as i64,
        mime_type,
        checksum: Some(checksum),
        project_id: None, // RAG files don't belong to projects
        thumbnail_count: 0, // No processing for RAG files
        page_count: 0,
        processing_metadata: serde_json::json!({}),
    };

    let file = files::create_file(file_create_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create rag_instance_files association
    add_file_to_rag_instance(instance_id, file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UploadFileResponse { file }))
}

/// Delete file from RAG instance
#[debug_handler]
pub async fn delete_rag_file_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((instance_id, file_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<FileOperationSuccessResponse>> {
    // Validate access
    let has_access = validate_rag_instance_access(auth_user.user.id, instance_id, false)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, AppError::from(e)))?;
    if !has_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    // Get file info to verify ownership and get filename
    let file_db = files::get_file_by_id_and_user(file_id, auth_user.user.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get file"),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("File")))?;

    // Remove from rag_instance_files table
    let removed = remove_file_from_rag_instance(instance_id, file_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to remove file from RAG instance"),
            )
        })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("File not found in RAG instance"),
        ));
    }

    // Delete from RAG storage
    let extension = extract_extension(&file_db.filename);
    RAG_FILE_STORAGE
        .delete_rag_file(instance_id, file_id, &extension)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete file from storage"),
            )
        })?;

    // Delete file from database (rag_instance_files will be handled by CASCADE)
    files::delete_file(file_id, auth_user.user.id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to delete file from database"),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(FileOperationSuccessResponse { success: true }),
    ))
}
