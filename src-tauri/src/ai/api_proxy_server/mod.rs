pub mod auth;
pub mod logging;
pub mod registry;
pub mod router;
pub mod security;
pub mod server;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use auth::*;
pub use logging::*;
pub use registry::*;
pub use router::*;
pub use security::*;
pub use server::*;

use crate::database::models::api_proxy_server_model::*;

/// HTTP forwarding provider trait for AI providers
#[async_trait]
pub trait HttpForwardingProvider: Send + Sync {
    /// Forward request to provider's API and return raw response
    async fn forward_request(
        &self, 
        request: serde_json::Value
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>;
}

// Global instance for the API proxy server
static PROXY_SERVER_INSTANCE: tokio::sync::OnceCell<Arc<RwLock<Option<ApiProxyServer>>>> =
    tokio::sync::OnceCell::const_new();

/// Get the global proxy server instance
pub async fn get_proxy_server_instance() -> Arc<RwLock<Option<ApiProxyServer>>> {
    PROXY_SERVER_INSTANCE
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await
        .clone()
}

/// Set the global proxy server instance
pub async fn set_proxy_server_instance(server: ApiProxyServer) {
    let instance = get_proxy_server_instance().await;
    *instance.write().await = Some(server);
}

/// Clear the global proxy server instance
pub async fn clear_proxy_server_instance() {
    let instance = get_proxy_server_instance().await;
    *instance.write().await = None;
}

// Configuration helper functions
pub async fn get_proxy_config() -> Result<ApiProxyServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let port = get_config_value("api_proxy_server_port", 8080).await?;
    let address = get_config_value("api_proxy_server_address", "127.0.0.1".to_string()).await?;
    let prefix = get_config_value("api_proxy_server_prefix", "/v1".to_string()).await?;
    let api_key = get_config_value("api_proxy_server_api_key", "".to_string()).await?;
    let allow_cors = get_config_value("api_proxy_server_allow_cors", true).await?;
    let log_level = get_config_value("api_proxy_server_log_level", "info".to_string()).await?;
    let autostart_on_startup = get_config_value("api_proxy_server_autostart_on_startup", false).await?;
    
    Ok(ApiProxyServerConfig {
        port,
        address,
        prefix,
        api_key,
        allow_cors,
        log_level,
        autostart_on_startup,
    })
}

pub async fn update_proxy_config(config: &ApiProxyServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_config_value("api_proxy_server_port", &config.port).await?;
    set_config_value("api_proxy_server_address", &config.address).await?;
    set_config_value("api_proxy_server_prefix", &config.prefix).await?;
    set_config_value("api_proxy_server_api_key", &config.api_key).await?;
    set_config_value("api_proxy_server_allow_cors", &config.allow_cors).await?;
    set_config_value("api_proxy_server_log_level", &config.log_level).await?;
    set_config_value("api_proxy_server_autostart_on_startup", &config.autostart_on_startup).await?;
    Ok(())
}

async fn get_config_value<T>(key: &str, default: T) -> Result<T, Box<dyn std::error::Error + Send + Sync>> 
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    match crate::database::queries::configuration::get_configuration(key).await? {
        Some(config) => Ok(serde_json::from_value(config.value)?),
        None => {
            set_config_value(key, &default).await?;
            Ok(default)
        }
    }
}

async fn set_config_value<T>(key: &str, value: &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: serde::Serialize,
{
    crate::database::queries::configuration::set_configuration(
        key,
        &serde_json::to_value(value)?,
        None,
    ).await?;
    Ok(())
}

// Public API functions
pub async fn start_proxy_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = get_proxy_config().await?;
    
    // Check if already running
    let instance = get_proxy_server_instance().await;
    if instance.read().await.is_some() {
        return Err("API Proxy Server is already running".into());
    }
    
    // Start the server
    let server = ApiProxyServer::start(config).await?;
    set_proxy_server_instance(server).await;
    
    Ok(())
}

pub async fn stop_proxy_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_proxy_server_instance().await;
    let mut server_lock = instance.write().await;
    
    if let Some(server) = server_lock.take() {
        server.stop().await?;
        tracing::info!("API Proxy Server stopped");
    }
    
    Ok(())
}

pub async fn get_proxy_server_status() -> Result<ApiProxyServerStatus, Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_proxy_server_instance().await;
    let server_lock = instance.read().await;
    
    let running = server_lock.is_some();
    let config = get_proxy_config().await?;
    
    let (active_models, server_url) = if running {
        // Get active models count from registry
        if let Some(server) = server_lock.as_ref() {
            let active_models = server.get_active_models_count().await;
            let server_url = Some(format!("http://{}:{}{}", config.address, config.port, config.prefix));
            (active_models, server_url)
        } else {
            (0, None)
        }
    } else {
        (0, None)
    };
    
    Ok(ApiProxyServerStatus {
        running,
        active_models,
        server_url,
    })
}

/// Check if autostart is enabled and start the proxy server if valid
pub async fn try_autostart_proxy_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = get_proxy_config().await?;
    
    // Only autostart if enabled
    if !config.autostart_on_startup {
        tracing::info!("API Proxy Server autostart is disabled");
        return Ok(());
    }
    
    // Check if already running
    let instance = get_proxy_server_instance().await;
    if instance.read().await.is_some() {
        tracing::info!("API Proxy Server is already running, skipping autostart");
        return Ok(());
    }
    
    // Validate configuration before starting
    if let Err(e) = validate_proxy_config(&config).await {
        tracing::warn!("API Proxy Server autostart skipped due to invalid config: {}", e);
        return Ok(()); // Don't fail startup, just skip autostart
    }
    
    // Try to start the server
    match start_proxy_server().await {
        Ok(()) => {
            tracing::info!("API Proxy Server auto-started successfully on {}:{}{}", 
                config.address, config.port, config.prefix);
        }
        Err(e) => {
            tracing::warn!("API Proxy Server autostart failed: {}", e);
            // Don't fail startup, just log the warning
        }
    }
    
    Ok(())
}

/// Reload proxy server models registry
pub async fn reload_proxy_models() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_proxy_server_instance().await;
    let server_guard = instance.read().await;
    
    if let Some(server) = server_guard.as_ref() {
        server.reload_models_only().await?;
        tracing::info!("API proxy server models registry reloaded");
    } else {
        return Err("API proxy server is not running".into());
    }
    
    Ok(())
}

/// Reload proxy server trusted hosts
pub async fn reload_proxy_trusted_hosts() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_proxy_server_instance().await;
    let server_guard = instance.read().await;
    
    if let Some(server) = server_guard.as_ref() {
        server.reload_trusted_hosts_only().await?;
        tracing::info!("API proxy server trusted hosts reloaded");
    } else {
        return Err("API proxy server is not running".into());
    }
    
    Ok(())
}

/// Validate proxy server configuration
async fn validate_proxy_config(config: &ApiProxyServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if port is valid and available
    if config.port == 0 {
        return Err("Invalid port: 0".into());
    }
    
    // Check if address is valid
    if config.address.trim().is_empty() {
        return Err("Invalid address: empty".into());
    }
    
    // Check if there are any enabled models
    let enabled_models = crate::database::queries::api_proxy_server_models::get_enabled_proxy_models().await?;
    if enabled_models.is_empty() {
        return Err("No enabled models configured for proxy".into());
    }
    
    Ok(())
}


#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Model {0} not configured in proxy")]
    ModelNotInProxy(Uuid),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Provider {0} not found")]
    ProviderNotFound(Uuid),
    
    #[error("Local model {0} not running")]
    LocalModelNotRunning(Uuid),
    
    #[error("Remote provider {0} missing base URL")]
    RemoteProviderMissingBaseUrl(Uuid),
    
    #[error("No default model configured")]
    NoDefaultModel,
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Host not trusted: {0}")]
    HostNotTrusted(String),
    
    #[error("Invalid client IP: {0}")]
    InvalidClientIP(String),
    
    #[error("Invalid CIDR notation: {0}")]
    InvalidCIDR(String),
    
    #[error("Model server unreachable: {0}")]
    ServerUnreachable(String),
    
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Proxy server not enabled")]
    ProxyDisabled,
    
    #[error("Internal error")]
    InternalError,
}