use async_trait::async_trait;
use crate::database::models::mcp_server::MCPServer;
use crate::mcp::proxy::get_proxy_manager;
use super::{MCPTransport, MCPConnectionInfo};

pub struct MCPProxyTransport {
    server: MCPServer,
}

impl MCPProxyTransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            server: server.clone(),
        })
    }
}

#[async_trait]
impl MCPTransport for MCPProxyTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        // Start the proxy for this stdio server
        let proxy_manager = get_proxy_manager();
        let proxy_port = proxy_manager.start_proxy(&self.server).await?;

        // Return connection info with proxy port
        Ok(MCPConnectionInfo {
            child: None, // Proxy manages the child process
            pid: None,   // No direct PID, proxy manages it
            port: Some(proxy_port),
        })
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Stop the proxy for this server
        let proxy_manager = get_proxy_manager();
        proxy_manager.stop_proxy(&self.server.id).await?;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // Check if proxy is healthy
        let proxy_manager = get_proxy_manager();
        proxy_manager.is_proxy_healthy(&self.server.id).await
    }
}