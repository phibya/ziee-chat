use crate::api::hub;
use axum::{routing::get, Router};

pub fn hub_routes() -> Router {
    Router::new()
        .route("/hub/data", get(hub::get_hub_data))
        .route("/hub/refresh", axum::routing::post(hub::refresh_hub_data))
        .route("/hub/version", get(hub::get_hub_version))
}
