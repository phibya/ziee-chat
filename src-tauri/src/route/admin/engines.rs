use crate::api::{
    engines::{list_engines, EngineInfo},
    middleware::auth_middleware,
};
use aide::axum::{routing::get_with, ApiRouter};
use axum::{middleware, Json};

pub fn admin_engine_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/engines",
        get_with(list_engines, |op| {
            op.description("List all available ML inference engines")
                .id("Admin.listEngines")
                .tag("admin")
                .response::<200, Json<Vec<EngineInfo>>>()
        })
        .layer(middleware::from_fn(auth_middleware)),
    )
}
