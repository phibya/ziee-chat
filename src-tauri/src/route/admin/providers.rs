use crate::api;
use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with, put_with}},
};
use axum::middleware;

pub fn admin_provider_routes() -> ApiRouter {
    ApiRouter::new()
        // Model provider routes
        .api_route(
            "/providers",
            get_with(api::providers::list_providers, |op| {
                op.description("List all providers")
                    .id("Admin.listProviders")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_read_middleware)),
        )
        .api_route(
            "/providers",
            post_with(api::providers::create_provider, |op| {
                op.description("Create a new provider")
                    .id("Admin.createProvider")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_create_middleware)),
        )
        .api_route(
            "/providers/{provider_id}",
            get_with(api::providers::get_provider, |op| {
                op.description("Get a specific provider")
                    .id("Admin.getProvider")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_read_middleware)),
        )
        .api_route(
            "/providers/{provider_id}",
            put_with(api::providers::update_provider, |op| {
                op.description("Update a provider")
                    .id("Admin.updateProvider")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_edit_middleware)),
        )
        .api_route(
            "/providers/{provider_id}",
            delete_with(api::providers::delete_provider, |op| {
                op.description("Delete a provider")
                    .id("Admin.deleteProvider")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_delete_middleware)),
        )
        .api_route(
            "/providers/{provider_id}/groups",
            get_with(api::providers::get_provider_groups, |op| {
                op.description("Get groups assigned to a provider")
                    .id("Admin.getProviderGroups")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_read_middleware)),
        )
        .api_route(
            "/providers/{provider_id}/models",
            get_with(api::models::list_provider_models, |op| {
                op.description("List models for a provider")
                    .id("Admin.listProviderModels")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_read_middleware)),
        )
        .api_route(
            "/devices",
            get_with(api::providers::get_available_devices, |op| {
                op.description("Get available devices")
                    .id("Admin.getAvailableDevices")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::providers_read_middleware)),
        )
}
