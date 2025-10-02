use async_trait::async_trait;
use tokio::sync::Mutex;
use std::sync::Arc;
use url::Url;

use crate::database::models::mcp_server::MCPServer;
use crate::ai::mcp::protocol::{MCPRequest, MCPResponse};
use super::{MCPTransport, MCPConnectionInfo};

pub struct MCPHttpTransport {
    server: MCPServer,
    client: reqwest::Client,
    base_url: String,
    initialized: Arc<Mutex<bool>>,
}

impl MCPHttpTransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = server.url.as_ref()
            .ok_or("URL is required for HTTP transport")?;

        // Validate URL format
        let _parsed_url = Url::parse(url)
            .map_err(|e| format!("Invalid URL '{}': {}", url, e))?;

        // Extract base URL for MCP endpoint
        let base_url = if url.ends_with("/mcp") {
            url.clone()
        } else {
            format!("{}/mcp", url.trim_end_matches('/'))
        };

        Ok(Self {
            server: server.clone(),
            client: reqwest::Client::new(),
            base_url,
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// Internal method to send MCP request without initialization check
    async fn send_request_internal(&self, request: MCPRequest) -> Result<MCPResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!("HTTP error {}: {}", status, error_body).into());
        }

        let mcp_response: MCPResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MCP response: {}", e))?;

        Ok(mcp_response)
    }

    pub async fn send_mcp_request(&self, request: MCPRequest) -> Result<MCPResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Check if initialized
        if !*self.initialized.lock().await {
            return Err("MCP session not initialized".into());
        }

        self.send_request_internal(request).await
    }

    async fn initialize_mcp_session(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Send initialize request
        let init_request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String("init".to_string())),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {
                        "listChanged": true
                    },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "ziee-mcp-client",
                    "version": "1.0.0"
                }
            })),
        };

        // Use internal method to bypass initialization check during initialization
        let response = self.send_request_internal(init_request).await?;

        if response.error.is_some() {
            return Err(format!("MCP initialization failed: {:?}", response.error).into());
        }

        // Step 2: Send initialized notification to complete handshake
        // This is required by MCP protocol - must send this after receiving initialize response
        let initialized_notification = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: None, // Notifications have no id
            method: "notifications/initialized".to_string(),
            params: None,
        };

        // Send notification (no response expected)
        let _ = self.send_request_internal(initialized_notification).await;

        *self.initialized.lock().await = true;
        println!("[{}] MCP HTTP session initialized", self.server.name);
        Ok(())
    }
}

#[async_trait]
impl MCPTransport for MCPHttpTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize MCP session with the HTTP server
        self.initialize_mcp_session().await?;

        // Extract port from URL if available
        let port = if let Ok(parsed_url) = Url::parse(&self.base_url) {
            parsed_url.port().map(|p| p as u16)
        } else {
            None
        };

        Ok(MCPConnectionInfo {
            child: None, // No process for HTTP transport
            pid: None,   // No PID for external HTTP server
            port,        // Port from URL
        })
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For HTTP transport, we can optionally send a shutdown notification
        // if the MCP server supports it, but most external servers won't
        *self.initialized.lock().await = false;
        println!("[{}] MCP HTTP session stopped", self.server.name);
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // For HTTP transport (external servers), check if server is reachable
        // The server is running independently, so we just verify connectivity

        // Try the MCP endpoint first
        let mcp_check = self.client
            .get(&self.base_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        if mcp_check.is_ok() {
            return true; // Server is reachable at MCP endpoint
        }

        // Fallback: try health endpoint
        let health_check = self.client
            .get(&format!("{}/health", self.base_url.trim_end_matches("/mcp")))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match health_check {
            Ok(response) => response.status().is_success(),
            Err(_) => false, // Server is not reachable
        }
    }
}