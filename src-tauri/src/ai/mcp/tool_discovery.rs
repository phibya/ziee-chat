use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::database::queries::{mcp_tools, mcp_servers};
use crate::ai::mcp::protocol::{MCPRequest, MCPResponse, ListToolsResponse as MCPListToolsResponse};

// Global locks for preventing concurrent tool discovery for same server
lazy_static::lazy_static! {
    static ref DISCOVERY_LOCKS: Arc<Mutex<HashMap<Uuid, Arc<Mutex<()>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// Error types for tool discovery
#[derive(Debug, thiserror::Error)]
pub enum ToolDiscoveryError {
    #[error("Server not found: {0}")]
    ServerNotFound(Uuid),
    #[error("Server not running (no proxy URL available)")]
    ServerNotRunning,
    #[error("MCP communication failed: {0}")]
    MCPCommunication(String),
    #[error("Invalid MCP response: {0}")]
    InvalidResponse(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Check if tools should be rediscovered (cache older than 10 minutes)
pub async fn should_rediscover_tools(server_id: Uuid) -> Result<bool, ToolDiscoveryError> {
    let server = mcp_servers::get_mcp_server_by_id(server_id)
        .await?
        .ok_or(ToolDiscoveryError::ServerNotFound(server_id))?;

    match server.tools_discovered_at {
        None => Ok(true), // Never discovered
        Some(discovered_at) => {
            let now = chrono::Utc::now();
            let cache_duration = chrono::Duration::minutes(10);
            Ok(now.signed_duration_since(discovered_at) > cache_duration)
        }
    }
}

/// Create MCP tools/list request
pub fn create_tools_list_request() -> MCPRequest {
    MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::Value::String(uuid::Uuid::new_v4().to_string())),
        method: "tools/list".to_string(),
        params: Some(serde_json::json!({})),
    }
}

/// Send HTTP request to MCP server
pub async fn send_mcp_request(
    server_url: &str,
    request: MCPRequest,
) -> Result<MCPResponse, ToolDiscoveryError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .post(server_url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(ToolDiscoveryError::MCPCommunication(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let mcp_response: MCPResponse = response.json().await?;
    Ok(mcp_response)
}

/// Parse tools from MCP response
pub fn parse_tools_response(
    response: MCPListToolsResponse,
) -> Result<Vec<(String, Option<String>, serde_json::Value)>, ToolDiscoveryError> {
    let mut tools = Vec::new();

    for tool in response.tools {
        tools.push((tool.name, tool.description, tool.input_schema));
    }

    Ok(tools)
}

/// Core function to discover and cache tools from MCP server via HTTP
pub async fn discover_and_cache_tools_http(server_id: Uuid) -> Result<i32, ToolDiscoveryError> {
    // Get or create discovery lock for this server
    let lock = {
        let mut locks = DISCOVERY_LOCKS.lock().await;
        locks
            .entry(server_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };

    // Acquire lock to prevent concurrent discovery
    let _guard = lock.lock().await;

    // Double-check if we still need to discover (another request might have completed)
    if !should_rediscover_tools(server_id).await? {
        // Tools were discovered by another request, get the count
        let tools = mcp_tools::get_cached_tools_for_server(server_id).await?;
        return Ok(tools.len() as i32);
    }

    // 1. Get server details including proxy URL
    let server = mcp_servers::get_mcp_server_by_id(server_id)
        .await?
        .ok_or(ToolDiscoveryError::ServerNotFound(server_id))?;

    // 2. Check if server has proxy URL (means it's running)
    let proxy_url = server.url.ok_or(ToolDiscoveryError::ServerNotRunning)?;
    if proxy_url.is_empty() {
        return Err(ToolDiscoveryError::ServerNotRunning);
    }

    tracing::info!("Discovering tools for server {} at {}", server_id, proxy_url);

    // 3. Create and send tools/list request
    let request = create_tools_list_request();
    let response = send_mcp_request(&proxy_url, request).await?;

    // 4. Handle response
    if let Some(error) = response.error {
        return Err(ToolDiscoveryError::MCPCommunication(format!(
            "MCP error: {} - {}",
            error.code, error.message
        )));
    }

    let result = response
        .result
        .ok_or_else(|| ToolDiscoveryError::InvalidResponse("No result in MCP response".to_string()))?;

    // 5. Parse tools from response
    let tools_response: MCPListToolsResponse = serde_json::from_value(result)
        .map_err(|e| ToolDiscoveryError::InvalidResponse(format!("Failed to parse tools response: {}", e)))?;

    let discovered_tools = parse_tools_response(tools_response)?;

    // 6. Clear old tools and cache new ones
    mcp_tools::clear_tools_cache_for_server(server_id).await?;
    mcp_tools::cache_discovered_tools(server_id, discovered_tools.clone()).await?;

    // 7. Update server's discovery timestamp and count
    let tools_count = discovered_tools.len() as i32;
    mcp_servers::update_tools_discovered(server_id, tools_count).await?;

    tracing::info!("Successfully discovered {} tools for server {}", tools_count, server_id);

    Ok(tools_count)
}

/// Discover tools directly from an MCP client session (for stdio transport)
pub async fn discover_and_cache_tools_direct<T>(
    server_id: Uuid,
    client_session: &T
) -> Result<i32, ToolDiscoveryError>
where
    T: Send + Sync,
    T: ToolDiscoveryClient,
{
    // Get or create discovery lock for this server
    let lock = {
        let mut locks = DISCOVERY_LOCKS.lock().await;
        locks
            .entry(server_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };

    // Acquire lock to prevent concurrent discovery
    let _guard = lock.lock().await;

    tracing::info!("Discovering tools directly for server {}", server_id);

    // 1. Create and send tools/list request directly to session
    let request = create_tools_list_request();
    let response = client_session.send_request(request).await
        .map_err(|e| ToolDiscoveryError::MCPCommunication(format!("Direct session error: {}", e)))?;

    // 2. Handle response
    if let Some(error) = response.error {
        return Err(ToolDiscoveryError::MCPCommunication(format!(
            "MCP error: {} - {}",
            error.code, error.message
        )));
    }

    let result = response
        .result
        .ok_or_else(|| ToolDiscoveryError::InvalidResponse("No result in MCP response".to_string()))?;

    // 3. Parse tools from response
    let tools_response: MCPListToolsResponse = serde_json::from_value(result)
        .map_err(|e| ToolDiscoveryError::InvalidResponse(format!("Failed to parse tools response: {}", e)))?;

    let discovered_tools = parse_tools_response(tools_response)?;

    // 4. Clear old tools and cache new ones
    mcp_tools::clear_tools_cache_for_server(server_id).await?;
    mcp_tools::cache_discovered_tools(server_id, discovered_tools.clone()).await?;

    // 5. Update server's discovery timestamp and count
    let tools_count = discovered_tools.len() as i32;
    mcp_servers::update_tools_discovered(server_id, tools_count).await?;

    tracing::info!("Successfully discovered {} tools directly for server {}", tools_count, server_id);

    Ok(tools_count)
}

/// Trait for types that can send MCP requests (to allow both HTTP and direct session usage)
pub trait ToolDiscoveryClient {
    type Error: std::fmt::Display;

    fn send_request(&self, request: MCPRequest) -> impl std::future::Future<Output = Result<MCPResponse, Self::Error>> + Send;
}