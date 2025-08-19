use crate::api;
use crate::api::configuration::TestProxyConnectionResponse;
use aide::axum::{routing::post_with, ApiRouter};
use axum::Json;

/// Public utility routes (no authentication required)
pub fn utils_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/utils/test-proxy",
        post_with(api::configuration::test_proxy_connection_public, |op| {
            op.description("Test the proxy connection")
                .id("Utils.testProxy")
                .tag("utils")
                .response::<200, Json<TestProxyConnectionResponse>>()
        }),
    )
}
