use axum::{http::StatusCode, response::Json, Extension};
use serde::{Deserialize, Serialize};
use crate::api::middleware::AuthenticatedUser;
use crate::database::queries::configuration::{
    is_user_registration_enabled, set_user_registration_enabled,
    get_default_language, set_default_language,
};

#[derive(Serialize)]
pub struct UserRegistrationStatusResponse {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserRegistrationRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct DefaultLanguageResponse {
    pub language: String,
}

#[derive(Deserialize)]
pub struct UpdateDefaultLanguageRequest {
    pub language: String,
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
    match set_user_registration_enabled(request.enabled).await {
        Ok(_) => Ok(Json(UserRegistrationStatusResponse {
            enabled: request.enabled,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Public endpoint to get default language (no auth required)
pub async fn get_default_language_public() -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match get_default_language().await {
        Ok(language) => Ok(Json(DefaultLanguageResponse { language })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to get default language
pub async fn get_default_language_admin(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match get_default_language().await {
        Ok(language) => Ok(Json(DefaultLanguageResponse { language })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Admin endpoint to update default language
pub async fn update_default_language(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateDefaultLanguageRequest>,
) -> Result<Json<DefaultLanguageResponse>, StatusCode> {
    match set_default_language(&request.language).await {
        Ok(_) => Ok(Json(DefaultLanguageResponse {
            language: request.language,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}