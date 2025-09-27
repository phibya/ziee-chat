pub mod ai_models;
pub mod api_proxy;
pub mod database;
pub mod file_storage;
pub mod hub;
pub mod mcp;
pub mod ngrok;
pub mod rag;

pub use ai_models::*;
pub use api_proxy::*;
pub use database::*;
pub use file_storage::*;
pub use hub::*;
pub use mcp::*;
pub use ngrok::*;
pub use rag::*;

/// Initialize all application components
pub async fn initialize_app_common() -> Result<(), String> {
    // Initialize database first - this must complete before other services
    initialize_database().await?;

    // Start all other services concurrently without waiting for completion
    // They will initialize in the background while the app continues
    tokio::spawn(async {
        if let Err(e) = initialize_file_storage().await {
            eprintln!("File storage initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_hub().await {
            eprintln!("Hub initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_ai_models().await {
            eprintln!("AI models initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_api_proxy().await {
            eprintln!("API proxy initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_ngrok().await {
            eprintln!("Ngrok initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_rag().await {
            eprintln!("RAG service initialization failed: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = initialize_mcp().await {
            eprintln!("MCP servers initialization failed: {}", e);
        }
    });

    Ok(())
}

/// Cleanup all application components
pub async fn cleanup_app_common() {
    // Cleanup MCP servers first (stop running servers)
    cleanup_mcp().await;

    // Cleanup RAG service
    cleanup_rag().await;

    // Cleanup AI models
    cleanup_ai_models().await;

    // Cleanup file storage
    cleanup_file_storage().await;

    // Cleanup database last
    cleanup_database().await;
}
