use crate::api::middleware::auth::get_authenticated_user;
use crate::api::permissions::{check_permission, permissions};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

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
pub async fn providers_read_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::edit permission
pub async fn providers_edit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::create permission
pub async fn providers_create_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::PROVIDERS_CREATE) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::providers::delete permission
pub async fn providers_delete_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
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

/// Middleware that checks for config::repositories::read permission
pub async fn repositories_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    if !check_permission(user, permissions::REPOSITORIES_READ) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

/// Middleware that checks for config::repositories::edit permission
pub async fn repositories_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    if !check_permission(user, permissions::REPOSITORIES_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

/// Middleware that checks for config::repositories::create permission
pub async fn repositories_create_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    if !check_permission(user, permissions::REPOSITORIES_CREATE) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

/// Middleware that checks for config::repositories::delete permission
pub async fn repositories_delete_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;
    if !check_permission(user, permissions::REPOSITORIES_DELETE) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

/// Middleware that checks for config::ngrok::read permission
pub async fn config_ngrok_read_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_NGROK_READ) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// Middleware that checks for config::ngrok::edit permission
pub async fn config_ngrok_edit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = get_authenticated_user(&req)?;

    if !check_permission(user, permissions::CONFIG_NGROK_EDIT) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}