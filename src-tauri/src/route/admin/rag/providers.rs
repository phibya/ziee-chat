use crate::api::middleware::permissions::{
    rag_admin_providers_create_middleware, rag_admin_providers_delete_middleware,
    rag_admin_providers_edit_middleware, rag_admin_providers_read_middleware,
};
use crate::api::rag::providers;
use crate::database::models::{RAGProvider, RAGProviderListResponse};
use aide::axum::{
    routing::{get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};

/// Admin routes for RAG provider management
pub fn admin_rag_provider_routes() -> ApiRouter {
    ApiRouter::new()
        // RAG Provider Management (Admin only)
        .api_route(
            "/providers",
            get_with(providers::list_rag_providers, |op| {
                op.description("List all RAG providers")
                    .id("Admin.listRagProviders")
                    .tag("admin")
                    .response::<200, Json<RAGProviderListResponse>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_read_middleware))
            .post_with(providers::create_rag_provider, |op| {
                op.description("Create new RAG provider")
                    .id("Admin.createRagProvider")
                    .tag("admin")
                    .response::<201, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_create_middleware)),
        )
        .api_route(
            "/providers/{provider_id}",
            get_with(providers::get_rag_provider, |op| {
                op.description("Get RAG provider by ID")
                    .id("Admin.getRagProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_read_middleware))
            .put_with(providers::update_rag_provider, |op| {
                op.description("Update RAG provider")
                    .id("Admin.updateRagProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(rag_admin_providers_edit_middleware))
            .delete_with(providers::delete_rag_provider, |op| {
                op.description("Delete RAG provider")
                    .id("Admin.deleteRagProvider")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(rag_admin_providers_delete_middleware)),
        )
        .api_route(
            "/providers/{provider_id}/test",
            post_with(providers::test_rag_provider, |op| {
                op.description("Test RAG provider connection")
                    .id("Admin.testRagProvider")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(rag_admin_providers_read_middleware)),
        )
}
