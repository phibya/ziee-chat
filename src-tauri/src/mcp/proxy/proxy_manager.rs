use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::database::models::mcp_server::{MCPServer, MCPTransportType};
use super::{stdio_proxy::MCPStdioProxy, ProxyError};

pub struct MCPProxyManager {
    proxies: Arc<RwLock<HashMap<Uuid, MCPStdioProxy>>>,
    port_allocator: Arc<Mutex<PortAllocator>>,
}

impl MCPProxyManager {
    pub fn new() -> Self {
        Self {
            proxies: Arc::new(RwLock::new(HashMap::new())),
            port_allocator: Arc::new(Mutex::new(PortAllocator::new(9000, 9999))),
        }
    }

    pub async fn start_proxy(&self, server: &MCPServer) -> Result<u16, ProxyError> {
        // Only create proxy for stdio transport
        if server.transport_type != MCPTransportType::Stdio {
            return Err(ProxyError::UnsupportedTransport);
        }

        // Check if proxy already exists
        {
            let proxies = self.proxies.read().await;
            if proxies.contains_key(&server.id) {
                if let Some(proxy) = proxies.get(&server.id) {
                    return Ok(proxy.proxy_port);
                }
            }
        }

        // Allocate a port
        let port = {
            let mut allocator = self.port_allocator.lock().await;
            allocator.allocate().ok_or(ProxyError::NoAvailablePorts)?
        };

        // Create and start proxy
        let mut proxy = MCPStdioProxy::new(server).await?;
        proxy.start(port).await.map_err(|e| {
            // Release port on failure
            tokio::spawn({
                let allocator = Arc::clone(&self.port_allocator);
                async move {
                    allocator.lock().await.release(port);
                }
            });
            e
        })?;

        // Store proxy
        {
            let mut proxies = self.proxies.write().await;
            proxies.insert(server.id, proxy);
        }

        println!("Started proxy for MCP server '{}' on port {}", server.name, port);
        Ok(port)
    }

    pub async fn stop_proxy(&self, server_id: &Uuid) -> Result<(), ProxyError> {
        let proxy = {
            let mut proxies = self.proxies.write().await;
            proxies.remove(server_id)
        };

        if let Some(mut proxy) = proxy {
            let port = proxy.proxy_port;
            proxy.stop().await?;

            // Release port
            let mut allocator = self.port_allocator.lock().await;
            allocator.release(port);

            println!("Stopped proxy for MCP server ID: {}", server_id);
        }

        Ok(())
    }

    pub async fn get_proxy_url(&self, server_id: &Uuid) -> Option<String> {
        let proxies = self.proxies.read().await;
        proxies.get(server_id).map(|proxy| proxy.proxy_url.clone())
    }

    pub async fn get_proxy_port(&self, server_id: &Uuid) -> Option<u16> {
        let proxies = self.proxies.read().await;
        proxies.get(server_id).map(|proxy| proxy.proxy_port)
    }

    pub async fn is_proxy_healthy(&self, server_id: &Uuid) -> bool {
        let proxies = self.proxies.read().await;
        if let Some(proxy) = proxies.get(server_id) {
            proxy.is_healthy().await
        } else {
            false
        }
    }

    pub async fn list_running_proxies(&self) -> Vec<(Uuid, u16, String)> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .map(|(id, proxy)| (*id, proxy.proxy_port, proxy.server_name.clone()))
            .collect()
    }

    pub async fn shutdown_all_proxies(&self) -> Result<(), ProxyError> {
        println!("Shutting down all MCP proxies...");

        let server_ids: Vec<Uuid> = {
            let proxies = self.proxies.read().await;
            proxies.keys().copied().collect()
        };

        for server_id in server_ids {
            if let Err(e) = self.stop_proxy(&server_id).await {
                eprintln!("Failed to stop proxy for server {}: {}", server_id, e);
            }
        }

        // Clear allocator
        let mut allocator = self.port_allocator.lock().await;
        allocator.clear();

        println!("All MCP proxies shut down");
        Ok(())
    }
}

impl Default for MCPProxyManager {
    fn default() -> Self {
        Self::new()
    }
}

struct PortAllocator {
    range_start: u16,
    range_end: u16,
    allocated: HashSet<u16>,
}

impl PortAllocator {
    pub fn new(range_start: u16, range_end: u16) -> Self {
        Self {
            range_start,
            range_end,
            allocated: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> Option<u16> {
        for port in self.range_start..=self.range_end {
            if !self.allocated.contains(&port) && self.is_port_available(port) {
                self.allocated.insert(port);
                return Some(port);
            }
        }
        None
    }

    pub fn release(&mut self, port: u16) {
        self.allocated.remove(&port);
    }

    pub fn clear(&mut self) {
        self.allocated.clear();
    }

    fn is_port_available(&self, port: u16) -> bool {
        // Try to bind to the port to check if it's available
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

// Global proxy manager instance
static MCP_PROXY_MANAGER: OnceLock<MCPProxyManager> = OnceLock::new();

pub fn get_proxy_manager() -> &'static MCPProxyManager {
    MCP_PROXY_MANAGER.get_or_init(|| MCPProxyManager::new())
}

pub async fn shutdown_all_mcp_proxies() -> Result<(), ProxyError> {
    let proxy_manager = get_proxy_manager();
    proxy_manager.shutdown_all_proxies().await
}