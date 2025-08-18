use aide::{
    axum::{ApiRouter, routing::{delete_with, get_with, post_with, put_with}},
};

use crate::api::rag_repositories::{
    create_rag_repository, delete_rag_repository, download_rag_database_from_repository,
    get_rag_repository, list_rag_repositories, list_rag_repository_databases,
    test_rag_repository_connection, update_rag_repository,
};

pub fn admin_rag_repository_routes() -> ApiRouter {
    ApiRouter::new()
        // RAG Repository routes
        .api_route(
            "/rag-repositories",
            get_with(list_rag_repositories, |op| {
                op.description("List all RAG repositories")
                    .id("Admin.listRAGRepositories")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories",
            post_with(create_rag_repository, |op| {
                op.description("Create a new RAG repository")
                    .id("Admin.createRAGRepository")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/{repository_id}",
            get_with(get_rag_repository, |op| {
                op.description("Get a specific RAG repository")
                    .id("Admin.getRAGRepository")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/{repository_id}",
            put_with(update_rag_repository, |op| {
                op.description("Update a RAG repository")
                    .id("Admin.updateRAGRepository")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/{repository_id}",
            delete_with(delete_rag_repository, |op| {
                op.description("Delete a RAG repository")
                    .id("Admin.deleteRAGRepository")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/{repository_id}/test-connection",
            post_with(test_rag_repository_connection, |op| {
                op.description("Test RAG repository connection")
                    .id("Admin.testRAGRepositoryConnection")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/{repository_id}/databases",
            get_with(list_rag_repository_databases, |op| {
                op.description("List databases for a RAG repository")
                    .id("Admin.listRAGRepositoryDatabases")
                    .tag("admin")
            })
        )
        .api_route(
            "/rag-repositories/download-database",
            post_with(download_rag_database_from_repository, |op| {
                op.description("Download RAG database from repository")
                    .id("Admin.downloadRAGDatabaseFromRepository")
                    .tag("admin")
            })
        )
}
