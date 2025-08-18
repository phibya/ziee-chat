use crate::api::api_proxy_server::*;
use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with, put_with}},
};

pub fn admin_api_proxy_server_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/api-proxy-server/config",
            get_with(get_proxy_config, |op| {
                op.description("Get API proxy server configuration")
                    .id("Admin.getApiProxyServerConfig")
                    .tag("admin")
            }).put_with(update_proxy_config, |op| {
                op.description("Update API proxy server configuration")
                    .id("Admin.updateApiProxyServerConfig")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/models",
            get_with(list_proxy_models, |op| {
                op.description("List API proxy server models")
                    .id("Admin.listApiProxyServerModels")
                    .tag("admin")
            }).post_with(add_model_to_proxy, |op| {
                op.description("Add model to API proxy server")
                    .id("Admin.addModelToApiProxyServer")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/models/{model_id}",
            put_with(update_proxy_model, |op| {
                op.description("Update API proxy server model")
                    .id("Admin.updateApiProxyServerModel")
                    .tag("admin")
            }).delete_with(remove_model_from_proxy, |op| {
                op.description("Remove model from API proxy server")
                    .id("Admin.removeModelFromApiProxyServer")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/trusted-hosts",
            get_with(list_trusted_hosts, |op| {
                op.description("List API proxy server trusted hosts")
                    .id("Admin.listApiProxyServerTrustedHosts")
                    .tag("admin")
            }).post_with(add_trusted_host, |op| {
                op.description("Add trusted host to API proxy server")
                    .id("Admin.addApiProxyServerTrustedHost")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/trusted-hosts/{host_id}",
            put_with(update_trusted_host, |op| {
                op.description("Update API proxy server trusted host")
                    .id("Admin.updateApiProxyServerTrustedHost")
                    .tag("admin")
            }).delete_with(remove_trusted_host, |op| {
                op.description("Remove trusted host from API proxy server")
                    .id("Admin.removeApiProxyServerTrustedHost")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/status",
            get_with(get_proxy_status, |op| {
                op.description("Get API proxy server status")
                    .id("Admin.getApiProxyServerStatus")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/start",
            post_with(start_proxy_server, |op| {
                op.description("Start API proxy server")
                    .id("Admin.startApiProxyServer")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/stop",
            post_with(stop_proxy_server, |op| {
                op.description("Stop API proxy server")
                    .id("Admin.stopApiProxyServer")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/reload/models",
            post_with(reload_proxy_models, |op| {
                op.description("Reload API proxy server models")
                    .id("Admin.reloadApiProxyServerModels")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/reload/trusted-hosts",
            post_with(reload_proxy_trusted_hosts, |op| {
                op.description("Reload API proxy server trusted hosts")
                    .id("Admin.reloadApiProxyServerTrustedHosts")
                    .tag("admin")
            }),
        )
        .api_route(
            "/api-proxy-server/logs/stream",
            get_with(subscribe_proxy_logs, |op| {
                op.description("Subscribe to API proxy server logs stream")
                    .id("Admin.subscribeApiProxyServerLogs")
                    .tag("admin")
            }),
        )
}