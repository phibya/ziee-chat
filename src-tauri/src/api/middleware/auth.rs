use crate::auth::AuthService;
use crate::database::models::User;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    // UNIFIED: Always validate JWT token for both desktop and web
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

/// Basic authenticated middleware that only checks for valid authentication
pub async fn authenticated_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Just ensure user is authenticated - no specific permission required
    let _ = get_authenticated_user(&req)?;
    Ok(next.run(req).await)
}
