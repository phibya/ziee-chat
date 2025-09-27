use async_trait::async_trait;
use std::process::Child;
use crate::database::models::mcp_server::{MCPServer, MCPTransportType};

pub mod stdio;
pub mod http;
pub mod sse;
pub mod proxy;

#[derive(Debug)]
pub struct MCPConnectionInfo {
    pub child: Option<Child>,
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

#[async_trait]
pub trait MCPTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>>;
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn is_healthy(&self) -> bool;
}

pub async fn create_mcp_transport(
    server: &MCPServer,
) -> Result<Box<dyn MCPTransport + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    match server.transport_type {
        MCPTransportType::Stdio => {
            // For stdio servers, use proxy transport to wrap them in HTTP
            Ok(Box::new(proxy::MCPProxyTransport::new(server)?))
        },
        MCPTransportType::Http => Ok(Box::new(http::MCPHttpTransport::new(server)?)),
        MCPTransportType::Sse => Ok(Box::new(sse::MCPSSETransport::new(server)?)),
    }
}