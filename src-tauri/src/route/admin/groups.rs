use crate::api;
use crate::database::models::{
    Provider, UserGroup, UserGroupListResponse, UserGroupProviderResponse, UserListResponse,
};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn admin_group_routes() -> ApiRouter {
    ApiRouter::new()
        // Admin user group management routes
        .api_route(
            "/groups",
            get_with(api::user_groups::list_user_groups, |op| {
                op.description("List all user groups (admin)")
                    .id("Admin.listGroups")
                    .tag("admin")
                    .response::<200, Json<UserGroupListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups",
            post_with(api::user_groups::create_user_group, |op| {
                op.description("Create a new user group")
                    .id("Admin.createGroup")
                    .tag("admin")
                    .response::<200, Json<UserGroup>>()
            })
            .layer(middleware::from_fn(
                api::middleware::groups_create_middleware,
            )),
        )
        .api_route(
            "/groups/{group_id}",
            get_with(api::user_groups::get_user_group, |op| {
                op.description("Get a specific user group")
                    .id("Admin.getGroup")
                    .tag("admin")
                    .response::<200, Json<UserGroup>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/{group_id}",
            put_with(api::user_groups::update_user_group, |op| {
                op.description("Update a user group")
                    .id("Admin.updateGroup")
                    .tag("admin")
                    .response::<200, Json<UserGroup>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .api_route(
            "/groups/{group_id}",
            delete_with(api::user_groups::delete_user_group, |op| {
                op.description("Delete a user group")
                    .id("Admin.deleteGroup")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                api::middleware::groups_delete_middleware,
            )),
        )
        .api_route(
            "/groups/{group_id}/members",
            get_with(api::user_groups::get_group_members, |op| {
                op.description("Get members of a user group")
                    .id("Admin.getGroupMembers")
                    .tag("admin")
                    .response::<200, Json<UserListResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/assign",
            post_with(api::user_groups::assign_user_to_group, |op| {
                op.description("Assign a user to a group")
                    .id("Admin.assignUserToGroup")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(api::middleware::groups_assign_users_middleware)),
        )
        .api_route(
            "/groups/{user_id}/{group_id}/remove",
            delete_with(api::user_groups::remove_user_from_group, |op| {
                op.description("Remove a user from a group")
                    .id("Admin.removeUserFromGroup")
                    .tag("admin")
                    .response::<200, ()>()
            })
            .layer(middleware::from_fn(api::middleware::groups_assign_users_middleware)),
        )
        // User Group Model Provider relationship routes
        .api_route(
            "/groups/{group_id}/providers",
            get_with(api::user_groups::get_group_providers, |op| {
                op.description("Get providers assigned to a group")
                    .id("Admin.getGroupProviders")
                    .tag("admin")
                    .response::<200, Json<Vec<Provider>>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/assign-provider",
            post_with(api::user_groups::assign_provider_to_group, |op| {
                op.description("Assign a provider to a group")
                    .id("Admin.assignProviderToGroup")
                    .tag("admin")
                    .response::<200, Json<UserGroupProviderResponse>>()
            })
            .layer(middleware::from_fn(api::middleware::groups_assign_providers_middleware)),
        )
        .api_route(
            "/groups/{group_id}/providers/{provider_id}",
            delete_with(api::user_groups::remove_provider_from_group, |op| {
                op.description("Remove a provider from a group")
                    .id("Admin.removeProviderFromGroup")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(api::middleware::groups_assign_providers_middleware)),
        )
        .api_route(
            "/user-group-provider-relationships",
            get_with(
                api::user_groups::list_user_group_provider_relationships,
                |op| {
                    op.description("List all user group provider relationships")
                        .id("Admin.listUserGroupProviderRelationships")
                        .tag("admin")
                        .response::<200, Json<Vec<UserGroupProviderResponse>>>()
                },
            )
            .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
}
