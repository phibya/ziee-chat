use super::providers::ProxyConfig;
use reqwest::Client;

/// Common HTTP client builder that handles proxy configuration
/// This can be used by all providers regardless of their API format
pub fn build_http_client(
    base_url: &str,
    proxy_config: Option<&ProxyConfig>,
) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let mut client_builder = Client::builder();

    // Configure proxy if provided
    if let Some(proxy_config) = proxy_config {
        if proxy_config.enabled && !proxy_config.url.is_empty() {
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
