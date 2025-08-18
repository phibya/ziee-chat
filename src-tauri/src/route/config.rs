use crate::api;
use crate::api::configuration::{UserRegistrationStatusResponse, DefaultLanguageResponse};
use aide::{
    axum::{ApiRouter, routing::get_with},
};
use axum::Json;

/// Public configuration routes (no authentication required)
pub fn config_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route("/config/user-registration", get_with(api::configuration::get_user_registration_status, |op| {
            op.description("Get user registration status")
                .id("Config.getUserRegistrationStatus")
                .tag("config")
                .response::<200, Json<UserRegistrationStatusResponse>>()
        }))
        
        .api_route("/config/default-language", get_with(api::configuration::get_default_language_public, |op| {
            op.description("Get default language")
                .id("Config.getDefaultLanguage")
                .tag("config")
                .response::<200, Json<DefaultLanguageResponse>>()
        }))
}
