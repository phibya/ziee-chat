use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_model_routes() -> Router {
    Router::new()
        // Model routes
        .route(
            "/api/admin/providers/{provider_id}/models",
            post(api::models::create_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            get(api::models::get_model).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            put(api::models::update_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}",
            delete(api::models::delete_model).layer(middleware::from_fn(
                api::middleware::providers_delete_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}/start",
            post(api::models::start_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}/stop",
            post(api::models::stop_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}/enable",
            post(api::models::enable_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/{model_id}/disable",
            post(api::models::disable_model).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        // Model uploads
        .route(
            "/api/admin/uploaded-models/upload-and-commit",
            post(api::model_uploads::upload_multiple_files_and_commit).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/models/initiate-repository-download",
            post(api::model_uploads::initiate_repository_download).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
}