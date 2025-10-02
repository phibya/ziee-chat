//! Global state for internal RAG MCP server
//!
//! Stores the randomly assigned port for the internal MCP server
//! so it can be accessed throughout the application.

use std::sync::OnceLock;

/// Global storage for the RAG MCP server port
static RAG_MCP_PORT: OnceLock<u16> = OnceLock::new();

/// Set the RAG MCP server port (can only be called once)
pub fn set_rag_mcp_port(port: u16) {
    if RAG_MCP_PORT.set(port).is_err() {
        tracing::warn!("RAG MCP port already set, ignoring duplicate call");
    } else {
        tracing::info!("RAG MCP server port set to: {}", port);
    }
}

/// Get the RAG MCP server port if it has been set
pub fn get_rag_mcp_port() -> Option<u16> {
    RAG_MCP_PORT.get().copied()
}

/// Get the full RAG MCP server URL if port has been set
pub fn get_rag_mcp_url() -> Option<String> {
    get_rag_mcp_port().map(|port| format!("http://127.0.0.1:{}/mcp", port))
}
