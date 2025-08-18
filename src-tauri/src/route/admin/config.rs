use crate::api;
use aide::{
    axum::{ApiRouter, routing::{get_with, post_with, put_with}},
};
use axum::middleware;

pub fn admin_config_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/config/user-registration",
            get_with(api::configuration::get_user_registration_status_admin, |op| {
                op.description("Get user registration status (admin)")
                    .id("Admin.getUserRegistrationStatus")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_user_registration_read_middleware,
            )),
        )
        .api_route(
            "/config/user-registration",
            put_with(api::configuration::update_user_registration_status, |op| {
                op.description("Update user registration status (admin)")
                    .id("Admin.updateUserRegistrationStatus")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_user_registration_edit_middleware,
            )),
        )
        .api_route(
            "/config/default-language",
            get_with(api::configuration::get_default_language_admin, |op| {
                op.description("Get default language (admin)")
                    .id("Admin.getDefaultLanguage")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_appearance_read_middleware,
            )),
        )
        .api_route(
            "/config/default-language",
            put_with(api::configuration::update_default_language, |op| {
                op.description("Update default language (admin)")
                    .id("Admin.updateDefaultLanguage")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_appearance_edit_middleware,
            )),
        )
        .api_route(
            "/config/proxy",
            get_with(api::configuration::get_proxy_settings, |op| {
                op.description("Get proxy settings (admin)")
                    .id("Admin.getProxySettings")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_proxy_read_middleware,
            )),
        )
        .api_route(
            "/config/proxy",
            put_with(api::configuration::update_proxy_settings, |op| {
                op.description("Update proxy settings (admin)")
                    .id("Admin.updateProxySettings")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_proxy_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok",
            get_with(api::configuration::get_ngrok_settings_handler, |op| {
                op.description("Get Ngrok settings (admin)")
                    .id("Admin.getNgrokSettings")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_ngrok_read_middleware,
            )),
        )
        .api_route(
            "/config/ngrok",
            put_with(api::configuration::update_ngrok_settings, |op| {
                op.description("Update Ngrok settings (admin)")
                    .id("Admin.updateNgrokSettings")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_ngrok_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/start",
            post_with(api::configuration::start_ngrok_tunnel, |op| {
                op.description("Start Ngrok tunnel (admin)")
                    .id("Admin.startNgrokTunnel")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_ngrok_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/stop",
            post_with(api::configuration::stop_ngrok_tunnel, |op| {
                op.description("Stop Ngrok tunnel (admin)")
                    .id("Admin.stopNgrokTunnel")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_ngrok_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/status",
            get_with(api::configuration::get_ngrok_status, |op| {
                op.description("Get Ngrok status (admin)")
                    .id("Admin.getNgrokStatus")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::config_ngrok_read_middleware,
            )),
        )
        .api_route(
            "/config/user/password",
            put_with(api::configuration::update_user_password, |op| {
                op.description("Update user account password")
                    .id("User.updateAccountPassword")
                    .tag("admin")
            }).layer(middleware::from_fn(
                api::middleware::authenticated_middleware,
            )),
        )
}
