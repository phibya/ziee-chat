use axum::{
    routing::{delete, get, post, put},
    Router,
};

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

pub fn admin_rag_provider_routes() -> Router {
    Router::new()
        // RAG Provider routes
        .route(
            "/rag-providers",
            get(list_rag_providers).post(create_rag_provider),
        )
        .route(
            "/rag-providers/{provider_id}",
            get(get_rag_provider)
                .put(update_rag_provider)
                .delete(delete_rag_provider),
        )
        .route(
            "/rag-providers/{provider_id}/clone",
            post(clone_rag_provider),
        )
        // RAG Database routes
        .route(
            "/rag-providers/{provider_id}/databases",
            get(list_rag_provider_databases).post(add_database_to_rag_provider),
        )
        .route(
            "/rag-databases/{database_id}",
            get(get_rag_database)
                .put(update_rag_database)
                .delete(delete_rag_database),
        )
        .route(
            "/rag-databases/{database_id}/start",
            post(start_rag_database),
        )
        .route("/rag-databases/{database_id}/stop", post(stop_rag_database))
        .route(
            "/rag-databases/{database_id}/enable",
            post(enable_rag_database),
        )
        .route(
            "/rag-databases/{database_id}/disable",
            post(disable_rag_database),
        )
}
