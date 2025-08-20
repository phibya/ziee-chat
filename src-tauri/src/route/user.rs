use crate::api;
use crate::api::user_settings::UserSettingsDeletionResponse;
use crate::database::models::{
    Assistant, AssistantListResponse, Model, ProviderListResponse, UserSetting,
    UserSettingsResponse,
};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn user_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/user/greet",
            post_with(api::user::greet, |op| {
                op.description("User greeting endpoint")
                    .id("User.greet")
                    .tag("user")
                    .response::<200, Json<String>>()
            }),
        )
        // User settings routes
        .api_route(
            "/user/settings",
            get_with(api::user_settings::get_user_settings, |op| {
                op.description("Get all user settings")
                    .id("UserSettings.getUserSettings")
                    .tag("user-settings")
                    .response::<200, Json<UserSettingsResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .api_route(
            "/user/settings",
            post_with(api::user_settings::set_user_setting, |op| {
                op.description("Set user setting")
                    .id("UserSettings.setUserSetting")
                    .tag("user-settings")
                    .response::<200, Json<UserSetting>>()
            })
            .layer(middleware::from_fn(
                api::middleware::settings_edit_middleware,
            )),
        )
        .api_route(
            "/user/settings/{key}",
            get_with(api::user_settings::get_user_setting, |op| {
                op.description("Get specific user setting")
                    .id("UserSettings.getUserSetting")
                    .tag("user-settings")
                    .response::<200, Json<UserSetting>>()
            })
            .layer(middleware::from_fn(
                api::middleware::settings_read_middleware,
            )),
        )
        .api_route(
            "/user/settings/{key}",
            delete_with(api::user_settings::delete_user_setting, |op| {
                op.description("Delete user setting")
                    .id("UserSettings.deleteUserSetting")
                    .tag("user-settings")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::settings_delete_middleware,
            )),
        )
        .api_route(
            "/user/settings/all",
            delete_with(api::user_settings::delete_all_user_settings, |op| {
                op.description("Delete all user settings")
                    .id("UserSettings.deleteAllUserSettings")
                    .tag("user-settings")
                    .response::<200, Json<UserSettingsDeletionResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::settings_delete_middleware,
            )),
        )
        // Assistant routes - User endpoints
        .api_route(
            "/assistants",
            get_with(api::assistants::list_assistants, |op| {
                op.description("List user assistants")
                    .id("Assistants.listAssistants")
                    .tag("assistants")
                    .response::<200, Json<AssistantListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_read_middleware)),
        )
        .api_route(
            "/assistants",
            post_with(api::assistants::create_assistant, |op| {
                op.description("Create new assistant")
                    .id("Assistants.createAssistant")
                    .tag("assistants")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_create_middleware)),
        )
        .api_route(
            "/assistants/{assistant_id}",
            get_with(api::assistants::get_assistant, |op| {
                op.description("Get assistant by ID")
                    .id("Assistants.getAssistant")
                    .tag("assistants")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_read_middleware)),
        )
        .api_route(
            "/assistants/{assistant_id}",
            put_with(api::assistants::update_assistant, |op| {
                op.description("Update assistant")
                    .id("Assistants.updateAssistant")
                    .tag("assistants")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_edit_middleware)),
        )
        .api_route(
            "/assistants/{assistant_id}",
            delete_with(api::assistants::delete_assistant, |op| {
                op.description("Delete assistant")
                    .id("Assistants.deleteAssistant")
                    .tag("assistants")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_delete_middleware)),
        )
        .api_route(
            "/assistants/default",
            get_with(api::assistants::get_default_assistant, |op| {
                op.description("Get default assistant")
                    .id("Assistants.getDefaultAssistant")
                    .tag("assistants")
                    .response::<200, Json<Assistant>>()
            })
            .layer(middleware::from_fn(api::middleware::assistants_read_middleware)),
        )
        // Provider routes - User endpoints (active providers/models only)
        .api_route(
            "/providers",
            get_with(api::providers::list_enabled_providers, |op| {
                op.description("List enabled providers")
                    .id("Providers.listEnabledProviders")
                    .tag("providers")
                    .response::<200, Json<ProviderListResponse>>()
            })
            .layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
        .api_route(
            "/providers/{provider_id}/models",
            get_with(api::models::list_enabled_provider_models, |op| {
                op.description("List enabled models for provider")
                    .id("Models.listEnabledProviderModels")
                    .tag("models")
                    .response::<200, Json<Vec<Model>>>()
            })
            .layer(middleware::from_fn(
                api::middleware::providers_read_middleware,
            )),
        )
}
