use crate::api::middleware::AuthenticatedUser;
use crate::database::queries::configuration::{
    get_default_language, get_proxy_no_proxy, get_proxy_password, get_proxy_url,
    get_proxy_username, is_host_ssl, is_peer_ssl, is_proxy_enabled, is_proxy_host_ssl,
    is_proxy_ignore_ssl_certificates, is_proxy_ssl, is_user_registration_enabled,
    set_default_language, set_host_ssl, set_peer_ssl, set_proxy_enabled, set_proxy_host_ssl,
    set_proxy_ignore_ssl_certificates, set_proxy_no_proxy, set_proxy_password, set_proxy_ssl,
    set_proxy_url, set_proxy_username, set_user_registration_enabled,
};
use axum::{http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserRegistrationStatusResponse {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserRegistrationRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct DefaultLanguageResponse {
    pub language: String,
}

#[derive(Deserialize)]
pub struct UpdateDefaultLanguageRequest {
    pub language: String,
}

#[derive(Serialize)]
pub struct ProxySettingsResponse {
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

#[derive(Deserialize)]
pub struct UpdateProxySettingsRequest {
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

#[derive(Deserialize)]
pub struct TestProxyConnectionRequest {
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

#[derive(Serialize)]
pub struct TestProxyConnectionResponse {
    pub success: bool,
    pub message: String,
}

// Public endpoint to check registration status (no auth required)
pub async fn get_user_registration_status(
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok(Json(UserRegistrationStatusResponse { enabled })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to check registration status
pub async fn get_user_registration_status_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok(Json(UserRegistrationStatusResponse { enabled })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to update registration status
pub async fn update_user_registration_status(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateUserRegistrationRequest>,
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match set_user_registration_enabled(request.enabled).await {
        Ok(_) => Ok(Json(UserRegistrationStatusResponse {
            enabled: request.enabled,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Public endpoint to get default language (no auth required)
pub async fn get_default_language_public() -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match get_default_language().await {
        Ok(language) => Ok(Json(DefaultLanguageResponse { language })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to get default language
pub async fn get_default_language_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match get_default_language().await {
        Ok(language) => Ok(Json(DefaultLanguageResponse { language })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to update default language
pub async fn update_default_language(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateDefaultLanguageRequest>,
) -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match set_default_language(&request.language).await {
        Ok(_) => Ok(Json(DefaultLanguageResponse {
            language: request.language,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to get proxy settings
pub async fn get_proxy_settings(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<ProxySettingsResponse>, StatusCode> {
    let enabled = is_proxy_enabled().await.unwrap_or(false);
    let url = get_proxy_url().await.unwrap_or_default();
    let username = get_proxy_username().await.unwrap_or_default();
    let password = get_proxy_password().await.unwrap_or_default();
    let no_proxy = get_proxy_no_proxy().await.unwrap_or_default();
    let ignore_ssl_certificates = is_proxy_ignore_ssl_certificates().await.unwrap_or(false);
    let proxy_ssl = is_proxy_ssl().await.unwrap_or(false);
    let proxy_host_ssl = is_proxy_host_ssl().await.unwrap_or(false);
    let peer_ssl = is_peer_ssl().await.unwrap_or(false);
    let host_ssl = is_host_ssl().await.unwrap_or(false);

    Ok(Json(ProxySettingsResponse {
        enabled,
        url,
        username,
        password,
        no_proxy,
        ignore_ssl_certificates,
        proxy_ssl,
        proxy_host_ssl,
        peer_ssl,
        host_ssl,
    }))
}

// Admin endpoint to update proxy settings
pub async fn update_proxy_settings(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateProxySettingsRequest>,
) -> Result<Json<ProxySettingsResponse>, StatusCode> {
    // Update all proxy settings
    if let Err(_) = set_proxy_enabled(request.enabled).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_url(&request.url).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_username(&request.username).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_password(&request.password).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_no_proxy(&request.no_proxy).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_ignore_ssl_certificates(request.ignore_ssl_certificates).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_ssl(request.proxy_ssl).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_proxy_host_ssl(request.proxy_host_ssl).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_peer_ssl(request.peer_ssl).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(_) = set_host_ssl(request.host_ssl).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(ProxySettingsResponse {
        enabled: request.enabled,
        url: request.url,
        username: request.username,
        password: request.password,
        no_proxy: request.no_proxy,
        ignore_ssl_certificates: request.ignore_ssl_certificates,
        proxy_ssl: request.proxy_ssl,
        proxy_host_ssl: request.proxy_host_ssl,
        peer_ssl: request.peer_ssl,
        host_ssl: request.host_ssl,
    }))
}

// Public endpoint to test proxy connection (no authentication required)
pub async fn test_proxy_connection_public(
    Json(request): Json<TestProxyConnectionRequest>,
) -> Result<Json<TestProxyConnectionResponse>, StatusCode> {
    // Allow testing proxy even when not enabled - only check URL is provided

    // Validate URL is provided
    if request.url.trim().is_empty() {
        return Ok(Json(TestProxyConnectionResponse {
            success: false,
            message: "Proxy URL is required".to_string(),
        }));
    }

    // Test the proxy connection by making a simple HTTP request through the proxy
    match test_proxy_connectivity(&request).await {
        Ok(()) => Ok(Json(TestProxyConnectionResponse {
            success: true,
            message: "Proxy connection successful".to_string(),
        })),
        Err(e) => Ok(Json(TestProxyConnectionResponse {
            success: false,
            message: format!("Proxy connection failed: {}", e),
        })),
    }
}

async fn test_proxy_connectivity(proxy_config: &TestProxyConnectionRequest) -> Result<(), String> {
    // Always test the proxy configuration regardless of enabled status
    // This allows users to test settings before enabling them

    // Use the common proxy testing utility
    let common_config = crate::database::models::ProxySettings::from(proxy_config);
    crate::utils::proxy::test_proxy_connectivity(&common_config).await
}
