use super::providers::ProxyConfig;
use reqwest::Client;
use std::time::Duration;

/// Configuration options for HTTP client creation
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Optional timeout in seconds (defaults to reqwest's default if None)
    pub timeout_seconds: Option<u64>,
    /// Whether to use the client for proxy testing (affects timeout)
    pub for_proxy_testing: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: None,
            for_proxy_testing: false,
        }
    }
}

impl HttpClientConfig {
    /// Create config for proxy testing with extended timeout
    pub fn for_proxy_testing() -> Self {
        Self {
            timeout_seconds: Some(30), // Extended timeout for proxy connections
            for_proxy_testing: true,
        }
    }

    /// Create config with custom timeout
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        Self {
            timeout_seconds: Some(timeout_seconds),
            for_proxy_testing: false,
        }
    }
}

/// Common HTTP client builder that handles proxy configuration
/// This can be used by all providers regardless of their API format
pub fn build_http_client(
    base_url: &str,
    proxy_config: Option<&ProxyConfig>,
) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    build_http_client_with_config(base_url, proxy_config, &HttpClientConfig::default())
}

/// Enhanced HTTP client builder with configurable options
/// This replaces all duplicate HTTP client creation logic across providers and utilities
pub fn build_http_client_with_config(
    base_url: &str,
    proxy_config: Option<&ProxyConfig>,
    config: &HttpClientConfig,
) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let mut client_builder = Client::builder();

    // Configure timeout if specified
    if let Some(timeout_seconds) = config.timeout_seconds {
        client_builder = client_builder.timeout(Duration::from_secs(timeout_seconds));
    }

    // Configure proxy if provided
    if let Some(proxy_config) = proxy_config {
        if proxy_config.enabled && !proxy_config.url.is_empty() {
            // Validate proxy URL protocol
            if let Ok(proxy_url) = reqwest::Url::parse(&proxy_config.url) {
                match proxy_url.scheme() {
                    "http" | "https" | "socks5" => {
                        // Valid proxy protocol, continue
                    }
                    _ => {
                        return Err(format!(
                            "Invalid proxy protocol '{}'. Only http://, https://, and socks5:// are supported",
                            proxy_url.scheme()
                        ).into());
                    }
                }
            } else {
                return Err("Invalid proxy URL format".into());
            }

            // Check if the base URL should bypass proxy based on no_proxy list
            let should_use_proxy = if let Ok(url) = reqwest::Url::parse(base_url) {
                !proxy_config.no_proxy.iter().any(|no_proxy_host| {
                    url.host_str()
                        .map(|host| host.contains(no_proxy_host) || no_proxy_host.contains(host))
                        .unwrap_or(false)
                })
            } else {
                true // If URL parsing fails, use proxy by default
            };

            if should_use_proxy {
                let mut proxy = reqwest::Proxy::all(&proxy_config.url)?;

                if let (Some(username), Some(password)) =
                    (&proxy_config.username, &proxy_config.password)
                {
                    proxy = proxy.basic_auth(username, password);
                }

                client_builder = client_builder.proxy(proxy);
            }
        }

        if proxy_config.ignore_ssl_certificates {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }
    }

    Ok(client_builder.build()?)
}
