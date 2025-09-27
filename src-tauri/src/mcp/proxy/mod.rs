pub mod stdio_proxy;
pub mod proxy_manager;

pub use stdio_proxy::MCPStdioProxy;
pub use proxy_manager::{MCPProxyManager, get_proxy_manager, shutdown_all_mcp_proxies};

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Failed to spawn MCP process: {0}")]
    ProcessSpawn(#[from] std::io::Error),

    #[error("MCP client communication failed: {0}")]
    ClientCommunication(String),

    #[error("HTTP server failed to start: {0}")]
    HttpServerStart(String),

    #[error("No available ports in range")]
    NoAvailablePorts,

    #[error("Unsupported transport type for proxy")]
    UnsupportedTransport,

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Process communication timeout")]
    Timeout,
}