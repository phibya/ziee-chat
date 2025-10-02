//! Startup logic for internal RAG MCP server

use super::global::set_rag_mcp_port;
use super::server::UnifiedRagMcpServer;
use tokio::net::TcpListener;

/// Start the internal RAG MCP server on a random available port
///
/// The server binds to 127.0.0.1:0 (random port) and stores the assigned
/// port globally for internal access.
pub async fn start_rag_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting internal RAG MCP server...");

    // Bind to localhost with random port (0 = let OS choose)
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    let port = local_addr.port();

    // Store port globally
    set_rag_mcp_port(port);

    tracing::info!(
        "RAG MCP server listening on http://127.0.0.1:{}/mcp",
        port
    );

    // Create router
    let app = UnifiedRagMcpServer::router();

    // Spawn server task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("RAG MCP server error: {}", e);
        }
    });

    Ok(())
}
