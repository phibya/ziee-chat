use crate::api;
use axum::routing::get;
use axum::Router;

/// Public configuration routes (no authentication required)
pub fn config_routes() -> Router {
    Router::new()
        .route(
            "/config/user-registration",
            get(api::configuration::get_user_registration_status),
        )
        .route(
            "/config/default-language",
            get(api::configuration::get_default_language_public),
        )
}