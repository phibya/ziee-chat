use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use axum::{middleware, Json};

use crate::api::middleware::permissions::{
    rag_admin_instances_create_middleware,
    rag_admin_instances_delete_middleware,
    rag_admin_instances_edit_middleware,
    rag_admin_instances_read_middleware,
};
use crate::api::rag::admin_instances::{
    create_system_rag_instance_handler,
    delete_system_rag_instance_handler,
    get_system_rag_instance_handler,
    list_system_rag_instances_handler,
    update_system_rag_instance_handler,
};
use crate::database::models::{RAGInstance, RAGInstanceListResponse};

pub fn admin_rag_instance_routes() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/providers/{provider_id}/instances",
            get_with(list_system_rag_instances_handler, |op| {
                op.description("List system RAG instances for provider")
                    .id("Admin.listSystemRagInstances")
                    .tag("admin")
                    .response::<200, Json<RAGInstanceListResponse>>()
            })
            .layer(middleware::from_fn(rag_admin_instances_read_middleware))
            .post_with(create_system_rag_instance_handler, |op| {
                op.description("Create system RAG instance for provider")
                    .id("Admin.createSystemRagInstance")
                    .tag("admin")
                    .response::<201, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(rag_admin_instances_create_middleware)),
        )
        .api_route(
            "/instances/{instance_id}",
            get_with(get_system_rag_instance_handler, |op| {
                op.description("Get system RAG instance by ID")
                    .id("Admin.getSystemRagInstance")
                    .tag("admin")
                    .response::<200, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(rag_admin_instances_read_middleware))
            .put_with(update_system_rag_instance_handler, |op| {
                op.description("Update system RAG instance")
                    .id("Admin.updateSystemRagInstance")
                    .tag("admin")
                    .response::<200, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(rag_admin_instances_edit_middleware))
            .delete_with(delete_system_rag_instance_handler, |op| {
                op.description("Delete system RAG instance")
                    .id("Admin.deleteSystemRagInstance")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(rag_admin_instances_delete_middleware)),
        )
}