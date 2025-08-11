use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_repository_routes() -> Router {
    Router::new()
        // Repository routes
        .route(
            "/repositories",
            get(api::repositories::list_repositories).layer(middleware::from_fn(
                api::middleware::repositories_read_middleware,
            )),
        )
        .route(
            "/repositories",
            post(api::repositories::create_repository).layer(middleware::from_fn(
                api::middleware::repositories_create_middleware,
            )),
        )
        .route(
            "/repositories/{repository_id}",
            get(api::repositories::get_repository).layer(middleware::from_fn(
                api::middleware::repositories_read_middleware,
            )),
        )
        .route(
            "/repositories/{repository_id}",
            put(api::repositories::update_repository).layer(middleware::from_fn(
                api::middleware::repositories_edit_middleware,
            )),
        )
        .route(
            "/repositories/{repository_id}",
            delete(api::repositories::delete_repository).layer(middleware::from_fn(
                api::middleware::repositories_delete_middleware,
            )),
        )
        .route(
            "/repositories/test",
            post(api::repositories::test_repository_connection).layer(middleware::from_fn(
                api::middleware::repositories_read_middleware,
            )),
        )
}