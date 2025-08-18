use axum::{
    debug_handler,
    extract::Request,
    http::{header, StatusCode},
    response::Json,
};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::app::is_desktop_app;
use crate::api::errors::{ApiResult, ApiResult2, AppError, ErrorCode};
use crate::auth::AuthService;
use crate::database::models::*;
use crate::database::queries::users;

static AUTH_SERVICE: Lazy<AuthService> = Lazy::new(|| AuthService::default());

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitResponse {
    pub needs_setup: bool,
    pub is_desktop: bool,
    pub token: Option<String>, //if desktop app, include token for auto-login
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

async fn create_root_user() -> ApiResult<String> {
    let user_request = CreateUserRequest {
        username: "root".to_string(),
        email: "root@domain.com".to_string(),
        password: "root".to_string(),
        profile: Some(serde_json::json!({})),
    };

    match AUTH_SERVICE.create_user(user_request).await {
        Ok(user) => {
            // Assign root user to admin group
            if let Err(e) =
                crate::database::queries::user_groups::assign_user_to_admin_group(user.id).await
            {
                eprintln!("Warning: Failed to assign root user to admin group: {}", e);
            }

            // Generate token for the new root user
            match AUTH_SERVICE.generate_token(&user) {
                Ok(token) => {
                    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24 * 7);

                    // Add login token to database
                    let login_token = AUTH_SERVICE.generate_login_token();
                    let when_created = chrono::Utc::now().timestamp_millis();

                    if let Err(e) =
                        users::add_login_token(user.id, login_token, when_created, Some(expires_at))
                            .await
                    {
                        return Err(AppError::from_error(ErrorCode::AuthTokenStorageFailed, e));
                    }

                    // Mark app as initialized
                    if let Err(e) =
                        crate::database::queries::configuration::mark_app_initialized().await
                    {
                        eprintln!("Warning: Failed to mark app as initialized: {}", e);
                    }

                    Ok(token)
                }
                Err(e) => Err(AppError::from_error(
                    ErrorCode::AuthTokenGenerationFailed,
                    e,
                )),
            }
        }
        Err(e) => Err(AppError::from_string(ErrorCode::UserRootCreationFailed, e)),
    }
}

/// Check if the app needs initial setup (not initialized)
#[debug_handler]
pub async fn check_init_status() -> ApiResult2<Json<InitResponse>> {
    let needs_setup = match crate::database::queries::configuration::is_app_initialized().await {
        Ok(is_initialized) => !is_initialized,
        Err(_) => true,
    };

    if is_desktop_app() {
        if needs_setup {
            match create_root_user().await {
                Ok(token) => {
                    return Ok((
                        StatusCode::OK,
                        Json(InitResponse {
                            needs_setup: false,
                            is_desktop: true,
                            token: Some(token),
                        }),
                    ));
                }
                Err(e) => return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::from_string(ErrorCode::UserRootCreationFailed, e.to_string()),
                  )),
            }
        } else {
            return Ok((
                StatusCode::OK,
                Json(InitResponse {
                    needs_setup: false,
                    is_desktop: true,
                    token: None,
                }),
            ));
        }
    }

    Ok((
        StatusCode::OK,
        Json(InitResponse {
            needs_setup,
            is_desktop: false,
            token: None, // No token for web app setup
        }),
    ))
}

/// Initialize the app with root user (for web app)
#[debug_handler]
pub async fn init_app(Json(payload): Json<CreateUserRequest>) -> ApiResult2<Json<AuthResponse>> {
    // Check if app is already initialized
    if let Ok(true) = crate::database::queries::configuration::is_app_initialized().await {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::app_already_initialized()
          ));
    }

    let mut root_request = payload;
    root_request.profile = Some(serde_json::json!({}));

    match AUTH_SERVICE.create_user(root_request).await {
        Ok(user) => {
            // Assign root user to admin group
            if let Err(e) =
                crate::database::queries::user_groups::assign_user_to_admin_group(user.id).await
            {
                eprintln!("Warning: Failed to assign root user to admin group: {}", e);
            }

            // Generate token for the new root user
            match AUTH_SERVICE.generate_token(&user) {
                Ok(token) => {
                    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24 * 7);

                    // Add login token to database
                    let login_token = AUTH_SERVICE.generate_login_token();
                    let when_created = chrono::Utc::now().timestamp_millis();

                    if let Err(e) =
                        users::add_login_token(user.id, login_token, when_created, Some(expires_at))
                            .await
                    {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            AppError::from_error(ErrorCode::AuthTokenStorageFailed, e)
                          ));
                    }

                    // Mark app as initialized
                    if let Err(e) =
                        crate::database::queries::configuration::mark_app_initialized().await
                    {
                        eprintln!("Warning: Failed to mark app as initialized: {}", e);
                    }

                    Ok((
                        StatusCode::OK,
                        Json(AuthResponse {
                            token,
                            user: user.sanitized(),
                            expires_at,
                        })
                      ))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::from_error(
                        ErrorCode::AuthTokenGenerationFailed,
                        e,
                    )
                  )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from_string(ErrorCode::UserRootCreationFailed, e.to_string()),
          )),
    }
}

/// Login endpoint
#[debug_handler]
pub async fn login(Json(payload): Json<LoginRequest>) -> ApiResult2<Json<AuthResponse>> {
    // For web app, authenticate with credentials
    match AUTH_SERVICE
        .authenticate_user(&payload.username_or_email, &payload.password)
        .await
    {
        Ok(Some(login_response)) => Ok((
            StatusCode::OK,
            Json(AuthResponse {
                token: login_response.token,
                user: login_response.user.sanitized(),
                expires_at: login_response.expires_at,
            })
          )),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            AppError::invalid_credentials()
          )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from_string(ErrorCode::AuthenticationFailed, e)
          )),
    }
}

/// Logout endpoint
#[debug_handler]
pub async fn logout(req: Request) -> ApiResult2<StatusCode> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let Some(token) = auth_header else {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::missing_auth_header()
          ));
    };

    // For web app, remove the login token
    if let Err(e) = AUTH_SERVICE.logout_user(token).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from_string(ErrorCode::AuthLogoutFailed, e)
          ));
    }

    Ok((
        StatusCode::OK,
        StatusCode::OK
      ))
}

/// Get current user endpoint
#[debug_handler]
pub async fn me(
    axum::Extension(auth_user): axum::Extension<crate::api::middleware::AuthenticatedUser>,
) -> ApiResult2<Json<User>> {
    Ok((
        StatusCode::OK,
        Json(auth_user.user.sanitized())
      ))
}

/// Register endpoint (for web app only)
#[debug_handler]
pub async fn register(Json(payload): Json<CreateUserRequest>) -> ApiResult2<Json<AuthResponse>> {
    // Desktop app doesn't support registration
    if is_desktop_app() {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::desktop_mode_restriction()
          ));
    }

    // Check if app is initialized
    if let Ok(false) = crate::database::queries::configuration::is_app_initialized().await {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::app_not_initialized()
          ));
    }

    // Check if user registration is enabled
    if let Ok(false) = crate::database::queries::configuration::is_user_registration_enabled().await
    {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::registration_disabled()
          ));
    }

    // Create new user
    match AUTH_SERVICE.create_user(payload).await {
        Ok(user) => {
            // Generate token for the new user
            match AUTH_SERVICE.generate_token(&user) {
                Ok(token) => {
                    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24 * 7);

                    // Add login token to database
                    let login_token = AUTH_SERVICE.generate_login_token();
                    let when_created = chrono::Utc::now().timestamp_millis();

                    if let Err(e) =
                        users::add_login_token(user.id, login_token, when_created, Some(expires_at))
                            .await
                    {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            AppError::from_error(ErrorCode::AuthTokenStorageFailed, e)
                          ));
                    }

                    Ok((
                        StatusCode::OK,
                        Json(AuthResponse {
                            token,
                            user: user.sanitized(),
                            expires_at,
                        })
                      ))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::from_error(
                        ErrorCode::AuthTokenGenerationFailed,
                        e,
                    )
                  )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            AppError::from_string(ErrorCode::UserCreationFailed, e)
          )),
    }
}
