use async_trait::async_trait;
use tokio::sync::Mutex;
use std::sync::Arc;
use url::Url;

use crate::database::models::mcp_server::MCPServer;
use crate::mcp::protocol::{MCPRequest, MCPResponse};
use super::{MCPTransport, MCPConnectionInfo};

pub struct MCPHttpTransport {
    server: MCPServer,
    client: reqwest::Client,
    base_url: String,
    initialized: Arc<Mutex<bool>>,
    request_id_counter: Arc<Mutex<u64>>,
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
            request_id_counter: Arc::new(Mutex::new(0)),
        })
    }

    async fn get_next_request_id(&self) -> String {
        let mut counter = self.request_id_counter.lock().await;
        *counter += 1;
        format!("http-{}", *counter)
    }

    async fn send_mcp_request(&self, request: MCPRequest) -> Result<MCPResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let mcp_response: MCPResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MCP response: {}", e))?;

        Ok(mcp_response)
    }

    async fn initialize_mcp_session(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let response = self.send_mcp_request(init_request).await?;

        if response.error.is_some() {
            return Err(format!("MCP initialization failed: {:?}", response.error).into());
        }

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
        if !*self.initialized.lock().await {
            return false;
        }

        // Send a ping request to check if the server is still responsive
        let ping_request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String(self.get_next_request_id().await)),
            method: "ping".to_string(),
            params: None,
        };

        match self.send_mcp_request(ping_request).await {
            Ok(_) => true,
            Err(_) => {
                // If ping fails, try a simpler health check
                let health_check = self.client
                    .get(&format!("{}/health", self.base_url.trim_end_matches("/mcp")))
                    .send()
                    .await;

                match health_check {
                    Ok(response) => response.status().is_success(),
                    Err(_) => false,
                }
            }
        }
    }
}