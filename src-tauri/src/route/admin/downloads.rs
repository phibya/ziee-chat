use crate::api;
use axum::routing::{delete, get, post};
use axum::{middleware, Router};

pub fn admin_download_routes() -> Router {
    Router::new()
        // Download management routes
        .route(
            "/api/admin/downloads",
            get(api::download_instances::list_all_downloads).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/downloads/{download_id}",
            get(api::download_instances::get_download).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/api/admin/downloads/{download_id}/cancel",
            post(api::download_instances::cancel_download).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/downloads/{download_id}",
            delete(api::download_instances::delete_download).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/api/admin/downloads/subscribe",
            get(api::download_instances::subscribe_download_progress).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
}