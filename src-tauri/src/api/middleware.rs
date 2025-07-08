use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::api::app::methods::is_desktop_app;
use crate::api::permissions::{check_permission, permissions};
use crate::auth::AuthService;
use crate::database::models::User;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Authentication middleware that validates JWT token and adds user to request extensions
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let Some(token) = auth_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let auth_service = AuthService::default();

    // For desktop app, get or create default admin user
    if is_desktop_app() {
        match auth_service.get_or_create_default_admin_user().await {
            Ok(user) => {
                req.extensions_mut().insert(AuthenticatedUser { user_id: user.id, user });
                return Ok(next.run(req).await);
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // For web app, validate JWT token
    match auth_service.get_user_by_token(token).await {
        Ok(Some(user)) => {
            req.extensions_mut().insert(AuthenticatedUser { user_id: user.id, user });
            Ok(next.run(req).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Check if the authenticated user has a specific permission
/// This function is now deprecated - use crate::api::permissions::check_permission instead
#[deprecated(note = "Use crate::api::permissions::check_permission instead")]
pub fn check_permission_legacy(user: &User, permission: &str) -> bool {
    check_permission(user, permission)
}

/// Extract authenticated user from request extensions
pub fn get_authenticated_user(req: &Request) -> Result<&User, StatusCode> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|auth_user| &auth_user.user)
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Middleware that checks for user_management permission (legacy - removed)
/// Use specific permission middleware instead
#[deprecated(note = "Use specific permission middleware instead")]
pub async fn user_management_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::USERS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for group_management permission (legacy - removed)
/// Use specific permission middleware instead
#[deprecated(note = "Use specific permission middleware instead")]
pub async fn group_management_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::GROUPS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for users::read permission
pub async fn users_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::USERS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for users::edit permission
pub async fn users_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::USERS_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for users::create permission
pub async fn users_create_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::USERS_CREATE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for users::delete permission
pub async fn users_delete_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::USERS_DELETE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for groups::read permission
pub async fn groups_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::GROUPS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for groups::edit permission
pub async fn groups_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::GROUPS_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for groups::create permission
pub async fn groups_create_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::GROUPS_CREATE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for groups::delete permission
pub async fn groups_delete_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::GROUPS_DELETE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::user-registration::read permission
pub async fn config_user_registration_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_USER_REGISTRATION_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::user-registration::edit permission
pub async fn config_user_registration_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_USER_REGISTRATION_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::updates::read permission
pub async fn config_updates_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_UPDATES_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::updates::edit permission
pub async fn config_updates_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_UPDATES_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::experimental::read permission
pub async fn config_experimental_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_EXPERIMENTAL_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::experimental::edit permission
pub async fn config_experimental_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_EXPERIMENTAL_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::data-folder::read permission
pub async fn config_data_folder_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_DATA_FOLDER_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::data-folder::edit permission
pub async fn config_data_folder_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_DATA_FOLDER_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::factory-reset::read permission
pub async fn config_factory_reset_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_FACTORY_RESET_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::factory-reset::edit permission
pub async fn config_factory_reset_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_FACTORY_RESET_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for settings::read permission
pub async fn settings_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::SETTINGS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for settings::edit permission
pub async fn settings_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::SETTINGS_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for settings::delete permission
pub async fn settings_delete_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::SETTINGS_DELETE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
