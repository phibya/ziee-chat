use crate::api;
use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with, put_with}},
};
use axum::middleware;

pub fn admin_group_routes() -> ApiRouter {
    ApiRouter::new()
        // Admin user group management routes
        .api_route(
            "/groups",
            get_with(api::user_groups::list_user_groups, |op| {
                op.description("List all user groups (admin)")
                    .id("Admin.listGroups")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups",
            post_with(api::user_groups::create_user_group, |op| {
                op.description("Create a new user group")
                    .id("Admin.createGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_create_middleware)),
        )
        .api_route(
            "/groups/{group_id}",
            get_with(api::user_groups::get_user_group, |op| {
                op.description("Get a specific user group")
                    .id("Admin.getGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/{group_id}",
            put_with(api::user_groups::update_user_group, |op| {
                op.description("Update a user group")
                    .id("Admin.updateGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .api_route(
            "/groups/{group_id}",
            delete_with(api::user_groups::delete_user_group, |op| {
                op.description("Delete a user group")
                    .id("Admin.deleteGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_delete_middleware)),
        )
        .api_route(
            "/groups/{group_id}/members",
            get_with(api::user_groups::get_group_members, |op| {
                op.description("Get members of a user group")
                    .id("Admin.getGroupMembers")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/assign",
            post_with(api::user_groups::assign_user_to_group, |op| {
                op.description("Assign a user to a group")
                    .id("Admin.assignUserToGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .api_route(
            "/groups/{user_id}/{group_id}/remove",
            delete_with(api::user_groups::remove_user_from_group, |op| {
                op.description("Remove a user from a group")
                    .id("Admin.removeUserFromGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        // User Group Model Provider relationship routes
        .api_route(
            "/groups/{group_id}/providers",
            get_with(api::user_groups::get_group_providers, |op| {
                op.description("Get providers assigned to a group")
                    .id("Admin.getGroupProviders")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .api_route(
            "/groups/assign-provider",
            post_with(api::user_groups::assign_provider_to_group, |op| {
                op.description("Assign a provider to a group")
                    .id("Admin.assignProviderToGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .api_route(
            "/groups/{group_id}/providers/{provider_id}",
            delete_with(api::user_groups::remove_provider_from_group, |op| {
                op.description("Remove a provider from a group")
                    .id("Admin.removeProviderFromGroup")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .api_route(
            "/user-group-provider-relationships",
            get_with(api::user_groups::list_user_group_provider_relationships, |op| {
                op.description("List all user group provider relationships")
                    .id("Admin.listUserGroupProviderRelationships")
                    .tag("admin")
            }).layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
}
