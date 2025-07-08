use axum::{http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};
use crate::api::middleware::{AuthenticatedUser, check_permission};
use crate::database::queries::configuration::{
    is_user_registration_enabled, set_user_registration_enabled,
};

#[derive(Serialize)]
pub struct UserRegistrationStatusResponse {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserRegistrationRequest {
    pub enabled: bool,
}

// Public endpoint to check registration status (no auth required)
pub async fn get_user_registration_status() -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok(Json(UserRegistrationStatusResponse { enabled })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to check registration status
pub async fn get_user_registration_status_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    // Check user_management permission
    if !check_permission(&auth_user.user, "user_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match is_user_registration_enabled().await {
        Ok(enabled) => Ok(Json(UserRegistrationStatusResponse { enabled })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to update registration status
pub async fn update_user_registration_status(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateUserRegistrationRequest>,
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    // Check user_management permission
    if !check_permission(&auth_user.user, "user_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match set_user_registration_enabled(request.enabled).await {
        Ok(_) => Ok(Json(UserRegistrationStatusResponse {
            enabled: request.enabled,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}