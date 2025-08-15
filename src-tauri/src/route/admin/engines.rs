use axum::{middleware, routing::get, Router};

use crate::api::{engines::list_engines, middleware::auth_middleware};

pub fn admin_engine_routes() -> Router {
    Router::new()
        .route("/engines", get(list_engines))
        .layer(middleware::from_fn(auth_middleware))
}
