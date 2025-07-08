use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::api::app::methods::is_desktop_app;
use crate::auth::AuthService;
use crate::database::models::User;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Authentication middleware that validates JWT token and adds user to request extensions
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
                req.extensions_mut().insert(AuthenticatedUser { user });
                return Ok(next.run(req).await);
            }
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    // For web app, validate JWT token
    match auth_service.get_user_by_token(token).await {
        Ok(Some(user)) => {
            req.extensions_mut().insert(AuthenticatedUser { user });
            Ok(next.run(req).await)
        }
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Check if the authenticated user has a specific permission
pub fn check_permission(user: &User, permission: &str) -> bool {
    for group in &user.groups {
        if !group.is_active {
            continue;
        }
        
        if let Some(permissions) = group.permissions.as_object() {
            if let Some(has_permission) = permissions.get(permission) {
                if has_permission.as_bool().unwrap_or(false) {
                    return true;
                }
            }
        }
    }
    false
}

/// Extract authenticated user from request extensions
pub fn get_authenticated_user(req: &Request) -> Result<&User, StatusCode> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|auth_user| &auth_user.user)
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Middleware that checks for user_management permission
pub async fn user_management_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    
    if !check_permission(user, "user_management") {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}

/// Middleware that checks for group_management permission
pub async fn group_management_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    
    if !check_permission(user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}

/// Middleware that checks for system_admin permission
pub async fn system_admin_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    
    if !check_permission(user, "system_admin") {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}