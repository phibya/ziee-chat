use crate::api;
use aide::{
    axum::{ApiRouter, routing::{get_with, post_with}},
};
use schemars::JsonSchema;

/// Public utility routes (no authentication required)
pub fn utils_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/utils/test-proxy",
        post_with(api::configuration::test_proxy_connection_public, |op| {
            op.description("Test the proxy connection")
                .id("Utils.testProxy")
                .tag("utils")
        }),
    )
}
