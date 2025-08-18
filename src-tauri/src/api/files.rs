use axum::{
    debug_handler,
    extract::{Extension, Multipart, Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::{middleware::AuthenticatedUser, errors::{ApiResult2, AppError, ErrorCode}},
    database::{models::file::*, queries::files},
    processing::ProcessingManager,
    utils::file_storage::{extract_extension, get_mime_type_from_extension},
    FILE_STORAGE,
};

// Initialize global processing manager
use once_cell::sync::Lazy;

static PROCESSING_MANAGER: Lazy<Arc<ProcessingManager>> =
    Lazy::new(|| Arc::new(ProcessingManager::new(FILE_STORAGE.clone())));

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PreviewParams {
    pub page: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DownloadTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DownloadTokenParams {
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadTokenClaims {
    pub file_id: String,
    pub user_id: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

// Initialize file storage on first use
pub async fn initialize_file_storage() -> Result<(), StatusCode> {
    FILE_STORAGE.initialize().await.map_err(|e| {
        eprintln!("Failed to initialize file storage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// Upload file (general)
#[debug_handler]
pub async fn upload_file(
    Extension(user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> ApiResult2<Json<UploadFileResponse>> {
    let mut file_data = None;
    let mut filename = String::new();
    let mut file_size = 0u64;

    // Extract multipart data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidInvalidInput, "Invalid multipart data")))?  
    {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| (StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidInvalidInput, "Failed to read file data")))?;
                file_size = data.len() as u64;
                file_data = Some(data);
            }
            _ => continue,
        }
    }

    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidMissingRequiredField, "No file data provided")))?;
    if filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidMissingRequiredField, "Filename cannot be empty")));
    }

    match process_file_upload(user.user_id, filename, file_data, file_size, None).await {
        Ok(response) => Ok((StatusCode::OK, response)),
        Err(status) => Err((status, AppError::internal_error("Failed to upload file")))
    }
}

// Upload file to project
#[debug_handler]
pub async fn upload_project_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> ApiResult2<Json<UploadFileResponse>> {
    let mut file_data = None;
    let mut filename = String::new();
    let mut file_size = 0u64;

    // Extract multipart data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidInvalidInput, "Invalid multipart data")))?  
    {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| (StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidInvalidInput, "Failed to read file data")))?;
                file_size = data.len() as u64;
                file_data = Some(data);
            }
            _ => continue,
        }
    }

    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidMissingRequiredField, "No file data provided")))?;
    if filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, AppError::new(ErrorCode::ValidMissingRequiredField, "Filename cannot be empty")));
    }

    match process_file_upload(
        user.user_id,
        filename,
        file_data,
        file_size,
        Some(project_id),
    )
    .await {
        Ok(response) => Ok((StatusCode::OK, response)),
        Err(status) => Err((status, AppError::internal_error("Failed to upload file to project")))
    }
}

async fn process_file_upload(
    user_id: Uuid,
    filename: String,
    file_data: bytes::Bytes,
    file_size: u64,
    project_id: Option<Uuid>,
) -> Result<Json<UploadFileResponse>, StatusCode> {
    let file_id = Uuid::new_v4();
    let extension = extract_extension(&filename);
    let mime_type = get_mime_type_from_extension(&extension);

    // Save original file
    let file_path = FILE_STORAGE.get_original_path(file_id, &extension);
    FILE_STORAGE
        .save_file_bytes(&file_path, &file_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculate checksum
    let checksum = FILE_STORAGE
        .calculate_checksum(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Process file content
    let processing_result = PROCESSING_MANAGER
        .process_file(&file_path, &mime_type)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save processed content
    if let Some(ref text_content) = processing_result.text_content {
        FILE_STORAGE
            .save_text_content(file_id, text_content)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Create file record
    let file_create_data = FileCreateData {
        id: file_id,
        user_id,
        filename,
        file_size: file_size as i64,
        mime_type,
        checksum: Some(checksum),
        project_id,
        thumbnail_count: processing_result.thumbnail_count,
        page_count: processing_result.page_count,
        processing_metadata: processing_result.metadata,
    };

    let file = files::create_file(file_create_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UploadFileResponse { file }))
}

// Get file metadata
#[debug_handler]
pub async fn get_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> ApiResult2<Json<File>> {
    let file = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to get file")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("File")))?;
    Ok((StatusCode::OK, Json(file)))
}

// Generate download token
#[debug_handler]
pub async fn generate_download_token(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> ApiResult2<Json<DownloadTokenResponse>> {
    // Verify file belongs to user
    let _file = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to verify file")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("File")))?;

    let now = Utc::now();
    let expires_at = now + Duration::hours(1);

    // Create JWT claims
    let claims = DownloadTokenClaims {
        file_id: file_id.to_string(),
        user_id: user.user_id.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    // Generate JWT token
    let jwt_secret = crate::utils::jwt_secret::get_jwt_secret();
    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(jwt_secret.as_ref());

    let token = encode(&header, &claims, &key).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to generate download token")))?;

    Ok((StatusCode::OK, Json(DownloadTokenResponse { token, expires_at })))
}

// Download file (with authentication)
#[debug_handler]
pub async fn download_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let file_db = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    download_file_internal(file_db).await
}

// Download file with token (no authentication required)
#[debug_handler]
pub async fn download_file_with_token(
    Path(file_id): Path<Uuid>,
    Query(params): Query<DownloadTokenParams>,
) -> Result<Response, StatusCode> {
    let token = params.token.ok_or(StatusCode::BAD_REQUEST)?;

    // Decode and validate JWT token
    let jwt_secret = crate::utils::jwt_secret::get_jwt_secret();
    let key = DecodingKey::from_secret(jwt_secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    let claims = decode::<DownloadTokenClaims>(&token, &key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .claims;

    // Verify file_id matches
    let token_file_id = Uuid::parse_str(&claims.file_id).map_err(|_| StatusCode::UNAUTHORIZED)?;

    if token_file_id != file_id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Parse user_id from token
    let user_id = Uuid::parse_str(&claims.user_id).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Get file info
    let file_db = files::get_file_by_id_and_user(token_file_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    download_file_internal(file_db).await
}

// Internal download function shared by both endpoints
async fn download_file_internal(file_db: File) -> Result<Response, StatusCode> {
    let extension = extract_extension(&file_db.filename);
    let file_path = FILE_STORAGE.get_original_path(file_db.id, &extension);
    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let file_data = FILE_STORAGE
        .read_file_bytes(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_db.filename),
        ),
    ];

    Ok((headers, file_data).into_response())
}

// Get file preview/thumbnail
#[debug_handler]
pub async fn get_file_preview(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
    Query(params): Query<PreviewParams>,
) -> Result<Response, StatusCode> {
    let _file_db = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let page = params.page.unwrap_or(1);
    let thumbnail_path = FILE_STORAGE.get_thumbnail_path(file_id, page);

    if !thumbnail_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let thumbnail_data = FILE_STORAGE
        .read_file_bytes(&thumbnail_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let headers = [
        (header::CONTENT_TYPE, "image/jpeg".to_string()),
        (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
    ];

    Ok((headers, thumbnail_data).into_response())
}

// Delete file
#[debug_handler]
pub async fn delete_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> ApiResult2<Json<serde_json::Value>> {
    let file_db = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to get file")))?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("File")))?;

    // Delete from database (this will cascade to relationships)
    let deleted = files::delete_file(file_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to delete file from database")))?;

    if !deleted {
        return Err((StatusCode::NOT_FOUND, AppError::not_found("File")));
    }

    // Delete from filesystem
    let extension = extract_extension(&file_db.filename);
    FILE_STORAGE
        .delete_file(file_id, Some(&extension))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to delete file from storage")))?;

    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true }))))
}

// List files by project
#[debug_handler]
pub async fn list_project_files(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<FileListParams>,
) -> ApiResult2<Json<FileListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let (files, total) = files::get_files_by_project(project_id, user.user_id, page, per_page)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to get project files")))?;

    Ok((StatusCode::OK, Json(FileListResponse {
        files,
        total,
        page,
        per_page,
    })))
}

// List files by message
#[debug_handler]
pub async fn list_message_files(
    Extension(user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> ApiResult2<Json<Vec<File>>> {
    let files = files::get_files_by_message(message_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to get message files")))?;
    Ok((StatusCode::OK, Json(files)))
}

// Remove file from message
#[debug_handler]
pub async fn remove_file_from_message(
    Extension(user): Extension<AuthenticatedUser>,
    Path((file_id, message_id)): Path<(Uuid, Uuid)>,
) -> ApiResult2<Json<serde_json::Value>> {
    // Verify file belongs to user
    let file_exists = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to verify file")))?
        .is_some();

    if !file_exists {
        return Err((StatusCode::NOT_FOUND, AppError::not_found("File")));
    }

    // Remove relationship
    let removed = files::delete_message_file_relationship(message_id, file_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to remove file from message")))?;

    if !removed {
        return Err((StatusCode::NOT_FOUND, AppError::not_found("File relationship")));
    }

    // Check if file is now orphaned
    let has_messages = files::check_file_has_message_associations(file_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to check file associations")))?;

    let has_project = files::check_file_has_project_association(file_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to check file project association")))?;

    // If orphaned, delete the file completely
    if !has_messages && !has_project {
        let file_db = files::get_file_by_id(file_id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to get file for cleanup")))?;

        if let Some(file_db) = file_db {
            // Delete from database
            files::delete_file(file_id, user.user_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to delete orphaned file from database")))?;

            // Delete from filesystem
            let extension = extract_extension(&file_db.filename);
            FILE_STORAGE
                .delete_file(file_id, Some(&extension))
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Failed to delete orphaned file from storage")))?;
        }
    }

    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true }))))
}
