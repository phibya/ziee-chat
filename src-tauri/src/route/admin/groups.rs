use crate::api;
use axum::routing::{delete, get, post, put};
use axum::{middleware, Router};

pub fn admin_group_routes() -> Router {
    Router::new()
        // Admin user group management routes
        .route(
            "/api/admin/groups",
            get(api::user_groups::list_user_groups)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups",
            post(api::user_groups::create_user_group).layer(middleware::from_fn(
                api::middleware::groups_create_middleware,
            )),
        )
        .route(
            "/api/admin/groups/{group_id}",
            get(api::user_groups::get_user_group)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}",
            put(api::user_groups::update_user_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}",
            delete(api::user_groups::delete_user_group).layer(middleware::from_fn(
                api::middleware::groups_delete_middleware,
            )),
        )
        .route(
            "/api/admin/groups/{group_id}/members",
            get(api::user_groups::get_group_members)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/assign",
            post(api::user_groups::assign_user_to_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{user_id}/{group_id}/remove",
            delete(api::user_groups::remove_user_from_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        // User Group Model Provider relationship routes
        .route(
            "/api/admin/groups/{group_id}/providers",
            get(api::user_groups::get_group_providers)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
        .route(
            "/api/admin/groups/assign-model-provider",
            post(api::user_groups::assign_provider_to_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/groups/{group_id}/providers/{provider_id}",
            delete(api::user_groups::remove_provider_from_group)
                .layer(middleware::from_fn(api::middleware::groups_edit_middleware)),
        )
        .route(
            "/api/admin/user-group-model-provider-relationships",
            get(api::user_groups::list_user_group_provider_relationships)
                .layer(middleware::from_fn(api::middleware::groups_read_middleware)),
        )
}