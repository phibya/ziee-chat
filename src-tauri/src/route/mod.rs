mod admin;
mod auth;
mod chat;
mod projects;
mod user;

use crate::api;
use axum::routing::get;
use axum::{middleware, Router};
use tower_http::cors::CorsLayer;

pub fn create_rest_router() -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .merge(auth::auth_routes())
        .route(
            "/api/config/user-registration",
            get(api::configuration::get_user_registration_status),
        )
        .route(
            "/api/config/default-language",
            get(api::configuration::get_default_language_public),
        )
        .route("/health", get(|| async { "Tauri + Localhost Plugin OK" }));

    // Protected routes requiring authentication
    let protected_routes = Router::new()
        .merge(auth::protected_auth_routes())
        .merge(admin::admin_routes())
        .merge(user::user_routes())
        .merge(chat::chat_routes())
        .merge(projects::project_routes())
        .layer(middleware::from_fn(api::middleware::auth_middleware));

    // Combine public and protected routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
}
