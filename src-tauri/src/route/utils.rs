use crate::api;
use axum::routing::post;
use axum::Router;

/// Public utility routes (no authentication required)
pub fn utils_routes() -> Router {
    Router::new().route(
        "/utils/test-proxy",
        post(api::configuration::test_proxy_connection_public),
    )
}
