use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::rag_repositories::{
    list_rag_repositories,
    get_rag_repository,
    create_rag_repository,
    update_rag_repository,
    delete_rag_repository,
    test_rag_repository_connection,
    list_rag_repository_databases,
    download_rag_database_from_repository,
};

pub fn admin_rag_repository_routes() -> Router {
    Router::new()
        // RAG Repository routes
        .route("/api/admin/rag-repositories", get(list_rag_repositories).post(create_rag_repository))
        .route("/api/admin/rag-repositories/{repository_id}", get(get_rag_repository).put(update_rag_repository).delete(delete_rag_repository))
        .route("/api/admin/rag-repositories/{repository_id}/test-connection", post(test_rag_repository_connection))
        .route("/api/admin/rag-repositories/{repository_id}/databases", get(list_rag_repository_databases))
        .route("/api/admin/rag-repositories/download-database", post(download_rag_database_from_repository))
}