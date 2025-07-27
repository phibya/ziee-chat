use axum::{
    extract::{Extension, Multipart, Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::middleware::AuthenticatedUser,
    database::{
        models::file::*,
        queries::files,
    },
    utils::file_storage::{FileStorage, extract_extension, get_mime_type_from_extension},
    processing::ProcessingManager,
    get_app_data_dir,
};

// Initialize global file storage and processing manager
use once_cell::sync::Lazy;

static FILE_STORAGE: Lazy<Arc<FileStorage>> = Lazy::new(|| {
    Arc::new(FileStorage::new(&get_app_data_dir()))
});

static PROCESSING_MANAGER: Lazy<Arc<ProcessingManager>> = Lazy::new(|| {
    Arc::new(ProcessingManager::new(FILE_STORAGE.clone()))
});

#[derive(Debug, Deserialize)]
pub struct PreviewParams {
    pub page: Option<u32>,
}

// Initialize file storage on first use
pub async fn initialize_file_storage() -> Result<(), StatusCode> {
    FILE_STORAGE.initialize().await.map_err(|e| {
        eprintln!("Failed to initialize file storage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// Upload file (general)
pub async fn upload_file(
    Extension(user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> Result<Json<UploadFileResponse>, StatusCode> {
    let mut file_data = None;
    let mut filename = String::new();
    let mut file_size = 0u64;

    // Extract multipart data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("");
        
        match field_name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_size = data.len() as u64;
                file_data = Some(data);
            }
            _ => continue,
        }
    }

    let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    if filename.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    process_file_upload(user.user_id, filename, file_data, file_size, None).await
}

// Upload file to project
pub async fn upload_project_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadFileResponse>, StatusCode> {
    let mut file_data = None;
    let mut filename = String::new();
    let mut file_size = 0u64;

    // Extract multipart data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("");
        
        match field_name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_size = data.len() as u64;
                file_data = Some(data);
            }
            _ => continue,
        }
    }

    let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    if filename.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    process_file_upload(user.user_id, filename, file_data, file_size, Some(project_id)).await
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
    FILE_STORAGE.save_file_bytes(&file_path, &file_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculate checksum
    let checksum = FILE_STORAGE.calculate_checksum(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Process file content
    let processing_result = PROCESSING_MANAGER.process_file(&file_path, &mime_type).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save processed content
    if let Some(ref text_content) = processing_result.text_content {
        FILE_STORAGE.save_text_content(file_id, text_content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(ref base64_content) = processing_result.base64_content {
        FILE_STORAGE.save_base64_content(file_id, base64_content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Create file record
    let file_create_data = FileCreateData {
        id: file_id,
        user_id,
        filename,
        file_path: file_path.to_string_lossy().to_string(),
        file_size: file_size as i64,
        mime_type,
        checksum: Some(checksum),
        project_id,
        thumbnail_count: processing_result.thumbnail_count,
        processing_metadata: processing_result.metadata,
    };

    let file = files::create_file(file_create_data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UploadFileResponse { file }))
}

// Get file metadata
pub async fn get_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<File>, StatusCode> {
    let file = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(file))
}

// Download file
pub async fn download_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let file_db = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_path = std::path::Path::new(&file_db.file_path);
    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let file_data = FILE_STORAGE.read_file_bytes(file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let content_type = file_db.mime_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_db.filename),
        ),
    ];

    Ok((headers, file_data).into_response())
}

// Get file preview/thumbnail
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

    let thumbnail_data = FILE_STORAGE.read_file_bytes(&thumbnail_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let headers = [
        (header::CONTENT_TYPE, "image/jpeg".to_string()),
        (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
    ];

    Ok((headers, thumbnail_data).into_response())
}

// Delete file
pub async fn delete_file(
    Extension(user): Extension<AuthenticatedUser>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let file_db = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Delete from database (this will cascade to relationships)
    let deleted = files::delete_file(file_id, user.user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !deleted {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete from filesystem
    let extension = extract_extension(&file_db.filename);
    FILE_STORAGE.delete_file(file_id, Some(&extension)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// List files by project
pub async fn list_project_files(
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<FileListParams>,
) -> Result<Json<FileListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let (files, total) = files::get_files_by_project(project_id, user.user_id, page, per_page)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FileListResponse {
        files,
        total,
        page,
        per_page,
    }))
}

// List files by message
pub async fn list_message_files(
    Extension(user): Extension<AuthenticatedUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Vec<File>>, StatusCode> {
    let files = files::get_files_by_message(message_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(files))
}

// Remove file from message
pub async fn remove_file_from_message(
    Extension(user): Extension<AuthenticatedUser>,
    Path((file_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify file belongs to user
    let file_exists = files::get_file_by_id_and_user(file_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    if !file_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    // Remove relationship
    let removed = files::delete_message_file_relationship(message_id, file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !removed {
        return Err(StatusCode::NOT_FOUND);
    }

    // Check if file is now orphaned
    let has_messages = files::check_file_has_message_associations(file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let has_project = files::check_file_has_project_association(file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If orphaned, delete the file completely
    if !has_messages && !has_project {
        let file_db = files::get_file_by_id(file_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(file_db) = file_db {
            // Delete from database
            files::delete_file(file_id, user.user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Delete from filesystem
            let extension = extract_extension(&file_db.filename);
            FILE_STORAGE.delete_file(file_id, Some(&extension)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(Json(serde_json::json!({ "success": true })))
}