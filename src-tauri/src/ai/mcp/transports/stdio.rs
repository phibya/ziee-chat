use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::{Json, Sse, IntoResponse},
    response::sse::{Event, KeepAlive},
    routing::{get, post},
    Router,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::convert::Infallible;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::{Mutex, broadcast};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use futures;

use crate::database::models::mcp_server::MCPServer;
use crate::database::queries::mcp_servers;
use crate::ai::mcp::logging::MCPLogger;
use crate::ai::mcp::protocol::{MCPRequest, MCPResponse, MCPNotification, InitializeRequest, InitializeResponse, MCPCapabilities, ClientInfo, methods, RootsCapability, PromptsCapability, ResourcesCapability, ToolsCapability, SessionCapability, StreamingCapability, MCPProtocolVersion, detect_protocol_version_from_request, parse_protocol_version};
use crate::ai::mcp::tool_discovery::{ToolDiscoveryClient, discover_and_cache_tools_direct};
use crate::utils::resource_paths::ResourcePaths;
use super::{MCPTransport, MCPConnectionInfo};

// Error types for stdio transport
#[derive(Debug, thiserror::Error)]
pub enum StdioTransportError {
    #[error("Failed to spawn MCP process: {0}")]
    ProcessSpawn(#[from] std::io::Error),
    #[error("MCP client communication failed: {0}")]
    ClientCommunication(String),
    #[error("HTTP server failed to start: {0}")]
    HttpServerStart(String),
    #[error("No available ports in range")]
    NoAvailablePorts,
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Process communication timeout")]
    Timeout,
    // New 2025 spec errors
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Session already exists: {0}")]
    SessionExists(String),
    #[error("Invalid session state: {0}")]
    InvalidSessionState(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Protocol negotiation failed")]
    ProtocolNegotiationFailed,
}

// Session management structures for 2025 spec
#[derive(Debug, Clone)]
struct MCPSession {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    protocol_version: MCPProtocolVersion,
    #[allow(dead_code)]
    created_at: std::time::SystemTime,
    last_activity: Arc<Mutex<std::time::SystemTime>>,
    is_active: Arc<AtomicBool>,
}

impl MCPSession {
    pub fn new(session_id: String, protocol_version: MCPProtocolVersion) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id: session_id,
            protocol_version,
            created_at: now,
            last_activity: Arc::new(Mutex::new(now)),
            is_active: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn update_activity(&self) {
        let mut last_activity = self.last_activity.lock().await;
        *last_activity = std::time::SystemTime::now();
    }

    pub fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    pub fn protocol_version(&self) -> &MCPProtocolVersion {
        &self.protocol_version
    }

    #[allow(dead_code)]
    pub fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }
}

// MCP Client Session - handles direct stdio communication
struct MCPClientSession {
    server_id: Uuid,
    server_name: String,
    child_process: Arc<Mutex<Option<Child>>>,
    request_sender: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    response_handlers: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<MCPResponse>>>>,
    notification_sender: Arc<broadcast::Sender<MCPNotification>>,
}

impl MCPClientSession {
    pub async fn new(server: &MCPServer) -> Result<Self, StdioTransportError> {
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

    pub async fn initialize(&self, server: &MCPServer) -> Result<MCPProtocolVersion, StdioTransportError> {
        // Try 2025 spec first, fallback to 2024
        match self.initialize_with_version(server, MCPProtocolVersion::V2025_06_18).await {
            Ok(version) => Ok(version),
            Err(_) => {
                // Fallback to 2024 version
                self.initialize_with_version(server, MCPProtocolVersion::V2024_11_05).await
            }
        }
    }

    pub async fn initialize_with_version(&self, server: &MCPServer, preferred_version: MCPProtocolVersion) -> Result<MCPProtocolVersion, StdioTransportError> {
        let logger = MCPLogger::new(server.id);

        logger.log_exec("INFO", &format!("Initializing MCP client session for server: {}", server.name));

        // Start the stdio MCP server process with bundled runtime support
        let command = server.command.as_ref()
            .ok_or_else(|| StdioTransportError::ClientCommunication("Command is required for stdio transport".to_string()))?;

        let args: Vec<String> = serde_json::from_value(server.args.clone()).unwrap_or_default();
        let env_vars = self.get_server_env(server).await?;

        logger.log_exec("INFO", &format!("Starting MCP server process: {} {:?}", command, args));

        // Use command resolution for bundled runtime support
        let (resolved_command, resolved_args) = resolve_command(command, &args);

        let mut cmd = Command::new(resolved_command);
        cmd.args(resolved_args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("IS_ZIEE_MCP", "1")
            .kill_on_drop(true);

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().map_err(|e| {
            logger.log_exec("ERROR", &format!("Failed to spawn MCP server process: {}", e));
            StdioTransportError::ProcessSpawn(e)
        })?;

        let pid = child.id();
        logger.log_exec("INFO", &format!("MCP server process started successfully (PID: {:?})", pid));

        let stdin = child.stdin.take().ok_or_else(|| {
            logger.log_exec("ERROR", "Failed to get stdin handle from child process");
            StdioTransportError::ClientCommunication("Failed to get stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            logger.log_exec("ERROR", "Failed to get stdout handle from child process");
            StdioTransportError::ClientCommunication("Failed to get stdout".to_string())
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

        // Send MCP initialize request with version-specific capabilities
        let capabilities = self.create_capabilities_for_version(&preferred_version);

        let init_request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::String("init".to_string())),
            method: methods::INITIALIZE.to_string(),
            params: Some(serde_json::to_value(InitializeRequest {
                protocol_version: preferred_version.as_str().to_string(),
                capabilities,
                client_info: ClientInfo {
                    name: "ziee-mcp-proxy".to_string(),
                    version: "1.0.0".to_string(),
                },
            }).unwrap()),
        };

        let init_response = self.send_request(init_request).await?;

        // Parse the response to confirm the negotiated version
        let negotiated_version = if let Some(result) = init_response.result {
            if let Ok(init_resp) = serde_json::from_value::<InitializeResponse>(result) {
                parse_protocol_version(&init_resp.protocol_version)
            } else {
                preferred_version.clone()
            }
        } else {
            preferred_version.clone()
        };

        println!("[{}] MCP session initialized with protocol version: {}",
                 self.server_name, negotiated_version.as_str());

        // Send notifications/initialized notification as required by MCP protocol
        let initialized_notification = MCPNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
            params: None,
        };

        if let Err(e) = self.send_notification(initialized_notification).await {
            return Err(StdioTransportError::ClientCommunication(
                format!("Failed to send initialized notification: {}", e)
            ));
        }

        println!("[{}] Sent notifications/initialized notification", self.server_name);

        Ok(negotiated_version)
    }

    fn create_capabilities_for_version(&self, version: &MCPProtocolVersion) -> MCPCapabilities {
        let mut capabilities = MCPCapabilities {
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
            // Default 2025 capabilities to None
            session_management: None,
            streaming: None,
            resumable_connections: None,
        };

        // Add 2025-specific capabilities
        if version.is_2025_spec() {
            capabilities.session_management = Some(SessionCapability {
                create: true,
                resume: true,
                terminate: true,
                list: true,
            });
            capabilities.streaming = Some(StreamingCapability {
                sse: true,
                websocket: false,
                http_streaming: true,
            });
            capabilities.resumable_connections = Some(true);
        }

        capabilities
    }

    pub async fn send_request(&self, request: MCPRequest) -> Result<MCPResponse, StdioTransportError> {
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
                .map_err(|e| StdioTransportError::JsonError(e))?;
            let request_line = format!("{}\n", request_str);

            // Log what we're sending to stdin
            logger.log_stdin(&request_str);

            match timeout(Duration::from_secs(5), stdin.write_all(request_line.as_bytes())).await {
                Ok(Ok(_)) => {
                    // Wait for response
                    match timeout(Duration::from_secs(30), response_receiver).await {
                        Ok(Ok(response)) => Ok(response),
                        Ok(Err(_)) => Err(StdioTransportError::ClientCommunication("Response channel closed".to_string())),
                        Err(_) => {
                            // Cleanup handler on timeout
                            self.response_handlers.lock().await.remove(&request_id);
                            Err(StdioTransportError::Timeout)
                        }
                    }
                }
                Ok(Err(e)) => Err(StdioTransportError::ProcessSpawn(e)),
                Err(_) => {
                    self.response_handlers.lock().await.remove(&request_id);
                    Err(StdioTransportError::Timeout)
                }
            }
        } else {
            Err(StdioTransportError::ClientCommunication("No stdin available".to_string()))
        }
    }

    pub async fn send_notification(&self, notification: MCPNotification) -> Result<(), StdioTransportError> {
        if let Some(stdin) = self.request_sender.lock().await.as_mut() {
            let notification_str = serde_json::to_string(&notification)
                .map_err(|e| StdioTransportError::JsonError(e))?;
            let notification_line = format!("{}\n", notification_str);

            match timeout(Duration::from_secs(5), stdin.write_all(notification_line.as_bytes())).await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(e)) => Err(StdioTransportError::ProcessSpawn(e)),
                Err(_) => Err(StdioTransportError::Timeout),
            }
        } else {
            Err(StdioTransportError::ClientCommunication("No stdin available".to_string()))
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

    async fn get_server_env(&self, server: &MCPServer) -> Result<HashMap<String, String>, StdioTransportError> {
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

    pub async fn get_process_pid(&self) -> Option<u32> {
        if let Ok(child_guard) = self.child_process.try_lock() {
            if let Some(child) = child_guard.as_ref() {
                return child.id();
            }
        }
        None
    }
}

// Implement ToolDiscoveryClient trait for MCPClientSession
impl ToolDiscoveryClient for MCPClientSession {
    type Error = StdioTransportError;

    fn send_request(&self, request: MCPRequest) -> impl std::future::Future<Output = Result<MCPResponse, Self::Error>> + Send {
        self.send_request(request)
    }
}

// HTTP Proxy Server - handles HTTP/SSE endpoints
struct MCPProxyServer {
    client_session: Arc<MCPClientSession>,
}

impl MCPProxyServer {
    pub fn new(client_session: Arc<MCPClientSession>) -> Self {
        Self {
            client_session,
        }
    }

    pub async fn handle_request(&self, request: MCPRequest) -> Result<MCPResponse, StdioTransportError> {
        self.client_session.send_request(request).await
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<MCPNotification> {
        self.client_session.subscribe_notifications()
    }
}

// Main Stdio Transport - contains everything
pub struct MCPStdioTransport {
    pub server_id: Uuid,
    pub server_name: String,
    server: MCPServer,
    state: Arc<Mutex<StdioTransportState>>,
}

struct StdioTransportState {
    proxy_port: u16,
    proxy_url: String,
    client_session: Option<Arc<MCPClientSession>>,
    proxy_server: Option<Arc<MCPProxyServer>>,
    // New for 2025 spec
    protocol_version: MCPProtocolVersion,
    sessions: HashMap<String, MCPSession>,
    default_session_id: Option<String>,
}

impl MCPStdioTransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let state = StdioTransportState {
            proxy_port: 0,
            proxy_url: String::new(),
            client_session: None,
            proxy_server: None,
            // Default to latest protocol version
            protocol_version: MCPProtocolVersion::V2025_06_18,
            sessions: HashMap::new(),
            default_session_id: None,
        };

        Ok(Self {
            server_id: server.id,
            server_name: server.name.clone(),
            server: server.clone(),
            state: Arc::new(Mutex::new(state)),
        })
    }

    async fn find_available_port() -> Result<u16, StdioTransportError> {
        for port in 8000..9000 {
            if let Ok(listener) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                drop(listener);
                return Ok(port);
            }
        }
        Err(StdioTransportError::NoAvailablePorts)
    }

    async fn create_http_server(&self) -> Router {
        let state = self.state.lock().await;
        let proxy_server = state.proxy_server.as_ref().unwrap().clone();
        drop(state);

        Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/sse", get(handle_sse_connection))
            .route("/sse/{session_id}", get(handle_sse_connection_with_session))
            .route("/messages/{session_id}", post(handle_sse_message))
            .route("/sessions", post(create_session))
            .route("/sessions/{session_id}", axum::routing::delete(terminate_session))
            .route("/sessions/{session_id}/resume", post(resume_session))
            .route("/health", get(handle_health_check))
            .with_state(proxy_server)
    }

    // Session management methods for 2025 spec
    pub async fn create_session(&self, session_id: Option<String>) -> Result<String, StdioTransportError> {
        let mut state = self.state.lock().await;

        let session_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        if state.sessions.contains_key(&session_id) {
            return Err(StdioTransportError::SessionExists(session_id));
        }

        let session = MCPSession::new(session_id.clone(), state.protocol_version.clone());
        state.sessions.insert(session_id.clone(), session);

        // Set as default session if none exists
        if state.default_session_id.is_none() {
            state.default_session_id = Some(session_id.clone());
        }

        Ok(session_id)
    }

    pub async fn terminate_session(&self, session_id: &str) -> Result<(), StdioTransportError> {
        let mut state = self.state.lock().await;

        if let Some(session) = state.sessions.get(session_id) {
            session.set_active(false);
            state.sessions.remove(session_id);

            // Clear default session if this was it
            if state.default_session_id.as_ref() == Some(&session_id.to_string()) {
                state.default_session_id = None;
            }

            Ok(())
        } else {
            Err(StdioTransportError::SessionNotFound(session_id.to_string()))
        }
    }

    pub async fn resume_session(&self, session_id: &str) -> Result<bool, StdioTransportError> {
        let state = self.state.lock().await;

        if let Some(session) = state.sessions.get(session_id) {
            if session.is_active() {
                session.update_activity().await;
                Ok(true)
            } else {
                Err(StdioTransportError::InvalidSessionState("Session is not active".to_string()))
            }
        } else {
            Err(StdioTransportError::SessionNotFound(session_id.to_string()))
        }
    }

    pub async fn set_protocol_version(&self, version: MCPProtocolVersion) {
        let mut state = self.state.lock().await;
        state.protocol_version = version;
    }

    pub async fn get_protocol_version(&self) -> MCPProtocolVersion {
        let state = self.state.lock().await;
        state.protocol_version.clone()
    }
}

#[async_trait]
impl MCPTransport for MCPStdioTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let logger = MCPLogger::new(self.server_id);
        logger.log_exec("INFO", &format!("Starting MCP stdio transport for server: {}", self.server_name));

        // Find available port
        let port = Self::find_available_port().await?;

        // Create and initialize client session with version negotiation
        let client_session = Arc::new(MCPClientSession::new(&self.server).await?);
        let negotiated_version = client_session.initialize(&self.server).await?;

        // Update our protocol version based on negotiation
        self.set_protocol_version(negotiated_version.clone()).await;

        // Create proxy server
        let proxy_server = Arc::new(MCPProxyServer::new(
            Arc::clone(&client_session),
        ));

        // Store components in state and update database with proxy URL
        let proxy_url = format!("http://127.0.0.1:{}/mcp", port);
        {
            let mut state = self.state.lock().await;
            state.client_session = Some(Arc::clone(&client_session));
            state.proxy_server = Some(Arc::clone(&proxy_server));
            state.proxy_port = port;
            state.proxy_url = proxy_url.clone();
        }

        // Update database with the proxy URL
        if let Err(e) = mcp_servers::update_mcp_server_proxy_url(&self.server_id, &proxy_url).await {
            eprintln!("Failed to update MCP server proxy URL in database: {}", e);
        }

        // Discover tools immediately after server initialization
        let client_session_for_discovery = Arc::clone(&client_session);
        let server_id_for_discovery = self.server_id;
        tokio::spawn(async move {
            match discover_and_cache_tools_direct(server_id_for_discovery, client_session_for_discovery.as_ref()).await {
                Ok(tool_count) => {
                    tracing::info!("Automatically discovered {} tools for server {}", tool_count, server_id_for_discovery);
                }
                Err(e) => {
                    tracing::warn!("Failed to automatically discover tools for server {}: {}", server_id_for_discovery, e);
                }
            }
        });

        // Start HTTP server
        let app = self.create_http_server().await;
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("HTTP server error: {}", e);
            }
        });

        // Get PID from client session
        let pid = client_session.get_process_pid().await;

        Ok(MCPConnectionInfo {
            child: None, // Managed internally
            pid,
            port: Some(port),
        })
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let logger = MCPLogger::new(self.server_id);
        logger.log_exec("INFO", "Stopping MCP stdio transport");

        // Clear proxy URL from database
        if let Err(e) = mcp_servers::update_mcp_server_proxy_url(&self.server_id, "").await {
            eprintln!("Failed to clear MCP server proxy URL in database: {}", e);
        }

        // Cleanup handled by Drop trait and internal session management
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        let state = self.state.lock().await;
        state.client_session.is_some()
    }
}

/// Resolve runtime commands to bundled executables if available
pub fn resolve_command(command: &str, args: &[String]) -> (PathBuf, Vec<String>) {
    match command {
        "npx" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                let mut new_args = vec!["x".to_string()];
                new_args.extend_from_slice(args);
                (bun_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        "node" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                (bun_path, args.to_vec())
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        "npm" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                (bun_path, args.to_vec())
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        "pip" | "pip3" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["pip".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        "uvx" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["tool".to_string(), "run".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        "python" | "python3" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["run".to_string(), "python".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },
        _ => (PathBuf::from(command), args.to_vec())
    }
}

// Security validation for 2025 spec
async fn validate_origin_header(headers: &HeaderMap) -> Result<(), StdioTransportError> {
    if let Some(origin) = headers.get("Origin") {
        let origin_str = origin.to_str()
            .map_err(|_| StdioTransportError::SecurityViolation("Invalid Origin header format".to_string()))?;

        // Allow localhost and local IPs
        if origin_str.starts_with("http://localhost") ||
           origin_str.starts_with("https://localhost") ||
           origin_str.starts_with("http://127.0.0.1") ||
           origin_str.starts_with("https://127.0.0.1") ||
           origin_str.starts_with("http://[::1]") ||
           origin_str.starts_with("https://[::1]") {
            Ok(())
        } else {
            Err(StdioTransportError::SecurityViolation(format!("Unauthorized origin: {}", origin_str)))
        }
    } else {
        // For 2025 spec, Origin header is required
        Err(StdioTransportError::SecurityViolation("Missing required Origin header".to_string()))
    }
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
    headers: HeaderMap,
    State(proxy_server): State<Arc<MCPProxyServer>>,
    Json(request_json): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Parse the JSON as an MCP request
    let request: MCPRequest = serde_json::from_value(request_json)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Detect protocol version from request
    let protocol_version = detect_protocol_version_from_request(&request);

    // Apply version-specific security and handling
    match protocol_version {
        MCPProtocolVersion::V2025_06_18 | MCPProtocolVersion::V2025_03_26 => {
            // Validate Origin header for 2025 spec
            if let Err(_) = validate_origin_header(&headers).await {
                return Err(StatusCode::FORBIDDEN);
            }

            // Handle with session context for 2025
            handle_mcp_request_with_session_v2025(&proxy_server, request).await
        }
        MCPProtocolVersion::V2024_11_05 => {
            // Legacy handling without strict security
            handle_mcp_request_common(&proxy_server, request).await
        }
    }
}

async fn handle_health_check() -> Json<Value> {
    Json(serde_json::json!({"status": "healthy"}))
}

// 2025 spec handler with session support
async fn handle_mcp_request_with_session_v2025(
    proxy_server: &Arc<MCPProxyServer>,
    request: MCPRequest,
) -> Result<Json<Value>, StatusCode> {
    // For now, delegate to common handler
    // TODO: Add session-aware processing here
    handle_mcp_request_common(proxy_server, request).await
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
        return handle_mcp_request_common(&proxy_server, request).await;
    }

    // Try to parse as notification
    if let Ok(notification) = serde_json::from_value::<MCPNotification>(request_json) {
        match proxy_server.client_session.send_notification(notification).await {
            Ok(_) => Ok(Json(serde_json::json!({"status": "sent"}))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Session management handlers for 2025 spec
async fn create_session(
    headers: HeaderMap,
    State(_proxy_server): State<Arc<MCPProxyServer>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Validate Origin header for 2025 spec
    if let Err(_) = validate_origin_header(&headers).await {
        return Err(StatusCode::FORBIDDEN);
    }

    // Extract session_id from request if provided
    let session_id = request.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // TODO: Create session through transport
    let session_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "status": "created"
    })))
}

async fn terminate_session(
    headers: HeaderMap,
    State(_proxy_server): State<Arc<MCPProxyServer>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Validate Origin header for 2025 spec
    if let Err(_) = validate_origin_header(&headers).await {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Terminate session through transport
    println!("Terminating session: {}", session_id);

    Ok(StatusCode::NO_CONTENT)
}

async fn resume_session(
    headers: HeaderMap,
    State(_proxy_server): State<Arc<MCPProxyServer>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // Validate Origin header for 2025 spec
    if let Err(_) = validate_origin_header(&headers).await {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Resume session through transport
    println!("Resuming session: {}", session_id);

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "status": "resumed"
    })))
}

// SSE connection handler with session support for 2025 spec
async fn handle_sse_connection_with_session(
    headers: HeaderMap,
    State(proxy_server): State<Arc<MCPProxyServer>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    // Validate Origin header for 2025 spec
    if let Err(_) = validate_origin_header(&headers).await {
        return axum::response::Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("Forbidden".to_string())
            .unwrap()
            .into_response();
    }

    println!("SSE connection with session: {}", session_id);

    // For now, delegate to regular SSE handler
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
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}