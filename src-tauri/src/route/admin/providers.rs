use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_provider_routes() -> Router {
    Router::new()
        // Model provider routes
        .route(
            "/providers",
            get(api::providers::list_providers).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/providers",
            post(api::providers::create_provider).layer(middleware::from_fn(
                api::middleware::providers_create_middleware,
            )),
        )
        .route(
            "/providers/{provider_id}",
            get(api::providers::get_provider).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/providers/{provider_id}",
            put(api::providers::update_provider).layer(middleware::from_fn(
                api::middleware::providers_edit_middleware,
            )),
        )
        .route(
            "/providers/{provider_id}",
            delete(api::providers::delete_provider).layer(middleware::from_fn(
                api::middleware::providers_delete_middleware,
            )),
        )
        .route(
            "/providers/{provider_id}/groups",
            get(api::providers::get_provider_groups).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/providers/{provider_id}/models",
            get(api::models::list_provider_models).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/devices",
            get(api::providers::get_available_devices).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
}
