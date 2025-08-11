use crate::api;
use axum::routing::{get, put};
use axum::{middleware, Router};

pub fn admin_config_routes() -> Router {
    Router::new()
        // Admin configuration routes
        .route(
            "/config/user-registration",
            get(api::configuration::get_user_registration_status_admin).layer(middleware::from_fn(
                api::middleware::config_user_registration_read_middleware,
            )),
        )
        .route(
            "/config/user-registration",
            put(api::configuration::update_user_registration_status).layer(middleware::from_fn(
                api::middleware::config_user_registration_edit_middleware,
            )),
        )
        .route(
            "/config/default-language",
            get(api::configuration::get_default_language_admin).layer(middleware::from_fn(
                api::middleware::config_appearance_read_middleware,
            )),
        )
        .route(
            "/config/default-language",
            put(api::configuration::update_default_language).layer(middleware::from_fn(
                api::middleware::config_appearance_edit_middleware,
            )),
        )
        .route(
            "/config/proxy",
            get(api::configuration::get_proxy_settings).layer(middleware::from_fn(
                api::middleware::config_proxy_read_middleware,
            )),
        )
        .route(
            "/config/proxy",
            put(api::configuration::update_proxy_settings).layer(middleware::from_fn(
                api::middleware::config_proxy_edit_middleware,
            )),
        )
}