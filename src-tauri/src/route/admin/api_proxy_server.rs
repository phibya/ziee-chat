use crate::api::api_proxy_server::*;
use crate::database::models::api_proxy_server_model::*;
use aide::axum::{
    routing::{get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_api_proxy_server_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/api-proxy-server/config",
            get_with(get_proxy_config, |op| {
                op.description("Get API proxy server configuration")
                    .id("Admin.getApiProxyServerConfig")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerConfig>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_read_middleware))
            .put_with(update_proxy_config, |op| {
                op.description("Update API proxy server configuration")
                    .id("Admin.updateApiProxyServerConfig")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerConfig>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/models",
            get_with(list_proxy_models, |op| {
                op.description("List API proxy server models")
                    .id("Admin.listApiProxyServerModels")
                    .tag("admin")
                    .response::<200, Json<Vec<ApiProxyServerModel>>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_read_middleware))
            .post_with(add_model_to_proxy, |op| {
                op.description("Add model to API proxy server")
                    .id("Admin.addModelToApiProxyServer")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerModel>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/models/{model_id}",
            put_with(update_proxy_model, |op| {
                op.description("Update API proxy server model")
                    .id("Admin.updateApiProxyServerModel")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerModel>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware))
            .delete_with(remove_model_from_proxy, |op| {
                op.description("Remove model from API proxy server")
                    .id("Admin.removeModelFromApiProxyServer")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/trusted-hosts",
            get_with(list_trusted_hosts, |op| {
                op.description("List API proxy server trusted hosts")
                    .id("Admin.listApiProxyServerTrustedHosts")
                    .tag("admin")
                    .response::<200, Json<Vec<ApiProxyServerTrustedHost>>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_read_middleware))
            .post_with(add_trusted_host, |op| {
                op.description("Add trusted host to API proxy server")
                    .id("Admin.addApiProxyServerTrustedHost")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerTrustedHost>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/trusted-hosts/{host_id}",
            put_with(update_trusted_host, |op| {
                op.description("Update API proxy server trusted host")
                    .id("Admin.updateApiProxyServerTrustedHost")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerTrustedHost>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware))
            .delete_with(remove_trusted_host, |op| {
                op.description("Remove trusted host from API proxy server")
                    .id("Admin.removeApiProxyServerTrustedHost")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/status",
            get_with(get_proxy_status, |op| {
                op.description("Get API proxy server status")
                    .id("Admin.getApiProxyServerStatus")
                    .tag("admin")
                    .response::<200, Json<ApiProxyServerStatus>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_read_middleware)),
        )
        .api_route(
            "/api-proxy-server/start",
            post_with(start_proxy_server, |op| {
                op.description("Start API proxy server")
                    .id("Admin.startApiProxyServer")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_start_middleware)),
        )
        .api_route(
            "/api-proxy-server/stop",
            post_with(stop_proxy_server, |op| {
                op.description("Stop API proxy server")
                    .id("Admin.stopApiProxyServer")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_stop_middleware)),
        )
        .api_route(
            "/api-proxy-server/reload/models",
            post_with(reload_proxy_models, |op| {
                op.description("Reload API proxy server models")
                    .id("Admin.reloadApiProxyServerModels")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/reload/trusted-hosts",
            post_with(reload_proxy_trusted_hosts, |op| {
                op.description("Reload API proxy server trusted hosts")
                    .id("Admin.reloadApiProxyServerTrustedHosts")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_configure_middleware)),
        )
        .api_route(
            "/api-proxy-server/logs/stream",
            get_with(subscribe_proxy_logs, |op| {
                op.description("Subscribe to API proxy server logs stream")
                    .id("Admin.subscribeApiProxyServerLogs")
                    .tag("admin")
                // SSE streams don't need response type specification in aide
            })
            .layer(middleware::from_fn(crate::api::middleware::api_proxy_read_middleware)),
        )
}
