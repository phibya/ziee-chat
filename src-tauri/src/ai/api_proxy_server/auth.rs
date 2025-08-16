use axum::{
    extract::Extension,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::database::models::api_proxy_server_model::ApiProxyServerConfig;
use super::{ProxyError, log_security_event};

pub async fn validate_api_key(
    headers: &HeaderMap,
    expected_key: &str,
) -> Result<(), ProxyError> {
    if expected_key.is_empty() {
        return Ok(()); // No API key required
    }
    
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));
        
    match auth_header {
        Some(key) if key == expected_key => Ok(()),
        _ => Err(ProxyError::Unauthorized),
    }
}

pub async fn auth_middleware(
    Extension(config): Extension<ApiProxyServerConfig>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode>
{
    match validate_api_key(&headers, &config.api_key).await {
        Ok(()) => Ok(next.run(request).await),
        Err(ProxyError::Unauthorized) => {
            // Extract client IP for logging (if available from previous middleware)
            let client_ip = headers
                .get("X-Forwarded-For")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown");
            
            log_security_event("INVALID_API_KEY", client_ip, "Invalid or missing API key");
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}