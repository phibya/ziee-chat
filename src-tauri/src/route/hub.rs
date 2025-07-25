use crate::api::hub;
use axum::{routing::get, Router};

pub fn hub_routes() -> Router {
    Router::new()
        .route("/api/hub/data", get(hub::get_hub_data))
        .route(
            "/api/hub/refresh",
            axum::routing::post(hub::refresh_hub_data),
        )
        .route("/api/hub/version", get(hub::get_hub_version))
}
