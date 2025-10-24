use crate::api;
use crate::api::auth_providers::DiscoverProviderResponse;
use crate::database::models::{AuthProvider, TestConnectionResult};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn auth_providers_routes() -> ApiRouter {
    ApiRouter::new()
        // Public endpoints (no authentication required)
        .api_route(
            "/auth-providers/discover",
            get_with(api::auth_providers::discover_provider, |op| {
                op.description("Discover authentication provider for a username/email (home realm discovery)")
                    .id("Auth.discoverProvider")
                    .tag("auth")
                    .response::<200, Json<DiscoverProviderResponse>>()
            }),
        )
        .api_route(
            "/auth-providers/enabled",
            get_with(api::auth_providers::list_enabled_providers, |op| {
                op.description("List all enabled authentication providers (public)")
                    .id("Auth.listEnabledProviders")
                    .tag("auth")
                    .response::<200, Json<Vec<AuthProvider>>>()
            }),
        )
        // Admin endpoints (require authentication and permissions)
        // Auth provider routes
        .api_route(
            "/auth-providers",
            get_with(api::auth_providers::list_auth_providers, |op| {
                op.description("List all authentication providers")
                    .id("Admin.listAuthProviders")
                    .tag("admin")
                    .response::<200, Json<Vec<AuthProvider>>>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_read_middleware,
            )),
        )
        .api_route(
            "/auth-providers",
            post_with(api::auth_providers::create_auth_provider, |op| {
                op.description("Create a new authentication provider")
                    .id("Admin.createAuthProvider")
                    .tag("admin")
                    .response::<201, Json<AuthProvider>>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_create_middleware,
            )),
        )
        .api_route(
            "/auth-providers/{provider_id}",
            get_with(api::auth_providers::get_auth_provider, |op| {
                op.description("Get a specific authentication provider")
                    .id("Admin.getAuthProvider")
                    .tag("admin")
                    .response::<200, Json<AuthProvider>>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_read_middleware,
            )),
        )
        .api_route(
            "/auth-providers/{provider_id}",
            put_with(api::auth_providers::update_auth_provider, |op| {
                op.description("Update an authentication provider")
                    .id("Admin.updateAuthProvider")
                    .tag("admin")
                    .response::<200, Json<AuthProvider>>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_edit_middleware,
            )),
        )
        .api_route(
            "/auth-providers/{provider_id}",
            delete_with(api::auth_providers::delete_auth_provider, |op| {
                op.description("Delete an authentication provider")
                    .id("Admin.deleteAuthProvider")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_delete_middleware,
            )),
        )
        .api_route(
            "/auth-providers/{provider_id}/test",
            post_with(api::auth_providers::test_auth_provider_connection, |op| {
                op.description("Test authentication provider connection")
                    .id("Admin.testAuthProviderConnection")
                    .tag("admin")
                    .response::<200, Json<TestConnectionResult>>()
            })
            .layer(middleware::from_fn(
                api::middleware::auth_providers_test_middleware,
            )),
        )
}
