use crate::database::models::{
    RAGRepository, RAGRepositoryConnectionTestResponse, RAGRepositoryListResponse,
};
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

use crate::api::rag::repositories::{
    create_rag_repository, delete_rag_repository, get_rag_repository, list_rag_repositories,
    test_rag_repository_connection, update_rag_repository,
};

pub fn admin_rag_repository_routes() -> ApiRouter {
    ApiRouter::new()
        // RAG Repository routes
        .api_route(
            "/repositories",
            get_with(list_rag_repositories, |op| {
                op.description("List all RAG repositories")
                    .id("Admin.listRAGRepositories")
                    .tag("admin")
                    .response::<200, Json<RAGRepositoryListResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_read_middleware,
            )),
        )
        .api_route(
            "/repositories",
            post_with(create_rag_repository, |op| {
                op.description("Create a new RAG repository")
                    .id("Admin.createRAGRepository")
                    .tag("admin")
                    .response::<200, Json<RAGRepository>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_create_middleware,
            )),
        )
        .api_route(
            "/repositories/{repository_id}",
            get_with(get_rag_repository, |op| {
                op.description("Get a specific RAG repository")
                    .id("Admin.getRAGRepository")
                    .tag("admin")
                    .response::<200, Json<RAGRepository>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_read_middleware,
            )),
        )
        .api_route(
            "/repositories/{repository_id}",
            put_with(update_rag_repository, |op| {
                op.description("Update a RAG repository")
                    .id("Admin.updateRAGRepository")
                    .tag("admin")
                    .response::<200, Json<RAGRepository>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_edit_middleware,
            )),
        )
        .api_route(
            "/repositories/{repository_id}",
            delete_with(delete_rag_repository, |op| {
                op.description("Delete a RAG repository")
                    .id("Admin.deleteRAGRepository")
                    .tag("admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_delete_middleware,
            )),
        )
        .api_route(
            "/repositories/{repository_id}/test-connection",
            post_with(test_rag_repository_connection, |op| {
                op.description("Test RAG repository connection")
                    .id("Admin.testRAGRepositoryConnection")
                    .tag("admin")
                    .response::<200, Json<RAGRepositoryConnectionTestResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::rag_repositories_read_middleware,
            )),
        )
}
