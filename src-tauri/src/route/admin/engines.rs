use crate::api::{engines::list_engines, middleware::auth_middleware};
use aide::{
    axum::{ApiRouter, routing::get_with},
};
use axum::middleware;

pub fn admin_engine_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/engines",
            get_with(list_engines, |op| {
                op.description("List all available ML inference engines")
                    .id("Admin.listEngines")
                    .tag("admin")
            }),
        )
        .layer(middleware::from_fn(auth_middleware))
}
