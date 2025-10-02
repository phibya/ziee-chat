use crate::ai::rag::mcp_server::start_rag_mcp_server;
use crate::ai::rag::service::RAGService;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

// Global RAG service instance
pub static RAG_SERVICE: Lazy<Arc<RwLock<Option<RAGService>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Initialize RAG service
pub async fn initialize_rag() -> Result<(), String> {
    let service = RAGService::new();

    // Start the RAG service - engine type determined from pending files
    if let Err(e) = service.start().await {
        eprintln!("Failed to start RAG service: {}", e);
        return Err(format!("Failed to start RAG service: {}", e));
    }

    // Store the service globally
    let mut global_service = RAG_SERVICE.write().await;
    *global_service = Some(service);

    println!("RAG service initialized successfully");

    // Start the internal RAG MCP server
    if let Err(e) = start_rag_mcp_server().await {
        eprintln!("Failed to start RAG MCP server: {}", e);
        return Err(format!("Failed to start RAG MCP server: {}", e));
    }

    println!("RAG MCP server started successfully");
    Ok(())
}

/// Cleanup RAG service
pub async fn cleanup_rag() {
    let mut global_service = RAG_SERVICE.write().await;
    if let Some(service) = global_service.take() {
        if let Err(e) = service.stop().await {
            eprintln!("Failed to stop RAG service: {}", e);
        } else {
            println!("RAG service stopped successfully");
        }
    }
}
