use crate::api;
use crate::api::files::{DownloadTokenResponse, FileOperationSuccessResponse};
use crate::database::models::file::UploadFileResponse;
use crate::database::models::file::{File, FileListResponse};
use crate::route::helper::BlobType;
use aide::axum::{
    routing::{delete_with, get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn file_routes() -> ApiRouter {
    ApiRouter::new()
        // General file operations
        .api_route(
            "/files/upload",
            post_with(api::files::upload_file, |op| {
                op.description("Upload a new file")
                    .id("Files.uploadFile")
                    .tag("files")
                    .response::<200, Json<UploadFileResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}",
            get_with(api::files::get_file, |op| {
                op.description("Get file metadata by ID")
                    .id("Files.getFile")
                    .tag("files")
                    .response::<200, Json<File>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}",
            delete_with(api::files::delete_file, |op| {
                op.description("Delete file by ID")
                    .id("Files.deleteFile")
                    .tag("files")
                    .response::<200, Json<FileOperationSuccessResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}/download",
            get_with(api::files::download_file, |op| {
                op.description("Download file by ID")
                    .id("Files.downloadFile")
                    .tag("files")
                    .response::<200, Json<BlobType>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}/download-token",
            post_with(api::files::generate_download_token, |op| {
                op.description("Generate download token for file")
                    .id("Files.generateDownloadToken")
                    .tag("files")
                    .response::<200, Json<DownloadTokenResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}/download-with-token",
            get_with(api::files::download_file_with_token, |op| {
                op.description("Download file using token (no auth required)")
                    .id("Files.downloadFileWithToken")
                    .tag("files")
                    .response::<200, Json<BlobType>>()
            }),
        )
        .api_route(
            "/files/{file_id}/preview",
            get_with(api::files::get_file_preview, |op| {
                op.description("Get file preview by ID")
                    .id("Files.getFilePreview")
                    .tag("files")
                    .response::<200, Json<BlobType>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        // Project file operations
        .api_route(
            "/projects/{project_id}/files",
            post_with(api::files::upload_project_file, |op| {
                op.description("Upload file to project")
                    .id("Files.uploadProjectFile")
                    .tag("files")
                    .response::<200, Json<UploadFileResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/projects/{project_id}/files",
            get_with(api::files::list_project_files, |op| {
                op.description("List files in project")
                    .id("Files.listProjectFiles")
                    .tag("files")
                    .response::<200, Json<FileListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        // Message file operations
        .api_route(
            "/messages/{message_id}/files",
            get_with(api::files::list_message_files, |op| {
                op.description("List files attached to message")
                    .id("Files.listMessageFiles")
                    .tag("files")
                    .response::<200, Json<Vec<File>>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .api_route(
            "/files/{file_id}/messages/{message_id}",
            delete_with(api::files::remove_file_from_message, |op| {
                op.description("Remove file from message")
                    .id("Files.removeFileFromMessage")
                    .tag("files")
                    .response::<200, Json<FileOperationSuccessResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
}
