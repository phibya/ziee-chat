use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::api::app::is_desktop_app;
use crate::api::permissions::{check_permission, permissions};
use crate::auth::AuthService;
use crate::database::models::User;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub user: User,
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
                req.extensions_mut().insert(AuthenticatedUser {
                    user_id: user.id,
                    user,
                });
                return Ok(next.run(req).await);
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // For web app, validate JWT token
    match auth_service.get_user_by_token(token).await {
        Ok(Some(user)) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: user.id,
                user,
            });
            Ok(next.run(req).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Extract authenticated user from request extensions
pub fn get_authenticated_user(req: &Request) -> Result<&User, StatusCode> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|auth_user| &auth_user.user)
        .ok_or(StatusCode::UNAUTHORIZED)
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
pub async fn config_user_registration_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_USER_REGISTRATION_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::user-registration::edit permission
pub async fn config_user_registration_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_USER_REGISTRATION_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::appearance::read permission
pub async fn config_appearance_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_APPEARANCE_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::appearance::edit permission
pub async fn config_appearance_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_APPEARANCE_EDIT) {
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

/// Middleware that checks for config::providers::read permission
pub async fn providers_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::edit permission
pub async fn providers_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::create permission
pub async fn providers_create_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_CREATE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::delete permission
pub async fn providers_delete_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_DELETE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::proxy::read permission
pub async fn config_proxy_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_PROXY_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::proxy::edit permission
pub async fn config_proxy_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_PROXY_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
