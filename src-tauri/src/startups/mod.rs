pub mod ai_models;
pub mod api_proxy;
pub mod database;
pub mod file_storage;
pub mod hub;
pub mod ngrok;
pub mod rag;

pub use ai_models::*;
pub use api_proxy::*;
pub use database::*;
pub use file_storage::*;
pub use hub::*;
pub use ngrok::*;
pub use rag::*;

/// Initialize all application components
pub async fn initialize_app_common() -> Result<(), String> {
    // Initialize database
    initialize_database().await?;

    // Initialize file storage
    initialize_file_storage().await?;

    // Initialize hub manager
    initialize_hub().await?;

    // Initialize AI models and services
    initialize_ai_models().await?;

    // Initialize API proxy
    initialize_api_proxy().await?;

    // Initialize ngrok tunnel
    initialize_ngrok().await?;

    // Initialize RAG service
    initialize_rag().await?;

    Ok(())
}

/// Cleanup all application components
pub async fn cleanup_app_common() {
    // Cleanup RAG service first
    cleanup_rag().await;

    // Cleanup AI models
    cleanup_ai_models().await;

    // Cleanup file storage
    cleanup_file_storage().await;

    // Cleanup database last
    cleanup_database().await;
}
