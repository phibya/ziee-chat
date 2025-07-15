use crate::api;
use axum::routing::{get, post};
use axum::Router;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/api/auth/init", get(api::auth::check_init_status))
        .route("/api/auth/setup", post(api::auth::init_app))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/register", post(api::auth::register))
}

pub fn protected_auth_routes() -> Router {
    Router::new()
        .route("/api/auth/logout", post(api::auth::logout))
        .route("/api/auth/me", get(api::auth::me))
}
