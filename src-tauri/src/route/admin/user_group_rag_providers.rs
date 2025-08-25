use aide::axum::routing::{get_with, post_with, put_with};
use aide::axum::ApiRouter;
use axum::{middleware, Json};

use crate::api::middleware::permissions::{
    rag_admin_providers_create_middleware,
    rag_admin_providers_edit_middleware,
    rag_admin_providers_read_middleware,
};
use crate::api::rag::admin_providers::{
    assign_rag_provider_to_group_handler,
    list_user_group_rag_provider_relationships_handler,
    remove_rag_provider_from_group_handler,
    update_group_rag_provider_permissions_handler,
};
use crate::database::models::UserGroupRAGProviderResponse;

pub fn admin_user_group_rag_provider_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/user-groups/{group_id}/rag-providers",
            post_with(assign_rag_provider_to_group_handler, |op| {
                op.description("Assign RAG provider to user group")
                    .id("Admin.assignRagProviderToGroup")
                    .tag("admin")
                    .response::<201, Json<UserGroupRAGProviderResponse>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_create_middleware)),
        )
        .api_route(
            "/user-groups/{group_id}/rag-providers/{provider_id}",
            put_with(update_group_rag_provider_permissions_handler, |op| {
                op.description("Update RAG provider permissions for user group")
                    .id("Admin.updateGroupRagProviderPermissions")
                    .tag("admin")
                    .response::<200, Json<UserGroupRAGProviderResponse>>()
            })
            .delete_with(remove_rag_provider_from_group_handler, |op| {
                op.description("Remove RAG provider from user group")
                    .id("Admin.removeRagProviderFromGroup")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(rag_admin_providers_edit_middleware)),
        )
        .api_route(
            "/user-group-rag-providers",
            get_with(list_user_group_rag_provider_relationships_handler, |op| {
                op.description("List all user group RAG provider relationships")
                    .id("Admin.listUserGroupRagProviderRelationships")
                    .tag("admin")
                    .response::<200, Json<Vec<UserGroupRAGProviderResponse>>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_read_middleware)),
        )
}