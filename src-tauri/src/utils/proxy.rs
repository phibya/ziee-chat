use reqwest;
use serde::{Deserialize, Serialize};

/// Common proxy configuration for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    pub proxy_ssl: bool,
    pub proxy_host_ssl: bool,
    pub peer_ssl: bool,
    pub host_ssl: bool,
}

/// Test proxy connectivity using a common HTTP test endpoint
pub async fn test_proxy_connectivity(proxy_config: &ProxyConfig) -> Result<(), String> {
    // Check if proxy is meant to be enabled
    if !proxy_config.enabled {
        return Err("Proxy is not enabled".to_string());
    }

    // Validate proxy URL format
    if proxy_config.url.trim().is_empty() {
        return Err("Proxy URL is empty".to_string());
    }

    // Parse and validate the proxy URL
    let _proxy_url = reqwest::Url::parse(&proxy_config.url)
        .map_err(|e| format!("Invalid proxy URL format: {}", e))?;

    // Create a reqwest client with proxy configuration
    let mut proxy_builder = reqwest::Proxy::all(&proxy_config.url)
        .map_err(|e| format!("Failed to create proxy: {}", e))?;

    // Add authentication if provided
    if !proxy_config.username.is_empty() {
        proxy_builder = proxy_builder.basic_auth(&proxy_config.username, &proxy_config.password);
    }

    // Build the client with proxy and SSL settings
    let mut client_builder = reqwest::Client::builder()
        .proxy(proxy_builder)
        .timeout(std::time::Duration::from_secs(30)) // Increased timeout for proxy connections
        .no_proxy(); // Disable system proxy to ensure we only use our configured proxy

    // Configure SSL verification based on settings
    if proxy_config.ignore_ssl_certificates {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    // Apply additional SSL settings if needed
    if proxy_config.proxy_ssl || proxy_config.host_ssl || proxy_config.peer_ssl {
        // These fields control specific SSL verification aspects
        // For now, we use ignore_ssl_certificates as a general flag
        // Future implementations might need more granular control
    }

    // Handle no_proxy list - domains that should bypass the proxy
    if !proxy_config.no_proxy.trim().is_empty() {
        // Parse the no_proxy list (comma-separated domains)
        // This is handled by the proxy configuration itself
        // The actual implementation would depend on the proxy behavior
    }

    let client = client_builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Test the proxy by making a request to a reliable endpoint
    // Using httpbin.org as it's a simple testing service that returns IP info
    let test_url = "https://httpbin.org/ip";

    match client.get(test_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // Try to read the response to ensure it's valid
                match response.text().await {
                    Ok(body) => {
                        // Verify the response contains expected IP information
                        if body.contains("origin") {
                            Ok(())
                        } else {
                            Err(format!("Unexpected response format: {}", body))
                        }
                    }
                    Err(e) => Err(format!("Failed to read response body: {}", e)),
                }
            } else {
                Err(format!(
                    "HTTP request failed with status: {}",
                    response.status()
                ))
            }
        }
        Err(e) => {
            // Check if it's a proxy-related error
            let error_msg = e.to_string();
            if error_msg.contains("proxy") || error_msg.contains("CONNECT") {
                Err(format!("Proxy connection failed: {}", e))
            } else if error_msg.contains("timeout") {
                Err("Proxy connection timed out".to_string())
            } else if error_msg.contains("dns") {
                Err(format!(
                    "DNS resolution failed (check proxy settings): {}",
                    e
                ))
            } else {
                Err(format!("Network request failed: {}", e))
            }
        }
    }
}

/// Convert from configuration.rs TestProxyConnectionRequest
impl From<&crate::api::configuration::TestProxyConnectionRequest> for ProxyConfig {
    fn from(request: &crate::api::configuration::TestProxyConnectionRequest) -> Self {
        ProxyConfig {
            enabled: request.enabled,
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password.clone(),
            no_proxy: request.no_proxy.clone(),
            ignore_ssl_certificates: request.ignore_ssl_certificates,
            proxy_ssl: request.proxy_ssl,
            proxy_host_ssl: request.proxy_host_ssl,
            peer_ssl: request.peer_ssl,
            host_ssl: request.host_ssl,
        }
    }
}

/// Convert from providers.rs ProviderProxySettings
impl From<&crate::database::models::ProviderProxySettings> for ProxyConfig {
    fn from(settings: &crate::database::models::ProviderProxySettings) -> Self {
        ProxyConfig {
            enabled: settings.enabled,
            url: settings.url.clone(),
            username: settings.username.clone(),
            password: settings.password.clone(),
            no_proxy: settings.no_proxy.clone(),
            ignore_ssl_certificates: settings.ignore_ssl_certificates,
            proxy_ssl: settings.proxy_ssl,
            proxy_host_ssl: settings.proxy_host_ssl,
            peer_ssl: settings.peer_ssl,
            host_ssl: settings.host_ssl,
        }
    }
}