use crate::api::rag::{files, instances};
use crate::database::models::{AddFilesToRAGInstanceResponse, RAGInstance, RAGInstanceFile, RAGInstanceListResponse, RAGProvider};
use aide::axum::{
    routing::{delete_with, get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn rag_routes() -> ApiRouter {
    ApiRouter::new()
        // User RAG routes  
        .nest("/rag", user_rag_routes())
}

/// User routes for RAG instances and enabled providers
fn user_rag_routes() -> ApiRouter {
    ApiRouter::new()
        // RAG providers available for creating instances
        .api_route(
            "/providers",
            get_with(instances::list_creatable_rag_providers_handler, |op| {
                op.description("List RAG providers available for creating instances")
                    .id("Rag.listCreatableProviders")
                    .tag("rag")
                    .response::<200, Json<Vec<RAGProvider>>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_read_middleware)),
        )
        // RAG instance management - list all user instances
        .api_route(
            "/instances",
            get_with(instances::list_user_rag_instances_handler, |op| {
                op.description("List user's RAG instances")
                    .id("Rag.listInstances")
                    .tag("rag")
                    .response::<200, Json<RAGInstanceListResponse>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_read_middleware)),
        )
        // RAG instance management - create instance for specific provider
        .api_route(
            "/providers/{provider_id}/instances",
            post_with(instances::create_user_rag_instance_handler, |op| {
                op.description("Create new RAG instance for provider")
                    .id("Rag.createInstance")
                    .tag("rag")
                    .response::<201, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_create_middleware)),
        )
        .api_route(
            "/instances/{instance_id}",
            get_with(instances::get_rag_instance_handler, |op| {
                op.description("Get RAG instance by ID")
                    .id("Rag.getInstance")
                    .tag("rag")
                    .response::<200, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_read_middleware))
            .put_with(instances::update_rag_instance_handler, |op| {
                op.description("Update RAG instance")
                    .id("Rag.updateInstance")
                    .tag("rag")
                    .response::<200, Json<RAGInstance>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_edit_middleware))
            .delete_with(instances::delete_rag_instance_handler, |op| {
                op.description("Delete RAG instance")
                    .id("Rag.deleteInstance")
                    .tag("rag")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_instances_delete_middleware)),
        )
        // .api_route(
        //     "/instances/{instance_id}/status",
        //     get_with(instances::get_instance_processing_status, |op| {
        //         op.description("Get RAG instance processing status")
        //             .id("Rag.getInstanceStatus")
        //             .tag("rag")
        //     }),
        // )
        // File management in RAG instances
        .api_route(
            "/instances/{instance_id}/files",
            get_with(files::list_rag_instance_files_handler, |op| {
                op.description("List files in RAG instance")
                    .id("Rag.listInstanceFiles")
                    .tag("rag")
                    .response::<200, Json<Vec<RAGInstanceFile>>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_files_read_middleware))
            .post_with(files::add_files_to_rag_instance_handler, |op| {
                op.description("Add files to RAG instance")
                    .id("Rag.addFilesToInstance")
                    .tag("rag")
                    .response::<200, Json<AddFilesToRAGInstanceResponse>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_files_add_middleware)),
        )
        .api_route(
            "/instances/{instance_id}/files/{file_id}",
            delete_with(files::remove_file_from_rag_instance_handler, |op| {
                op.description("Remove file from RAG instance")
                    .id("Rag.removeFileFromInstance")
                    .tag("rag")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::permissions::rag_files_remove_middleware)),
        )
        // TODO: Query endpoint - Enable once OperationHandler trait issue is resolved
        // .api_route(
        //     "/instances/{instance_id}/query",
        //     post_with(instances::query_rag_instance, |op| {
        //         op.description("Query RAG instance")
        //             .id("Rag.queryInstance")
        //             .tag("rag")
        //     }),
        // )
}