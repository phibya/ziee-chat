use serde::{Deserialize, Serialize};

/// Common proxy settings structure used for both system-wide and provider-specific proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub no_proxy: String,
    #[serde(default)]
    pub ignore_ssl_certificates: bool,
    // #[serde(default)]
    // pub proxy_ssl: bool,
    // #[serde(default)]
    // pub proxy_host_ssl: bool,
    // #[serde(default)]
    // pub peer_ssl: bool,
    // #[serde(default)]
    // pub host_ssl: bool,
}

/// Convert from API configuration TestProxyConnectionRequest
impl From<&crate::api::configuration::TestProxyConnectionRequest> for ProxySettings {
    fn from(request: &crate::api::configuration::TestProxyConnectionRequest) -> Self {
        ProxySettings {
            enabled: request.enabled,
            url: request.url.clone(),
            username: request.username.clone(),
            password: request.password.clone(),
            no_proxy: request.no_proxy.clone(),
            ignore_ssl_certificates: request.ignore_ssl_certificates,
            // proxy_ssl: request.proxy_ssl,
            // proxy_host_ssl: request.proxy_host_ssl,
            // peer_ssl: request.peer_ssl,
            // host_ssl: request.host_ssl,
        }
    }
}
