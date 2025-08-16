use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};

use crate::database::models::api_proxy_server_model::ApiProxyServerTrustedHost;
use crate::database::queries::api_proxy_server_models;
use super::{ProxyError, log_security_event};

#[derive(Debug)]
pub struct SecurityValidator {
    trusted_hosts: Arc<RwLock<Vec<ApiProxyServerTrustedHost>>>,
}

impl SecurityValidator {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let trusted_hosts = load_trusted_hosts().await?;
        Ok(Self {
            trusted_hosts: Arc::new(RwLock::new(trusted_hosts)),
        })
    }
    
    pub async fn reload_trusted_hosts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let hosts = load_trusted_hosts().await?;
        *self.trusted_hosts.write().await = hosts;
        Ok(())
    }
    
    pub async fn validate_host(&self, client_ip: &str) -> Result<bool, ProxyError> {
        let trusted_hosts = self.trusted_hosts.read().await;
        
        for host in trusted_hosts.iter() {
            if !host.enabled {
                continue;
            }
            
            if self.matches_host_pattern(&host.host, client_ip)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn matches_host_pattern(&self, pattern: &str, client_ip: &str) -> Result<bool, ProxyError> {
        // Handle exact matches first
        if pattern == client_ip {
            return Ok(true);
        }
        
        // Handle domain names (localhost special case)
        if pattern == "localhost" && (client_ip == "127.0.0.1" || client_ip == "::1") {
            return Ok(true);
        }
        
        // Handle CIDR notation
        if pattern.contains('/') {
            return self.matches_cidr(pattern, client_ip);
        }
        
        // Handle IP address
        self.matches_ip(pattern, client_ip)
    }
    
    fn matches_cidr(&self, cidr: &str, client_ip: &str) -> Result<bool, ProxyError> {
        // Parse client IP
        let client_addr: IpAddr = client_ip.parse()
            .map_err(|_| ProxyError::InvalidClientIP(client_ip.to_string()))?;
        
        // Parse CIDR
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(ProxyError::InvalidCIDR(cidr.to_string()));
        }
        
        let network_ip: IpAddr = parts[0].parse()
            .map_err(|_| ProxyError::InvalidCIDR(cidr.to_string()))?;
        let prefix_len: u8 = parts[1].parse()
            .map_err(|_| ProxyError::InvalidCIDR(cidr.to_string()))?;
        
        // Check if client IP is in the CIDR range
        match (network_ip, client_addr) {
            (IpAddr::V4(net), IpAddr::V4(client)) => {
                if prefix_len > 32 {
                    return Err(ProxyError::InvalidCIDR(cidr.to_string()));
                }
                let mask = !((1u32 << (32 - prefix_len)) - 1);
                Ok((u32::from(net) & mask) == (u32::from(client) & mask))
            }
            (IpAddr::V6(net), IpAddr::V6(client)) => {
                if prefix_len > 128 {
                    return Err(ProxyError::InvalidCIDR(cidr.to_string()));
                }
                let net_bytes = net.octets();
                let client_bytes = client.octets();
                
                let full_bytes = (prefix_len / 8) as usize;
                let remaining_bits = prefix_len % 8;
                
                // Check full bytes
                if net_bytes[..full_bytes] != client_bytes[..full_bytes] {
                    return Ok(false);
                }
                
                // Check remaining bits if any
                if remaining_bits > 0 && full_bytes < 16 {
                    let mask = !((1u8 << (8 - remaining_bits)) - 1);
                    return Ok((net_bytes[full_bytes] & mask) == (client_bytes[full_bytes] & mask));
                }
                
                Ok(true)
            }
            _ => Ok(false), // IPv4 vs IPv6 mismatch
        }
    }
    
    fn matches_ip(&self, pattern: &str, client_ip: &str) -> Result<bool, ProxyError> {
        Ok(pattern == client_ip)
    }
}

async fn load_trusted_hosts() -> Result<Vec<ApiProxyServerTrustedHost>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(api_proxy_server_models::get_enabled_trusted_hosts().await?)
}

pub async fn host_validation_middleware(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    Extension(security): Extension<Arc<SecurityValidator>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode>
{
    let client_ip = remote_addr.ip().to_string();
    
    match security.validate_host(&client_ip).await {
        Ok(true) => {
            // Log successful validation at debug level
            tracing::debug!("Trusted host access: {}", client_ip);
            Ok(next.run(request).await)
        }
        Ok(false) => {
            // Log blocked access at warn level
            log_security_event("BLOCKED_HOST", &client_ip, "Host not in trusted list");
            Err(StatusCode::FORBIDDEN)
        }
        Err(e) => {
            // Log validation error
            tracing::error!("Host validation error for {}: {}", client_ip, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}