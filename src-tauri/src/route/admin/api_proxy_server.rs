use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::api_proxy_server::*;

pub fn admin_api_proxy_server_routes() -> Router {
    Router::new()
        .route("/api-proxy-server/config", get(get_proxy_config).put(update_proxy_config))
        .route("/api-proxy-server/models", get(list_proxy_models).post(add_model_to_proxy))
        .route("/api-proxy-server/models/{model_id}", put(update_proxy_model).delete(remove_model_from_proxy))
        .route("/api-proxy-server/trusted-hosts", get(list_trusted_hosts).post(add_trusted_host))
        .route("/api-proxy-server/trusted-hosts/{host_id}", put(update_trusted_host).delete(remove_trusted_host))
        .route("/api-proxy-server/status", get(get_proxy_status))
        .route("/api-proxy-server/start", post(start_proxy_server))
        .route("/api-proxy-server/stop", post(stop_proxy_server))
        .route("/api-proxy-server/reload/models", post(reload_proxy_models))
        .route("/api-proxy-server/reload/trusted-hosts", post(reload_proxy_trusted_hosts))
        .route("/api-proxy-server/logs/stream", get(subscribe_proxy_logs))
}