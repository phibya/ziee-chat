use crate::api::app::is_desktop_app;
use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::auth::AuthService;
use crate::database::queries::configuration::{
    get_default_language, get_ngrok_settings, get_proxy_no_proxy, get_proxy_password,
    get_proxy_url, get_proxy_username, is_proxy_enabled, is_proxy_ignore_ssl_certificates,
    is_user_registration_enabled, set_default_language, set_ngrok_settings, set_proxy_enabled,
    set_proxy_ignore_ssl_certificates, set_proxy_no_proxy, set_proxy_password, set_proxy_url,
    set_proxy_username, set_user_registration_enabled, NgrokSettings,
};
use crate::utils::ngrok::NgrokService;
use aide::axum::IntoApiResponse;
use axum::{debug_handler, http::StatusCode, response::Json, Extension};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Global ngrok service instance
static NGROK_SERVICE: Lazy<Arc<Mutex<Option<NgrokService>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Serialize, JsonSchema)]
pub struct UserRegistrationStatusResponse {
    pub enabled: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateUserRegistrationRequest {
    pub enabled: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct DefaultLanguageResponse {
    pub language: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateDefaultLanguageRequest {
    pub language: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ProxySettingsResponse {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    // pub proxy_ssl: bool,
    // pub proxy_host_ssl: bool,
    // pub peer_ssl: bool,
    // pub host_ssl: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateProxySettingsRequest {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    // pub proxy_ssl: bool,
    // pub proxy_host_ssl: bool,
    // pub peer_ssl: bool,
    // pub host_ssl: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TestProxyConnectionRequest {
    pub enabled: bool,
    pub url: String,
    pub username: String,
    pub password: String,
    pub no_proxy: String,
    pub ignore_ssl_certificates: bool,
    // pub proxy_ssl: bool,
    // pub proxy_host_ssl: bool,
    // pub peer_ssl: bool,
    // pub host_ssl: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TestProxyConnectionResponse {
    pub success: bool,
    pub message: String,
}

// Ngrok API types
#[derive(Serialize, JsonSchema)]
pub struct NgrokSettingsResponse {
    pub api_key: String, // Will be empty in response for security
    pub tunnel_enabled: bool,
    pub tunnel_url: Option<String>,
    pub tunnel_status: String,
    pub auto_start: bool,
    pub domain: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateNgrokSettingsRequest {
    pub api_key: Option<String>,
    pub tunnel_enabled: Option<bool>,
    pub auto_start: Option<bool>,
    pub domain: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct NgrokStatusResponse {
    pub tunnel_active: bool,
    pub tunnel_url: Option<String>,
    pub tunnel_status: String,
    pub last_error: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateUserPasswordRequest {
    pub current_password: Option<String>, // Optional for desktop apps
    pub new_password: String,
}

// Public endpoint to check registration status (no auth required)
#[debug_handler]
pub async fn get_user_registration_status() -> ApiResult<Json<UserRegistrationStatusResponse>> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok((
            StatusCode::OK,
            Json(UserRegistrationStatusResponse { enabled }),
        )),
        Err(e) => {
            eprintln!("Error getting user registration status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Admin endpoint to check registration status
#[debug_handler]
pub async fn get_user_registration_status_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<UserRegistrationStatusResponse>> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok((
            StatusCode::OK,
            Json(UserRegistrationStatusResponse { enabled }),
        )),
        Err(e) => {
            eprintln!("Error getting user registration status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user registration status"),
            ))
        }
    }
}

/// Admin endpoint to update registration status
#[debug_handler]
pub async fn update_user_registration_status(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateUserRegistrationRequest>,
) -> ApiResult<Json<UserRegistrationStatusResponse>> {
    match set_user_registration_enabled(request.enabled).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(UserRegistrationStatusResponse {
                enabled: request.enabled,
            }),
        )),
        Err(e) => {
            eprintln!("Error updating user registration status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update user registration status"),
            ))
        }
    }
}

// Public endpoint to get default language (no auth required)
#[debug_handler]
pub async fn get_default_language_public() -> ApiResult<Json<DefaultLanguageResponse>> {
    match get_default_language().await {
        Ok(language) => Ok((StatusCode::OK, Json(DefaultLanguageResponse { language }))),
        Err(e) => {
            eprintln!("Error getting default language: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

/// Admin endpoint to get default language
#[debug_handler]
pub async fn get_default_language_admin(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<DefaultLanguageResponse>> {
    match get_default_language().await {
        Ok(language) => Ok((StatusCode::OK, Json(DefaultLanguageResponse { language }))),
        Err(e) => {
            eprintln!("Error getting default language: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get default language"),
            ))
        }
    }
}

/// Admin endpoint to update default language
#[debug_handler]
pub async fn update_default_language(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateDefaultLanguageRequest>,
) -> ApiResult<Json<DefaultLanguageResponse>> {
    match set_default_language(&request.language).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(DefaultLanguageResponse {
                language: request.language,
            }),
        )),
        Err(e) => {
            eprintln!("Error updating default language: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update default language"),
            ))
        }
    }
}

/// Admin endpoint to get proxy settings
#[debug_handler]
pub async fn get_proxy_settings(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<ProxySettingsResponse>> {
    let enabled = is_proxy_enabled().await.unwrap_or(false);
    let url = get_proxy_url().await.unwrap_or_default();
    let username = get_proxy_username().await.unwrap_or_default();
    let password = get_proxy_password().await.unwrap_or_default();
    let no_proxy = get_proxy_no_proxy().await.unwrap_or_default();
    let ignore_ssl_certificates = is_proxy_ignore_ssl_certificates().await.unwrap_or(false);
    // let proxy_ssl = is_proxy_ssl().await.unwrap_or(false);
    // let proxy_host_ssl = is_proxy_host_ssl().await.unwrap_or(false);
    // let peer_ssl = is_peer_ssl().await.unwrap_or(false);
    // let host_ssl = is_host_ssl().await.unwrap_or(false);

    Ok((
        StatusCode::OK,
        Json(ProxySettingsResponse {
            enabled,
            url,
            username,
            password,
            no_proxy,
            ignore_ssl_certificates,
            // proxy_ssl,
            // proxy_host_ssl,
            // peer_ssl,
            // host_ssl,
        }),
    ))
}

/// Admin endpoint to update proxy settings
#[debug_handler]
pub async fn update_proxy_settings(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateProxySettingsRequest>,
) -> ApiResult<Json<ProxySettingsResponse>> {
    // Update all proxy settings
    if let Err(e) = set_proxy_enabled(request.enabled).await {
        eprintln!("Error setting proxy enabled: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    if let Err(e) = set_proxy_url(&request.url).await {
        eprintln!("Error setting proxy URL: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    if let Err(e) = set_proxy_username(&request.username).await {
        eprintln!("Error setting proxy username: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    if let Err(e) = set_proxy_password(&request.password).await {
        eprintln!("Error setting proxy password: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    if let Err(e) = set_proxy_no_proxy(&request.no_proxy).await {
        eprintln!("Error setting proxy no_proxy: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    if let Err(e) = set_proxy_ignore_ssl_certificates(request.ignore_ssl_certificates).await {
        eprintln!("Error setting proxy ignore SSL certificates: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update proxy settings"),
        ));
    }
    // if let Err(_) = set_proxy_ssl(request.proxy_ssl).await {
    //     return Err(StatusCode::INTERNAL_SERVER_ERROR);
    // }
    // if let Err(_) = set_proxy_host_ssl(request.proxy_host_ssl).await {
    //     return Err(StatusCode::INTERNAL_SERVER_ERROR);
    // }
    // if let Err(_) = set_peer_ssl(request.peer_ssl).await {
    //     return Err(StatusCode::INTERNAL_SERVER_ERROR);
    // }
    // if let Err(_) = set_host_ssl(request.host_ssl).await {
    //     return Err(StatusCode::INTERNAL_SERVER_ERROR);
    // }

    Ok((
        StatusCode::OK,
        Json(ProxySettingsResponse {
            enabled: request.enabled,
            url: request.url,
            username: request.username,
            password: request.password,
            no_proxy: request.no_proxy,
            ignore_ssl_certificates: request.ignore_ssl_certificates,
            // proxy_ssl: request.proxy_ssl,
            // proxy_host_ssl: request.proxy_host_ssl,
            // peer_ssl: request.peer_ssl,
            // host_ssl: request.host_ssl,
        }),
    ))
}

// Public endpoint to test proxy connection (no authentication required)
pub async fn test_proxy_connection(
    Json(request): Json<TestProxyConnectionRequest>,
) -> impl IntoApiResponse {
    // Allow testing proxy even when not enabled - only check URL is provided

    // Validate URL is provided
    if request.url.trim().is_empty() {
        return (
            StatusCode::OK,
            Json(TestProxyConnectionResponse {
                success: false,
                message: "Proxy URL is required".to_string(),
            }),
        );
    }

    // Test the proxy connection by making a simple HTTP request through the proxy
    match test_proxy_connectivity(&request).await {
        Ok(()) => (
            StatusCode::OK,
            Json(TestProxyConnectionResponse {
                success: true,
                message: "Proxy connection successful".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(TestProxyConnectionResponse {
                success: false,
                message: format!("Proxy connection failed: {}", e),
            }),
        ),
    }
}

async fn test_proxy_connectivity(proxy_config: &TestProxyConnectionRequest) -> Result<(), String> {
    // Always test the proxy configuration regardless of enabled status
    // This allows users to test settings before enabling them

    // Use the common proxy testing utility
    let common_config = crate::database::models::ProxySettings::from(proxy_config);
    crate::utils::proxy::test_proxy_connectivity(&common_config).await
}

// Ngrok API handlers

#[debug_handler]
pub async fn get_ngrok_settings_handler(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<NgrokSettingsResponse>> {
    match get_ngrok_settings().await {
        Ok(settings) => Ok((
            StatusCode::OK,
            Json(NgrokSettingsResponse {
                api_key: settings.api_key,
                tunnel_enabled: settings.tunnel_enabled,
                tunnel_url: settings.tunnel_url,
                tunnel_status: settings.tunnel_status,
                auto_start: settings.auto_start,
                domain: settings.domain,
            }),
        )),
        Err(e) => {
            eprintln!("Error getting ngrok settings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get ngrok settings"),
            ))
        }
    }
}

#[debug_handler]
pub async fn update_ngrok_settings(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateNgrokSettingsRequest>,
) -> ApiResult<Json<NgrokSettingsResponse>> {
    // Get current settings
    let mut settings = match get_ngrok_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error getting current ngrok settings: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get current ngrok settings"),
            ));
        }
    };

    // Update fields if provided
    if let Some(api_key) = payload.api_key {
        if !api_key.is_empty() {
            settings.api_key = api_key;
        }
    }

    if let Some(tunnel_enabled) = payload.tunnel_enabled {
        settings.tunnel_enabled = tunnel_enabled;
    }

    if let Some(auto_start) = payload.auto_start {
        settings.auto_start = auto_start;
    }

    if let Some(domain) = payload.domain {
        settings.domain = if domain.is_empty() {
            None
        } else {
            Some(domain)
        };
    }

    // Save updated settings
    match set_ngrok_settings(&settings).await {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(NgrokSettingsResponse {
                api_key: settings.api_key,
                tunnel_enabled: settings.tunnel_enabled,
                tunnel_url: settings.tunnel_url,
                tunnel_status: settings.tunnel_status,
                auto_start: settings.auto_start,
                domain: settings.domain,
            }),
        )),
        Err(e) => {
            eprintln!("Error updating ngrok settings: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update ngrok settings"),
            ))
        }
    }
}

#[debug_handler]
pub async fn start_ngrok_tunnel(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<NgrokStatusResponse>> {
    // Get current settings
    let settings = match get_ngrok_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error getting ngrok settings: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get ngrok settings"),
            ));
        }
    };

    if settings.api_key.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidMissingRequiredField,
                "API key not configured",
            ),
        ));
    }

    // Get the HTTP port from the global config
    let local_port = *crate::HTTP_PORT;

    // Create and start ngrok service
    let mut ngrok_service = NgrokService::new(settings.api_key.clone());

    match ngrok_service
        .start_tunnel(local_port, settings.domain.clone())
        .await
    {
        Ok(tunnel_url) => {
            // Update settings with tunnel info
            let mut updated_settings = settings;
            updated_settings.tunnel_url = Some(tunnel_url.clone());
            updated_settings.tunnel_status = "active".to_string();

            if let Err(e) = set_ngrok_settings(&updated_settings).await {
                eprintln!("Error saving tunnel settings: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to save tunnel settings"),
                ));
            }

            // Store the service globally for later management
            {
                let mut global_service = NGROK_SERVICE.lock().await;
                *global_service = Some(ngrok_service);
            }

            Ok((
                StatusCode::OK,
                Json(NgrokStatusResponse {
                    tunnel_active: true,
                    tunnel_url: Some(tunnel_url),
                    tunnel_status: "active".to_string(),
                    last_error: None,
                }),
            ))
        }
        Err(e) => {
            // Update settings with error info
            let mut updated_settings = settings;
            updated_settings.tunnel_status = "error".to_string();

            let _ = set_ngrok_settings(&updated_settings).await;

            eprintln!("Error starting ngrok tunnel: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to start ngrok tunnel"),
            ))
        }
    }
}

#[debug_handler]
pub async fn stop_ngrok_tunnel(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<NgrokStatusResponse>> {
    // Stop the global ngrok service
    {
        let mut global_service = NGROK_SERVICE.lock().await;
        if let Some(mut service) = global_service.take() {
            if let Err(e) = service.stop_tunnel().await {
                eprintln!("Error stopping ngrok tunnel: {}", e);
                return Ok((
                    StatusCode::OK,
                    Json(NgrokStatusResponse {
                        tunnel_active: false,
                        tunnel_url: None,
                        tunnel_status: "error".to_string(),
                        last_error: Some(format!("Failed to stop tunnel: {}", e)),
                    }),
                ));
            }
        }
    }

    // Update settings
    let mut settings = match get_ngrok_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error getting ngrok settings: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get ngrok settings"),
            ));
        }
    };

    settings.tunnel_url = None;
    settings.tunnel_status = "inactive".to_string();

    if let Err(e) = set_ngrok_settings(&settings).await {
        eprintln!("Error updating ngrok settings: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::internal_error("Failed to update ngrok settings"),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(NgrokStatusResponse {
            tunnel_active: false,
            tunnel_url: None,
            tunnel_status: "inactive".to_string(),
            last_error: None,
        }),
    ))
}

#[debug_handler]
pub async fn get_ngrok_status(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<NgrokStatusResponse>> {
    // Check if service is running
    let tunnel_active = {
        let global_service = NGROK_SERVICE.lock().await;
        global_service
            .as_ref()
            .map_or(false, |service| service.is_tunnel_active())
    };

    // Get current settings
    let settings = match get_ngrok_settings().await {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error getting ngrok settings: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get ngrok settings"),
            ));
        }
    };

    Ok((
        StatusCode::OK,
        Json(NgrokStatusResponse {
            tunnel_active,
            tunnel_url: settings.tunnel_url,
            tunnel_status: if tunnel_active {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            last_error: None,
        }),
    ))
}

/// Try to autostart ngrok tunnel if configured
pub async fn try_autostart_ngrok_tunnel() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_desktop_app() {
        // Only autostart for desktop apps
        return Ok(());
    }

    // Get ngrok settings from database
    let settings = get_ngrok_settings()
        .await
        .map_err(|e| format!("Failed to get ngrok settings: {}", e))?;

    // Check if autostart is enabled
    if !settings.auto_start {
        tracing::info!("Ngrok tunnel autostart is disabled");
        return Ok(());
    }

    // Validate settings before attempting to start
    if let Err(e) = validate_ngrok_config(&settings).await {
        tracing::warn!("Ngrok autostart validation failed: {}", e);
        // Don't fail startup, just log the warning
        return Ok(());
    }

    // Check if tunnel is already running
    let tunnel_active = {
        let global_service = NGROK_SERVICE.lock().await;
        global_service
            .as_ref()
            .map_or(false, |service| service.is_tunnel_active())
    };

    if tunnel_active {
        tracing::info!("Ngrok tunnel is already running, skipping autostart");
        return Ok(());
    }

    // Get HTTP port for tunneling
    let http_port = crate::get_http_port();

    tracing::info!("Starting ngrok tunnel autostart on port {}", http_port);

    // Start the tunnel
    match start_ngrok_tunnel_internal(&settings.api_key, http_port, settings.domain.clone()).await {
        Ok(tunnel_url) => {
            tracing::info!("Ngrok tunnel autostart successful: {}", tunnel_url);

            // Update the settings with the tunnel URL
            let mut updated_settings = settings;
            updated_settings.tunnel_url = Some(tunnel_url);
            updated_settings.tunnel_status = "active".to_string();
            updated_settings.tunnel_enabled = true;

            if let Err(e) = set_ngrok_settings(&updated_settings).await {
                tracing::error!("Failed to save ngrok tunnel URL: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Ngrok tunnel autostart failed: {}", e);
            // Don't fail startup, just log the error
        }
    }

    Ok(())
}

/// Validate ngrok configuration
async fn validate_ngrok_config(
    settings: &NgrokSettings,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if API key is provided
    if settings.api_key.trim().is_empty() {
        return Err("Ngrok API key is required".into());
    }

    // Check if API key looks valid (basic format check)
    if settings.api_key.len() < 10 {
        return Err("Ngrok API key appears to be invalid".into());
    }

    Ok(())
}

/// Internal function to start ngrok tunnel
async fn start_ngrok_tunnel_internal(
    api_key: &str,
    local_port: u16,
    domain: Option<String>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use crate::utils::ngrok::NgrokService;

    // Create new ngrok service
    let mut service = NgrokService::new(api_key.to_string());

    // Start tunnel
    let tunnel_url = service
        .start_tunnel(local_port, domain)
        .await
        .map_err(|e| format!("Failed to start ngrok tunnel: {}", e))?;

    // Store service in global state
    {
        let mut global_service = NGROK_SERVICE.lock().await;
        *global_service = Some(service);
    }

    Ok(tunnel_url)
}

#[debug_handler]
pub async fn update_user_password(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateUserPasswordRequest>,
) -> ApiResult<StatusCode> {
    let auth_service = AuthService::default();

    // For desktop apps, skip current password verification
    if !is_desktop_app() {
        // Web apps: verify current password
        if let Some(current_password) = &payload.current_password {
            match auth_service
                .verify_user_password(&auth_user.user, current_password)
                .await
            {
                Ok(false) => {
                    return Err((StatusCode::UNAUTHORIZED, AppError::invalid_credentials()));
                }
                Err(e) => {
                    eprintln!("Error verifying current password: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Failed to verify current password"),
                    ));
                }
                _ => {}
            }
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                AppError::new(
                    crate::api::errors::ErrorCode::ValidMissingRequiredField,
                    "Current password is required for web apps",
                ),
            ));
        }
    }

    // Update to new password
    match auth_service
        .update_user_password(&auth_user.user.id, &payload.new_password)
        .await
    {
        Ok(_) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Err(e) => {
            eprintln!("Error updating user password: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to update user password"),
            ))
        }
    }
}
