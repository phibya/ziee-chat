pub mod admin;
mod auth;
mod chat;
mod config;
mod files;
mod hub;
mod projects;
mod user;
mod utils;

use crate::api;
use axum::routing::get;
use axum::{middleware, Router};
use tower_http::cors::CorsLayer;

pub fn create_rest_router() -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .merge(auth::auth_routes())
        .merge(config::config_routes())
        .merge(utils::utils_routes())
        .merge(hub::hub_routes())
        .route("/health", get(|| async { "Tauri + Localhost Plugin OK" }));

    // Protected routes requiring authentication
    let protected_routes = Router::new()
        .merge(auth::protected_auth_routes())
        .merge(admin::admin_routes())
        .merge(user::user_routes())
        .merge(chat::chat_routes())
        .merge(projects::project_routes())
        .layer(middleware::from_fn(api::middleware::auth_middleware));

    // File routes (already have auth middleware applied individually)
    let file_routes = files::file_routes();

    // Combine public and protected routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(file_routes)
        .layer(CorsLayer::permissive())
}
