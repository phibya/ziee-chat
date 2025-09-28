use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, Sse},
    response::sse::{Event, KeepAlive},
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::convert::Infallible;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command, ChildStdout};
use tokio::sync::{Mutex, broadcast};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::database::models::mcp_server::MCPServer;
use super::ProxyError;
use crate::mcp::logging::MCPLogger;

use crate::mcp::protocol::{MCPRequest, MCPResponse, MCPNotification, InitializeRequest, MCPCapabilities, ClientInfo, methods, RootsCapability, PromptsCapability, ResourcesCapability, ToolsCapability, MCP_PROTOCOL_VERSION};

// MCP Client Session - handles communication with stdio MCP server
pub struct MCPClientSession {
    server_id: Uuid,
    server_name: String,
    child_process: Arc<Mutex<Option<Child>>>,
    request_sender: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    response_handlers: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<MCPResponse>>>>,
    notification_sender: Arc<broadcast::Sender<MCPNotification>>,
}

impl MCPClientSession {
    pub async fn new(server: &MCPServer) -> Result<Self, ProxyError> {
        let (notification_sender, _) = broadcast::channel(1000);

        Ok(Self {
            server_id: server.id,
            server_name: server.name.clone(),
            child_process: Arc::new(Mutex::new(None)),
            request_sender: Arc::new(Mutex::new(None)),
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
            notification_sender: Arc::new(notification_sender),
        })
    }

    pub async fn initialize(&self, server: &MCPServer) -> Result<(), ProxyError> {
        let logger = MCPLogger::new(server.id);

        logger.log_exec("INFO", &format!("Initializing MCP client session for server: {}", server.name));

        // Start the stdio MCP server process with bundled runtime support
        let command = server.command.as_ref()
            .ok_or_else(|| ProxyError::ClientCommunication("Command is required for stdio transport".to_string()))?;

        let args: Vec<String> = serde_json::from_value(server.args.clone()).unwrap_or_default();
        let env_vars = self.get_server_env(server).await?;

        logger.log_exec("INFO", &format!("Starting MCP server process: {} {:?}", command, args));

        // Use the same command resolution from stdio.rs for bundled runtime support
        let (resolved_command, resolved_args) = crate::mcp::transports::stdio::resolve_command(command, &args);

        let mut cmd = Command::new(resolved_command);
        cmd.args(resolved_args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("IS_ZIEE_MCP", "1");

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().map_err(|e| {
            logger.log_exec("ERROR", &format!("Failed to spawn MCP server process: {}", e));
            ProxyError::ProcessSpawn(e)
        })?;

        let pid = child.id();
        logger.log_exec("INFO", &format!("MCP server process started successfully (PID: {:?})", pid));

        let stdin = child.stdin.take().ok_or_else(|| {
            logger.log_exec("ERROR", "Failed to get stdin handle from child process");
            ProxyError::ClientCommunication("Failed to get stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            logger.log_exec("ERROR", "Failed to get stdout handle from child process");
            ProxyError::ClientCommunication("Failed to get stdout".to_string())
        })?;

        // Capture stderr for logging
        if let Some(stderr) = child.stderr.take() {
            let logger_clone = logger.clone();
            let server_name = server.name.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if !trimmed.is_empty() {
                                logger_clone.log_stderr(trimmed);
                            }
                        }
                        Err(e) => {
                            logger_clone.log_exec("ERROR", &format!("Error reading stderr: {}", e));
                            break;
                        }
                    }
                }
                println!("[{}] MCP server stderr reader closed", server_name);
            });
        }

        *self.request_sender.lock().await = Some(stdin);
        *self.child_process.lock().await = Some(child);

        // Start stdout reader task
        self.start_stdout_reader(stdout).await;

        // Send MCP initialize request
        let init_request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String("init".to_string())),
            method: methods::INITIALIZE.to_string(),
            params: Some(serde_json::to_value(InitializeRequest {
                protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                capabilities: MCPCapabilities {
                    experimental: Some(std::collections::HashMap::new()),
                    logging: Some(serde_json::json!({})),
                    prompts: Some(PromptsCapability {
                        list_changed: Some(false),
                    }),
                    resources: Some(ResourcesCapability {
                        list_changed: Some(false),
                        subscribe: Some(false),
                    }),
                    roots: Some(RootsCapability {
                        list_changed: Some(true),
                    }),
                    sampling: Some(serde_json::json!({})),
                    tools: Some(ToolsCapability {
                        list_changed: Some(false),
                    }),
                },
                client_info: ClientInfo {
                    name: "ziee-mcp-proxy".to_string(),
                    version: "1.0.0".to_string(),
                },
            }).unwrap()),
        };

        let _init_response = self.send_request(init_request).await?;
        println!("[{}] MCP session initialized", self.server_name);

        Ok(())
    }

    pub async fn send_request(&self, request: MCPRequest) -> Result<MCPResponse, ProxyError> {
        let logger = MCPLogger::new(self.server_id);

        let request_id = request.id.as_ref()
            .map(|id| match id {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => id.to_string(),
            })
            .unwrap_or_else(|| "unknown".to_string());

        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();

        // Register response handler
        self.response_handlers.lock().await.insert(request_id.clone(), response_sender);

        // Send request
        if let Some(stdin) = self.request_sender.lock().await.as_mut() {
            let request_str = serde_json::to_string(&request)
                .map_err(|e| ProxyError::JsonError(e))?;
            let request_line = format!("{}\n", request_str);

            // Log what we're sending to stdin
            logger.log_stdin(&request_str);

            match timeout(Duration::from_secs(5), stdin.write_all(request_line.as_bytes())).await {
                Ok(Ok(_)) => {
                    // Wait for response
                    match timeout(Duration::from_secs(30), response_receiver).await {
                        Ok(Ok(response)) => Ok(response),
                        Ok(Err(_)) => Err(ProxyError::ClientCommunication("Response channel closed".to_string())),
                        Err(_) => {
                            // Cleanup handler on timeout
                            self.response_handlers.lock().await.remove(&request_id);
                            Err(ProxyError::Timeout)
                        }
                    }
                }
                Ok(Err(e)) => Err(ProxyError::ProcessSpawn(e)),
                Err(_) => {
                    self.response_handlers.lock().await.remove(&request_id);
                    Err(ProxyError::Timeout)
                }
            }
        } else {
            Err(ProxyError::ClientCommunication("No stdin available".to_string()))
        }
    }

    pub async fn send_notification(&self, notification: MCPNotification) -> Result<(), ProxyError> {
        if let Some(stdin) = self.request_sender.lock().await.as_mut() {
            let notification_str = serde_json::to_string(&notification)
                .map_err(|e| ProxyError::JsonError(e))?;
            let notification_line = format!("{}\n", notification_str);

            match timeout(Duration::from_secs(5), stdin.write_all(notification_line.as_bytes())).await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(e)) => Err(ProxyError::ProcessSpawn(e)),
                Err(_) => Err(ProxyError::Timeout),
            }
        } else {
            Err(ProxyError::ClientCommunication("No stdin available".to_string()))
        }
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<MCPNotification> {
        self.notification_sender.subscribe()
    }

    async fn start_stdout_reader(&self, stdout: ChildStdout) {
        let logger = MCPLogger::new(self.server_id);

        let response_handlers = Arc::clone(&self.response_handlers);
        let notification_sender = Arc::clone(&self.notification_sender);
        let server_name = self.server_name.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        logger.log_exec("INFO", "MCP server stdout closed");
                        println!("[{}] MCP server stdout closed", server_name);
                        break;
                    }
                    Ok(_) => {
                        let line = line.trim();
                        if !line.is_empty() {
                            // Log what we received from stdout
                            logger.log_stdout(line);

                            if let Ok(json_value) = serde_json::from_str::<Value>(line) {
                                if json_value.get("id").is_some() {
                                    // This is a response
                                    if let Ok(response) = serde_json::from_value::<MCPResponse>(json_value) {
                                        if let Some(id_value) = response.id.as_ref() {
                                            let id_string = match id_value {
                                                Value::String(s) => s.clone(),
                                                Value::Number(n) => n.to_string(),
                                                _ => id_value.to_string(),
                                            };
                                            let mut handlers = response_handlers.lock().await;
                                            if let Some(sender) = handlers.remove(&id_string) {
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
                    }
                    Err(e) => {
                        logger.log_exec("ERROR", &format!("Error reading from stdout: {}", e));
                        eprintln!("[{}] Error reading from stdout: {}", server_name, e);
                        break;
                    }
                }
            }
        });
    }

    async fn get_server_env(&self, server: &MCPServer) -> Result<HashMap<String, String>, ProxyError> {
        let mut env_vars = HashMap::new();

        if let Ok(env_map) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(server.environment_variables.clone()) {
            for (key, value) in env_map {
                if let Some(val_str) = value.as_str() {
                    env_vars.insert(key, val_str.to_string());
                }
            }
        }

        Ok(env_vars)
    }
}


// MCP Proxy Server - implements MCP server that proxies to client session
pub struct MCPProxyServer {
    client_session: Arc<MCPClientSession>,
    server_name: String,
}

impl MCPProxyServer {
    pub fn new(client_session: Arc<MCPClientSession>, server_name: String) -> Self {
        Self {
            client_session,
            server_name,
        }
    }

    pub async fn handle_request(&self, request: MCPRequest) -> Result<MCPResponse, ProxyError> {
        // Proxy the request to the underlying MCP server via client session
        println!("[{}] Proxying request: {}", self.server_name, request.method);
        self.client_session.send_request(request).await
    }

    pub async fn handle_notification(&self, notification: MCPNotification) -> Result<(), ProxyError> {
        // Proxy the notification to the underlying MCP server via client session
        println!("[{}] Proxying notification: {}", self.server_name, notification.method);
        self.client_session.send_notification(notification).await
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<MCPNotification> {
        self.client_session.subscribe_notifications()
    }
}

// Main stdio proxy that creates the HTTP/SSE server
pub struct MCPStdioProxy {
    pub server_id: Uuid,
    pub server_name: String,
    pub proxy_port: u16,
    pub proxy_url: String,
    server: MCPServer,
    client_session: Option<Arc<MCPClientSession>>,
    proxy_server: Option<Arc<MCPProxyServer>>,
}

impl MCPStdioProxy {
    pub async fn new(server: &MCPServer) -> Result<Self, ProxyError> {
        Ok(Self {
            server_id: server.id,
            server_name: server.name.clone(),
            proxy_port: 0, // Will be set when starting
            proxy_url: String::new(), // Will be set when starting
            server: server.clone(),
            client_session: None,
            proxy_server: None,
        })
    }

    pub async fn start(&mut self, port: u16) -> Result<(), ProxyError> {
        let logger = MCPLogger::new(self.server_id);

        logger.log_exec("INFO", &format!("Starting MCP stdio proxy for server: {} on port {}", self.server_name, port));

        // 1. Create and initialize MCP client session
        let client_session = Arc::new(MCPClientSession::new(&self.server).await?);
        client_session.initialize(&self.server).await?;

        // 2. Create MCP proxy server
        let proxy_server = Arc::new(MCPProxyServer::new(
            Arc::clone(&client_session),
            self.server_name.clone(),
        ));

        self.client_session = Some(client_session);
        self.proxy_server = Some(proxy_server);

        // 3. Start HTTP server with MCP-compliant routes
        self.proxy_port = port;
        self.proxy_url = format!("http://127.0.0.1:{}", port);

        let app = self.create_http_server();
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .map_err(|e| ProxyError::HttpServerStart(e.to_string()))?;

        // Spawn the HTTP server in the background
        let server_future = axum::serve(listener, app);
        tokio::spawn(async move {
            if let Err(e) = server_future.await {
                eprintln!("HTTP server error: {}", e);
            }
        });

        logger.log_exec("INFO", &format!("MCP stdio proxy started successfully on port {}", port));
        println!("Started MCP proxy for '{}' on port {}", self.server_name, port);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ProxyError> {
        let logger = MCPLogger::new(self.server_id);

        logger.log_exec("INFO", "Stopping MCP stdio proxy");

        // Stop HTTP server (will be dropped when the proxy is dropped)

        // Clean up client session and proxy server
        self.client_session = None;
        self.proxy_server = None;

        logger.log_exec("INFO", "MCP stdio proxy stopped successfully");
        println!("Stopped MCP proxy for '{}'", self.server_name);
        Ok(())
    }

    pub async fn is_healthy(&self) -> bool {
        // Check if client session is available and healthy
        self.client_session.is_some()
    }

    fn create_http_server(&self) -> Router {
        let proxy_server = self.proxy_server.as_ref().unwrap().clone();

        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/sse", get(handle_sse_connection))
            .route("/messages/{session_id}", post(handle_sse_message))
            .route("/health", get(handle_health_check))
            .with_state(proxy_server)
    }

    // This method is no longer needed as the client session handles stdout reading
    // async fn start_stdout_reader(&self) -> tokio::task::JoinHandle<()> { ... }

    // These methods are no longer needed as the client session handles server setup
    // async fn get_server_command(&self) -> Result<String, ProxyError> { ... }
    // async fn get_server_args(&self) -> Result<Vec<String>, ProxyError> { ... }
    // async fn get_server_env(&self) -> Result<HashMap<String, String>, ProxyError> { ... }
}

// HTTP handler functions for MCP-compliant transports

// Helper function to handle MCP request responses consistently
async fn handle_mcp_request_common(
    proxy_server: &Arc<MCPProxyServer>,
    request: MCPRequest,
) -> Result<Json<Value>, StatusCode> {
    match proxy_server.handle_request(request).await {
        Ok(response) => {
            let response_json = serde_json::to_value(response)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(response_json))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn handle_mcp_request(
    State(proxy_server): State<Arc<MCPProxyServer>>,
    Json(request_json): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Parse the JSON as an MCP request
    let request: MCPRequest = serde_json::from_value(request_json)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Handle the request through the proxy server
    handle_mcp_request_common(&proxy_server, request).await
}

async fn handle_health_check() -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}

// SSE connection handler - streams notifications from the MCP server
async fn handle_sse_connection(
    State(proxy_server): State<Arc<MCPProxyServer>>,
) -> impl axum::response::IntoResponse {
    let notification_receiver = proxy_server.subscribe_notifications();

    let stream = futures::stream::unfold(notification_receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(notification) => {
                    if let Ok(data) = serde_json::to_string(&notification) {
                        return Some((Ok::<_, Infallible>(Event::default().data(data)), receiver));
                    }
                    continue;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Skip lagged messages
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Channel closed, exit the loop
                    return None;
                }
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// SSE message handler for sending messages to the server
async fn handle_sse_message(
    State(proxy_server): State<Arc<MCPProxyServer>>,
    axum::extract::Path(_session_id): axum::extract::Path<String>,
    Json(request_json): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // For SSE, we accept both requests and notifications

    // Try to parse as request first
    if let Ok(request) = serde_json::from_value::<MCPRequest>(request_json.clone()) {
        // Handle as request
        handle_mcp_request_common(&proxy_server, request).await
    } else if let Ok(notification) = serde_json::from_value::<MCPNotification>(request_json) {
        // Handle as notification
        match proxy_server.handle_notification(notification).await {
            Ok(_) => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0"
                });
                Ok(Json(response))
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}