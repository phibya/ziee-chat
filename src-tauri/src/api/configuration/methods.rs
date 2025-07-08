use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
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

// Admin endpoint to check registration status (for now, no auth check - matches existing admin endpoints)
pub async fn get_user_registration_status_admin() -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match is_user_registration_enabled().await {
        Ok(enabled) => Ok(Json(UserRegistrationStatusResponse { enabled })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to update registration status (for now, no auth check - matches existing admin endpoints)
pub async fn update_user_registration_status(
    Json(request): Json<UpdateUserRegistrationRequest>,
) -> Result<Json<UserRegistrationStatusResponse>, StatusCode> {
    match set_user_registration_enabled(request.enabled).await {
        Ok(_) => Ok(Json(UserRegistrationStatusResponse {
            enabled: request.enabled,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}