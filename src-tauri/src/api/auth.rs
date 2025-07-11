use axum::{
    debug_handler,
    extract::Request,
    http::{header, StatusCode},
    response::Json,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::api::app::is_desktop_app;
use crate::auth::AuthService;
use crate::database::models::*;
use crate::database::queries::users;

static AUTH_SERVICE: Lazy<AuthService> = Lazy::new(|| AuthService::default());

#[derive(Debug, Serialize, Deserialize)]
pub struct InitResponse {
    pub needs_setup: bool,
    pub is_desktop: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Check if the app needs initial setup (not initialized)
#[debug_handler]
pub async fn check_init_status() -> Result<Json<InitResponse>, (StatusCode, Json<ErrorResponse>)> {
    let needs_setup = match crate::database::queries::configuration::is_app_initialized().await {
        Ok(is_initialized) => !is_initialized,
        Err(_) => true,
    };

    Ok(Json(InitResponse {
        needs_setup,
        is_desktop: is_desktop_app(),
    }))
}

/// Initialize the app with root user (for web app)
#[debug_handler]
pub async fn init_app(
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if app is already initialized
    if let Ok(true) = crate::database::queries::configuration::is_app_initialized().await {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "App already initialized".to_string(),
            }),
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

                    if let Err(_) =
                        users::add_login_token(user.id, login_token, when_created, Some(expires_at))
                            .await
                    {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to store login token".to_string(),
                            }),
                        ));
                    }

                    // Mark app as initialized
                    if let Err(e) =
                        crate::database::queries::configuration::mark_app_initialized().await
                    {
                        eprintln!("Warning: Failed to mark app as initialized: {}", e);
                    }

                    Ok(Json(AuthResponse {
                        token,
                        user,
                        expires_at,
                    }))
                }
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to generate token".to_string(),
                    }),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to create root user: {}", e),
            }),
        )),
    }
}

/// Login endpoint
#[debug_handler]
pub async fn login(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // For desktop app, always auto-login with default admin
    if is_desktop_app() {
        match AUTH_SERVICE.auto_login_desktop().await {
            Ok(login_response) => {
                return Ok(Json(AuthResponse {
                    token: login_response.token,
                    user: login_response.user,
                    expires_at: login_response.expires_at,
                }));
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Desktop auto-login failed: {}", e),
                    }),
                ));
            }
        }
    }

    // For web app, authenticate with credentials
    match AUTH_SERVICE
        .authenticate_user(&payload.username_or_email, &payload.password)
        .await
    {
        Ok(Some(login_response)) => Ok(Json(AuthResponse {
            token: login_response.token,
            user: login_response.user,
            expires_at: login_response.expires_at,
        })),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Authentication failed: {}", e),
            }),
        )),
    }
}

/// Logout endpoint
#[debug_handler]
pub async fn logout(req: Request) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let Some(token) = auth_header else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Missing or invalid authorization header".to_string(),
            }),
        ));
    };

    // For desktop app, don't actually logout (just return success)
    if is_desktop_app() {
        return Ok(StatusCode::OK);
    }

    // For web app, remove the login token
    if let Err(e) = AUTH_SERVICE.logout_user(token).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Logout failed: {}", e),
            }),
        ));
    }

    Ok(StatusCode::OK)
}

/// Get current user endpoint
#[debug_handler]
pub async fn me(
    axum::Extension(auth_user): axum::Extension<crate::api::middleware::AuthenticatedUser>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(auth_user.user))
}

/// Register endpoint (for web app only)
#[debug_handler]
pub async fn register(
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Desktop app doesn't support registration
    if is_desktop_app() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Registration not supported in desktop mode".to_string(),
            }),
        ));
    }

    // Check if app is initialized
    if let Ok(false) = crate::database::queries::configuration::is_app_initialized().await {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "App not initialized. Please initialize the app first".to_string(),
            }),
        ));
    }

    // Check if user registration is enabled
    if let Ok(false) = crate::database::queries::configuration::is_user_registration_enabled().await
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "User registration is currently disabled".to_string(),
            }),
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

                    if let Err(_) =
                        users::add_login_token(user.id, login_token, when_created, Some(expires_at))
                            .await
                    {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to store login token".to_string(),
                            }),
                        ));
                    }

                    Ok(Json(AuthResponse {
                        token,
                        user,
                        expires_at,
                    }))
                }
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to generate token".to_string(),
                    }),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            }),
        )),
    }
}
