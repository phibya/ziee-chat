use crate::api;
use axum::routing::{get, post};
use axum::Router;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth/init", get(api::auth::check_init_status))
        .route("/auth/setup", post(api::auth::init_app))
        .route("/auth/login", post(api::auth::login))
        .route("/auth/register", post(api::auth::register))
}

pub fn protected_auth_routes() -> Router {
    Router::new()
        .route("/auth/logout", post(api::auth::logout))
        .route("/auth/me", get(api::auth::me))
}
