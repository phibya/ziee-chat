use axum::{routing::get, Router};

use crate::api::hardware::{get_hardware_info, subscribe_hardware_usage};

pub fn hardware_routes() -> Router {
    Router::new()
        .route("/hardware", get(get_hardware_info))
        .route("/hardware/usage-stream", get(subscribe_hardware_usage))
}