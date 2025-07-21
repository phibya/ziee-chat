use crate::ai::core::provider_base::{build_http_client_with_config, HttpClientConfig};
use crate::ai::core::providers::ProxyConfig;
use crate::database::models::ProxySettings;

/// Test proxy connectivity using a common HTTP test endpoint
pub async fn test_proxy_connectivity(proxy_config: &ProxySettings) -> Result<(), String> {
    // Note: We don't check if proxy is enabled here, as this function is used for testing
    // The caller should decide whether to test based on enabled status

    // Validate proxy URL format
    if proxy_config.url.trim().is_empty() {
        return Err("Proxy URL is empty".to_string());
    }

    // Convert ProxySettings to ProxyConfig for use with common client builder
    let proxy_config_converted = ProxyConfig {
        enabled: true, // Always enabled for testing
        url: proxy_config.url.clone(),
        username: if proxy_config.username.is_empty() {
            None
        } else {
            Some(proxy_config.username.clone())
        },
        password: if proxy_config.password.is_empty() {
            None
        } else {
            Some(proxy_config.password.clone())
        },
        no_proxy: proxy_config
            .no_proxy
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        ignore_ssl_certificates: proxy_config.ignore_ssl_certificates,
    };

    // Use the common HTTP client builder with proxy testing configuration
    let client = build_http_client_with_config(
        "https://httpbin.org/ip", // Test URL for proxy validation
        Some(&proxy_config_converted),
        &HttpClientConfig::for_proxy_testing(),
    )
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
