use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot, broadcast};
use url::Url;

use crate::database::models::mcp_server::MCPServer;
use crate::mcp::protocol::{MCPRequest, MCPResponse, MCPNotification};
use super::{MCPTransport, MCPConnectionInfo};

pub struct MCPSSETransport {
    server: MCPServer,
    client: reqwest::Client,
    base_url: String,
    sse_url: String,
    messages_url: String,
    session_id: String,
    initialized: Arc<Mutex<bool>>,
    response_handlers: Arc<Mutex<HashMap<String, oneshot::Sender<MCPResponse>>>>,
    notification_sender: Arc<broadcast::Sender<MCPNotification>>,
    sse_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl MCPSSETransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = server.url.as_ref()
            .ok_or("URL is required for SSE transport")?;

        // Validate URL format
        let _parsed_url = Url::parse(url)
            .map_err(|e| format!("Invalid URL '{}': {}", url, e))?;

        let base_url = url.trim_end_matches('/');
        let sse_url = format!("{}/sse", base_url);

        // Generate a unique session ID for this SSE connection
        let session_id = uuid::Uuid::new_v4().to_string();
        let messages_url = format!("{}/messages/{}", base_url, session_id);

        let (notification_sender, _) = broadcast::channel(1000);

        Ok(Self {
            server: server.clone(),
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            sse_url,
            messages_url,
            session_id,
            initialized: Arc::new(Mutex::new(false)),
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
            notification_sender: Arc::new(notification_sender),
            sse_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn send_mcp_request(&self, request: MCPRequest) -> Result<MCPResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request_id = request.id.as_ref()
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string();

        let (response_sender, response_receiver) = oneshot::channel();

        // Register response handler
        self.response_handlers.lock().await.insert(request_id.clone(), response_sender);

        // Send request via HTTP POST to messages endpoint
        let response = self.client
            .post(&self.messages_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                // Clean up handler on error
                tokio::spawn({
                    let handlers = Arc::clone(&self.response_handlers);
                    let req_id = request_id.clone();
                    async move {
                        handlers.lock().await.remove(&req_id);
                    }
                });
                format!("SSE request failed: {}", e)
            })?;

        if !response.status().is_success() {
            self.response_handlers.lock().await.remove(&request_id);
            return Err(format!("SSE HTTP error: {}", response.status()).into());
        }

        // Wait for response via SSE stream or direct HTTP response
        match tokio::time::timeout(std::time::Duration::from_secs(30), response_receiver).await {
            Ok(Ok(mcp_response)) => Ok(mcp_response),
            Ok(Err(_)) => {
                self.response_handlers.lock().await.remove(&request_id);
                Err("Response channel closed".into())
            }
            Err(_) => {
                self.response_handlers.lock().await.remove(&request_id);
                Err("Request timeout".into())
            }
        }
    }

    async fn start_sse_listener(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response_handlers = Arc::clone(&self.response_handlers);
        let notification_sender = Arc::clone(&self.notification_sender);
        let server_name = self.server.name.clone();
        let sse_url = self.sse_url.clone();
        let client = self.client.clone();

        let handle = tokio::spawn(async move {
            loop {
                match client.get(&sse_url).send().await {
                    Ok(response) => {
                        if !response.status().is_success() {
                            eprintln!("[{}] SSE connection failed: {}", server_name, response.status());
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            continue;
                        }

                        let stream = response.bytes_stream().eventsource();
                        futures::pin_mut!(stream);

                        while let Some(event_result) = stream.next().await {
                            match event_result {
                                Ok(event) => {
                                    if let Ok(json_value) = serde_json::from_str::<Value>(&event.data) {
                                        if json_value.get("id").is_some() {
                                            // This is a response
                                            if let Ok(response) = serde_json::from_value::<MCPResponse>(json_value) {
                                                if let Some(id) = response.id.as_ref().and_then(|v| v.as_str()) {
                                                    let mut handlers = response_handlers.lock().await;
                                                    if let Some(sender) = handlers.remove(id) {
                                                        let _ = sender.send(response);
                                                    }
                                                }
                                            }
                                        } else if json_value.get("method").is_some() {
                                            // This is a notification
                                            if let Ok(notification) = serde_json::from_value::<MCPNotification>(json_value) {
                                                let _ = notification_sender.send(notification);
                                            }
                                        }
                                    }
                                }
                                Err(e) if e.to_string().contains("stream ended") => {
                                    println!("[{}] SSE stream ended, reconnecting...", server_name);
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("[{}] SSE error: {}", server_name, e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[{}] Failed to connect to SSE: {}", server_name, e);
                    }
                }

                // Wait before reconnecting
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });

        *self.sse_handle.lock().await = Some(handle);
        Ok(())
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

        let response = self.send_mcp_request(init_request).await?;

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

        // Send notification (no response expected for notifications)
        let _ = self.send_mcp_request(initialized_notification).await;

        *self.initialized.lock().await = true;
        println!("[{}] MCP SSE session initialized (session: {})", self.server.name, self.session_id);
        Ok(())
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<MCPNotification> {
        self.notification_sender.subscribe()
    }
}

#[async_trait]
impl MCPTransport for MCPSSETransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        // Start SSE listener first
        self.start_sse_listener().await?;

        // Give SSE connection time to establish
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Initialize MCP session
        self.initialize_mcp_session().await?;

        // Extract port from URL if available
        let port = if let Ok(parsed_url) = Url::parse(&self.base_url) {
            parsed_url.port().map(|p| p as u16)
        } else {
            None
        };

        Ok(MCPConnectionInfo {
            child: None, // No process for SSE transport
            pid: None,   // No PID for external SSE server
            port,        // Port from URL
        })
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Stop SSE listener
        if let Some(handle) = self.sse_handle.lock().await.take() {
            handle.abort();
        }

        *self.initialized.lock().await = false;
        println!("[{}] MCP SSE session stopped", self.server.name);
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // For SSE transport, first check if server is reachable
        // Then verify SSE connection status

        // Quick connectivity check - try to reach the base endpoint
        let connectivity_check = self.client
            .get(&self.base_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        if connectivity_check.is_err() {
            return false; // Server is not reachable at all
        }

        // Server is reachable, now check if we have an active SSE connection
        if !*self.initialized.lock().await {
            return true; // Server is up but we haven't initialized yet - still healthy
        }

        // We're initialized, check if SSE handle is still running
        if let Some(handle) = self.sse_handle.lock().await.as_ref() {
            if handle.is_finished() {
                return false; // SSE connection died
            }
        } else {
            return false; // No SSE handle
        }

        true // Server reachable and SSE connection active
    }
}