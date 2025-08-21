use crate::database::models::{RAGDatabase, RAGProvider, RAGProviderListResponse};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

use crate::api::rag_providers::{
    add_database_to_rag_provider,
    clone_rag_provider,
    create_rag_provider,
    delete_rag_database,
    delete_rag_provider,
    disable_rag_database,
    enable_rag_database,
    get_rag_database,
    get_rag_provider,
    // RAG Database endpoints
    list_rag_provider_databases,
    // RAG Provider endpoints
    list_rag_providers,
    start_rag_database,
    stop_rag_database,
    update_rag_database,
    update_rag_provider,
};

pub fn admin_rag_provider_routes() -> ApiRouter {
    ApiRouter::new()
        // RAG Provider routes
        .api_route(
            "/rag-providers",
            get_with(list_rag_providers, |op| {
                op.description("List all RAG providers")
                    .id("Admin.listRAGProviders")
                    .tag("admin")
                    .response::<200, Json<RAGProviderListResponse>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_read_middleware)),
        )
        .api_route(
            "/rag-providers",
            post_with(create_rag_provider, |op| {
                op.description("Create a new RAG provider")
                    .id("Admin.createRAGProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_create_middleware)),
        )
        .api_route(
            "/rag-providers/{provider_id}",
            get_with(get_rag_provider, |op| {
                op.description("Get a specific RAG provider")
                    .id("Admin.getRAGProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_read_middleware)),
        )
        .api_route(
            "/rag-providers/{provider_id}",
            put_with(update_rag_provider, |op| {
                op.description("Update a RAG provider")
                    .id("Admin.updateRAGProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_edit_middleware)),
        )
        .api_route(
            "/rag-providers/{provider_id}",
            delete_with(delete_rag_provider, |op| {
                op.description("Delete a RAG provider")
                    .id("Admin.deleteRAGProvider")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_delete_middleware)),
        )
        .api_route(
            "/rag-providers/{provider_id}/clone",
            post_with(clone_rag_provider, |op| {
                op.description("Clone a RAG provider")
                    .id("Admin.cloneRAGProvider")
                    .tag("admin")
                    .response::<200, Json<RAGProvider>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_providers_create_middleware)),
        )
        // RAG Database routes
        .api_route(
            "/rag-providers/{provider_id}/databases",
            get_with(list_rag_provider_databases, |op| {
                op.description("List databases for a RAG provider")
                    .id("Admin.listRAGProviderDatabases")
                    .tag("admin")
                    .response::<200, Json<Vec<RAGDatabase>>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_read_middleware)),
        )
        .api_route(
            "/rag-providers/{provider_id}/databases",
            post_with(add_database_to_rag_provider, |op| {
                op.description("Add a database to a RAG provider")
                    .id("Admin.addDatabaseToRAGProvider")
                    .tag("admin")
                    .response::<200, Json<RAGDatabase>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_create_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}",
            get_with(get_rag_database, |op| {
                op.description("Get a specific RAG database")
                    .id("Admin.getRAGDatabase")
                    .tag("admin")
                    .response::<200, Json<RAGDatabase>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_read_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}",
            put_with(update_rag_database, |op| {
                op.description("Update a RAG database")
                    .id("Admin.updateRAGDatabase")
                    .tag("admin")
                    .response::<200, Json<RAGDatabase>>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_edit_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}",
            delete_with(delete_rag_database, |op| {
                op.description("Delete a RAG database")
                    .id("Admin.deleteRAGDatabase")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_delete_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}/start",
            post_with(start_rag_database, |op| {
                op.description("Start a RAG database")
                    .id("Admin.startRAGDatabase")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_edit_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}/stop",
            post_with(stop_rag_database, |op| {
                op.description("Stop a RAG database")
                    .id("Admin.stopRAGDatabase")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_edit_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}/enable",
            post_with(enable_rag_database, |op| {
                op.description("Enable a RAG database")
                    .id("Admin.enableRAGDatabase")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_edit_middleware)),
        )
        .api_route(
            "/rag-databases/{database_id}/disable",
            post_with(disable_rag_database, |op| {
                op.description("Disable a RAG database")
                    .id("Admin.disableRAGDatabase")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(crate::api::middleware::rag_repositories_edit_middleware)),
        )
}
