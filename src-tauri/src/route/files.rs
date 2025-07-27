use crate::api;
use axum::routing::{delete, get, post};
use axum::{middleware, Router};

pub fn file_routes() -> Router {
    Router::new()
        // General file operations
        .route(
            "/api/files/upload",
            post(api::files::upload_file)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/files/{id}",
            get(api::files::get_file)
                .delete(api::files::delete_file)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/files/{id}/download",
            get(api::files::download_file)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/files/{id}/download-token",
            post(api::files::generate_download_token)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/files/{id}/download-with-token",
            get(api::files::download_file_with_token),
        )
        .route(
            "/api/files/{id}/preview",
            get(api::files::get_file_preview)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        
        // Project file operations
        .route(
            "/api/projects/{id}/files",
            post(api::files::upload_project_file)
                .get(api::files::list_project_files)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        
        // Message file operations
        .route(
            "/api/messages/{id}/files",
            get(api::files::list_message_files)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/files/{file_id}/messages/{message_id}",
            delete(api::files::remove_file_from_message)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
}