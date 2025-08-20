use crate::api;
use crate::api::configuration::{
    DefaultLanguageResponse, NgrokSettingsResponse, NgrokStatusResponse, ProxySettingsResponse,
    UserRegistrationStatusResponse,
};
use aide::axum::{
    routing::{get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_config_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/config/user-registration",
            get_with(
                api::configuration::get_user_registration_status_admin,
                |op| {
                    op.description("Get user registration status (admin)")
                        .id("Admin.getUserRegistrationStatus")
                        .tag("admin")
                        .response::<200, Json<UserRegistrationStatusResponse>>()
                },
            )
            .layer(middleware::from_fn(
                api::middleware::config_user_registration_read_middleware,
            )),
        )
        .api_route(
            "/config/user-registration",
            put_with(api::configuration::update_user_registration_status, |op| {
                op.description("Update user registration status (admin)")
                    .id("Admin.updateUserRegistrationStatus")
                    .tag("admin")
                    .response::<200, Json<UserRegistrationStatusResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_user_registration_edit_middleware,
            )),
        )
        .api_route(
            "/config/default-language",
            get_with(api::configuration::get_default_language_admin, |op| {
                op.description("Get default language (admin)")
                    .id("Admin.getDefaultLanguage")
                    .tag("admin")
                    .response::<200, Json<DefaultLanguageResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_appearance_read_middleware,
            )),
        )
        .api_route(
            "/config/default-language",
            put_with(api::configuration::update_default_language, |op| {
                op.description("Update default language (admin)")
                    .id("Admin.updateDefaultLanguage")
                    .tag("admin")
                    .response::<200, Json<DefaultLanguageResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_appearance_edit_middleware,
            )),
        )
        .api_route(
            "/config/proxy",
            get_with(api::configuration::get_proxy_settings, |op| {
                op.description("Get proxy settings (admin)")
                    .id("Admin.getProxySettings")
                    .tag("admin")
                    .response::<200, Json<ProxySettingsResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_proxy_read_middleware,
            )),
        )
        .api_route(
            "/config/proxy",
            put_with(api::configuration::update_proxy_settings, |op| {
                op.description("Update proxy settings (admin)")
                    .id("Admin.updateProxySettings")
                    .tag("admin")
                    .response::<200, Json<ProxySettingsResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_proxy_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok",
            get_with(api::configuration::get_ngrok_settings_handler, |op| {
                op.description("Get Ngrok settings (admin)")
                    .id("Admin.getNgrokSettings")
                    .tag("admin")
                    .response::<200, Json<NgrokSettingsResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_ngrok_read_middleware,
            )),
        )
        .api_route(
            "/config/ngrok",
            put_with(api::configuration::update_ngrok_settings, |op| {
                op.description("Update Ngrok settings (admin)")
                    .id("Admin.updateNgrokSettings")
                    .tag("admin")
                    .response::<200, Json<NgrokSettingsResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_ngrok_edit_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/start",
            post_with(api::configuration::start_ngrok_tunnel, |op| {
                op.description("Start Ngrok tunnel (admin)")
                    .id("Admin.startNgrokTunnel")
                    .tag("admin")
                    .response::<200, Json<NgrokStatusResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_ngrok_start_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/stop",
            post_with(api::configuration::stop_ngrok_tunnel, |op| {
                op.description("Stop Ngrok tunnel (admin)")
                    .id("Admin.stopNgrokTunnel")
                    .tag("admin")
                    .response::<200, Json<NgrokStatusResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_ngrok_stop_middleware,
            )),
        )
        .api_route(
            "/config/ngrok/status",
            get_with(api::configuration::get_ngrok_status, |op| {
                op.description("Get Ngrok status (admin)")
                    .id("Admin.getNgrokStatus")
                    .tag("admin")
                    .response::<200, Json<NgrokStatusResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::config_ngrok_read_middleware,
            )),
        )
        .api_route(
            "/config/user/password",
            put_with(api::configuration::update_user_password, |op| {
                op.description("Update user account password")
                    .id("User.updateAccountPassword")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::authenticated_middleware,
            )),
        )
}
