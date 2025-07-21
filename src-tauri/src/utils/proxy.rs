use crate::database::models::ProxySettings;
use reqwest;
use reqwest::NoProxy;

/// Test proxy connectivity using a common HTTP test endpoint
pub async fn test_proxy_connectivity(proxy_config: &ProxySettings) -> Result<(), String> {
    // Note: We don't check if proxy is enabled here, as this function is used for testing
    // The caller should decide whether to test based on enabled status

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

    // Handle no_proxy list - domains that should bypass the proxy
    if !proxy_config.no_proxy.trim().is_empty() {
        proxy_builder = proxy_builder.no_proxy(NoProxy::from_string(&proxy_config.no_proxy));
    }

    // Build the client with proxy and SSL settings
    let mut client_builder = reqwest::Client::builder()
        .proxy(proxy_builder)
        .timeout(std::time::Duration::from_secs(30)); // Increased timeout for proxy connections

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

    let client = client_builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Test the proxy by making a request to a reliable endpoint
    // Using httpbin.org as it's a simple testing service that returns IP info
    // Try HTTP first for better compatibility with local proxies
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
