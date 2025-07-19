// MCP server functionality temporarily disabled due to dependency issues

use std::sync::Arc;

// Stub for the MistralRs type - replace with actual import when needed
pub type MistralRs = Arc<dyn std::any::Any + Send + Sync>;

/// Create a stub MCP server that does nothing
pub async fn create_http_mcp_server(
    _mistralrs: MistralRs,
    _host: String,
    _port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Return immediately - MCP functionality is disabled
    Ok(())
}