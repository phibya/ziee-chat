pub mod admin;
mod auth;
mod chat;
mod config;
mod files;
mod helper;
mod hub;
mod projects;
mod user;
mod utils;

use crate::api;
use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::{middleware, Router};
use tower_http::cors::CorsLayer;

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("")
}

pub fn create_rest_router_internal() -> (OpenApi, Router) {
    let mut api = OpenApi::default();

    // API routes with /api prefix
    let api_routes = ApiRouter::new()
        // Public API routes (no authentication required)
        .merge(auth::auth_routes())
        .merge(config::config_routes())
        .merge(utils::utils_routes())
        .merge(hub::hub_routes())
        // Protected API routes requiring authentication
        .merge(
            ApiRouter::new()
                .merge(auth::protected_auth_routes())
                .merge(admin::admin_routes())
                .merge(user::user_routes())
                .merge(chat::chat_routes())
                .merge(projects::project_routes())
                .layer(middleware::from_fn(api::middleware::auth_middleware)),
        );

    // File routes (already have auth middleware applied individually)
    let file_routes = files::file_routes();

    // Combine all routes
    let router = ApiRouter::new()
        .nest("/api", api_routes.merge(file_routes))
        .finish_api_with(&mut api, api_docs)
        .layer(CorsLayer::permissive());

    (api, router)
}

pub fn create_rest_router() -> Router {
    let (_api, router) = create_rest_router_internal();
    router
}
