//! MCP Tool Executor - Bridge between chat tool calling and MCP server execution
//!
//! This module provides a unified interface for executing tools across different
//! MCP transport types (HTTP, SSE, Stdio via proxy).

use serde_json::Value;
use uuid::Uuid;

use crate::database::models::mcp_server::MCPTransportType;
use crate::database::queries::mcp_servers;
use crate::mcp::protocol::{MCPRequest, MCPResponse, methods};
use crate::mcp::transports::http::MCPHttpTransport;
use crate::mcp::transports::sse::MCPSSETransport;
use crate::mcp::transports::MCPTransport;

// ============================================
// Result Types
// ============================================

/// Result from MCP tool execution
#[derive(Debug)]
pub struct MCPToolExecutionResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub duration_ms: i64,
}

// ============================================
// Error Types
// ============================================

#[derive(Debug)]
pub enum MCPToolExecutionError {
    ServerNotFound,
    InvalidTransportType(String),
    ServerNotRunning,
    ConnectionFailed(String),
    RequestFailed(String),
    InvalidResponse(String),
    ToolNotFound(String),
    ToolExecutionFailed { code: i32, message: String },
}

impl std::fmt::Display for MCPToolExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerNotFound => write!(f, "MCP server not found"),
            Self::InvalidTransportType(t) => {
                write!(f, "Invalid transport type: {}", t)
            }
            Self::ServerNotRunning => write!(f, "MCP server is not running"),
            Self::ConnectionFailed(e) => write!(f, "Failed to connect to MCP server: {}", e),
            Self::RequestFailed(e) => write!(f, "MCP request failed: {}", e),
            Self::InvalidResponse(e) => write!(f, "Invalid MCP response: {}", e),
            Self::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            Self::ToolExecutionFailed { code, message } => {
                write!(f, "Tool execution failed (code {}): {}", code, message)
            }
        }
    }
}

impl std::error::Error for MCPToolExecutionError {}

// ============================================
// Transport Wrapper - Unified Interface
// ============================================

/// Wrapper enum to provide unified interface across all transport types
enum MCPTransportWrapper {
    Http(MCPHttpTransport),
    Sse(MCPSSETransport),
}

impl MCPTransportWrapper {
    /// Send MCP request through the appropriate transport
    pub async fn send_mcp_request(
        &self,
        request: MCPRequest,
    ) -> Result<MCPResponse, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::Http(transport) => transport.send_mcp_request(request).await,
            Self::Sse(transport) => transport.send_mcp_request(request).await,
        }
    }
}

// ============================================
// Main Execution Functions
// ============================================

/// Execute a tool via MCP transport
///
/// This is the main entry point for chat tool calling. It:
/// 1. Gets the server from database
/// 2. Creates/gets appropriate transport
/// 3. Sends tools/call request
/// 4. Returns structured result
pub async fn execute_mcp_tool(
    server_id: Uuid,
    tool_name: String,
    arguments: Value,
) -> Result<MCPToolExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Executing MCP tool '{}' on server {}",
        tool_name,
        server_id
    );

    let start_time = std::time::Instant::now();

    // Get transport for this server
    let transport = get_or_create_transport(server_id).await?;

    // Create tool call request
    let request_id = format!("tool-{}", Uuid::new_v4());
    let request = create_tool_call_request(&tool_name, &arguments, request_id);

    // Send request via transport
    let response = transport.send_mcp_request(request).await?;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    // Parse response
    if let Some(error) = response.error {
        tracing::error!(
            "MCP tool '{}' execution failed: {} (code: {})",
            tool_name,
            error.message,
            error.code
        );

        return Ok(MCPToolExecutionResult {
            success: false,
            result: None,
            error_message: Some(error.message),
            error_code: Some(error.code.to_string()),
            duration_ms,
        });
    }

    tracing::info!(
        "MCP tool '{}' executed successfully in {}ms",
        tool_name,
        duration_ms
    );

    Ok(MCPToolExecutionResult {
        success: true,
        result: response.result,
        error_message: None,
        error_code: None,
        duration_ms,
    })
}

/// Get or create transport for the given server
async fn get_or_create_transport(
    server_id: Uuid,
) -> Result<MCPTransportWrapper, Box<dyn std::error::Error + Send + Sync>> {
    // Get server from database
    let server = mcp_servers::get_mcp_server_by_id(server_id)
        .await?
        .ok_or_else(|| MCPToolExecutionError::ServerNotFound)?;

    // Handle different transport types
    match server.transport_type {
        MCPTransportType::Http => {
            tracing::debug!("Creating HTTP transport for server {}", server_id);
            let transport = MCPHttpTransport::new(&server)?;

            // Always initialize the session for newly created transport
            // Even if server is running, we need to establish our MCP session
            tracing::debug!("Initializing HTTP transport session...");
            transport.start().await?;

            Ok(MCPTransportWrapper::Http(transport))
        }
        MCPTransportType::Sse => {
            tracing::debug!("Creating SSE transport for server {}", server_id);
            let transport = MCPSSETransport::new(&server)?;

            // Always initialize the session for newly created transport
            // Even if server is running, we need to establish our MCP session
            tracing::debug!("Initializing SSE transport session...");
            transport.start().await?;

            Ok(MCPTransportWrapper::Sse(transport))
        }
        MCPTransportType::Stdio => {
            tracing::debug!(
                "Using stdio proxy URL for server {} (proxy-based transport)",
                server_id
            );

            // For stdio, the proxy URL is stored in the `url` field
            // The stdio transport creates an HTTP proxy when started and updates the URL
            if let Some(proxy_url) = &server.url {
                // Check if it looks like a proxy URL (localhost)
                if proxy_url.contains("127.0.0.1") || proxy_url.contains("localhost") {
                    // Create a temporary server config pointing to the proxy
                    let mut proxy_server = server.clone();
                    proxy_server.transport_type = MCPTransportType::Http;
                    // url is already set to the proxy URL

                    let transport = MCPHttpTransport::new(&proxy_server)?;

                    // Proxy should already be running if stdio transport was started
                    // Verify server is reachable
                    if !transport.is_healthy().await {
                        return Err(Box::new(MCPToolExecutionError::ServerNotRunning));
                    }

                    // Initialize session with the proxy
                    tracing::debug!("Initializing stdio proxy HTTP transport session...");
                    transport.start().await?;

                    Ok(MCPTransportWrapper::Http(transport))
                } else {
                    // URL doesn't look like a proxy, server probably not started
                    Err(Box::new(MCPToolExecutionError::ServerNotRunning))
                }
            } else {
                // No URL set, server definitely not started
                Err(Box::new(MCPToolExecutionError::ServerNotRunning))
            }
        }
    }
}

/// Create MCP tools/call request
fn create_tool_call_request(tool_name: &str, arguments: &Value, request_id: String) -> MCPRequest {
    MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::Value::String(request_id)),
        method: methods::CALL_TOOL.to_string(), // "tools/call"
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": arguments,
        })),
    }
}
