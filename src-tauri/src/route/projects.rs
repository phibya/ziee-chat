use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn project_routes() -> Router {
    Router::new()
        .route(
            "/api/projects",
            get(api::projects::list_projects)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects",
            post(api::projects::create_project)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}",
            get(api::projects::get_project)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}",
            put(api::projects::update_project)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}",
            delete(api::projects::delete_project)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}/documents",
            post(api::projects::upload_document)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}/documents/{document_id}",
            delete(api::projects::delete_document)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}/conversations/{conversation_id}",
            post(api::projects::link_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/projects/{project_id}/conversations/{conversation_id}",
            delete(api::projects::unlink_conversation)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
}