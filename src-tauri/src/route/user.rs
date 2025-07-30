use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn user_routes() -> Router {
    Router::new()
        .route("/api/user/greet", post(api::user::greet))
        // User settings routes
        .route(
            "/api/user/settings",
            get(api::user_settings::get_user_settings).layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .route(
            "/api/user/settings",
            post(api::user_settings::set_user_setting).layer(middleware::from_fn(
                api::middleware::settings_edit_middleware,
            )),
        )
        .route(
            "/api/user/settings/{key}",
            get(api::user_settings::get_user_setting).layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .route(
            "/api/user/settings/{key}",
            delete(api::user_settings::delete_user_setting).layer(middleware::from_fn(
                api::middleware::settings_delete_middleware,
            )),
        )
        .route(
            "/api/user/settings/all",
            delete(api::user_settings::delete_all_user_settings).layer(middleware::from_fn(
                api::middleware::settings_delete_middleware,
            )),
        )
        // Assistant routes - User endpoints
        .route(
            "/api/assistants",
            get(api::assistants::list_assistants)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants",
            post(api::assistants::create_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            get(api::assistants::get_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            put(api::assistants::update_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/{assistant_id}",
            delete(api::assistants::delete_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        .route(
            "/api/assistants/default",
            get(api::assistants::get_default_assistant)
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        )
        // Provider routes - User endpoints (active providers/models only)
        .route(
            "/api/providers",
            get(api::providers::list_enabled_providers).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .route(
            "/api/providers/{provider_id}/models",
            get(api::models::list_enabled_provider_models).layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
}
