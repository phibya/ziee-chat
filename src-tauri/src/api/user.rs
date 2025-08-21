use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult2, AppError},
    middleware::AuthenticatedUser,
};
use crate::database::{
    models::{ResetPasswordRequest, UpdateUserRequest},
    queries::users,
};
use crate::types::PaginationQuery;
use crate::utils::password;

#[derive(Deserialize, JsonSchema)]
pub struct UserHello {
    name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct UserActiveStatusResponse {
    pub is_active: bool,
}

#[debug_handler]
pub async fn greet(Json(payload): Json<UserHello>) -> ApiResult2<Json<String>> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            AppError::new(
                crate::api::errors::ErrorCode::ValidMissingRequiredField,
                "Name cannot be empty",
            ),
        ));
    }
    // Return a greeting message
    let greeting = format!("Hello, {}!", name);
    Ok((StatusCode::OK, Json(greeting)))
}

// List users with pagination
#[debug_handler]
pub async fn list_users(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult2<Json<crate::database::models::UserListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match users::list_users(page, per_page).await {
        Ok(mut response) => {
            // Sanitize users in the response
            response.users = response
                .users
                .into_iter()
                .map(|user| user.sanitized())
                .collect();
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            eprintln!("Error listing users: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

// Get user by ID
#[debug_handler]
pub async fn get_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> ApiResult2<Json<crate::database::models::User>> {
    match users::get_user_by_id(user_id).await {
        Ok(Some(user)) => Ok((StatusCode::OK, Json(user.sanitized()))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User"))),
        Err(e) => {
            eprintln!("Error getting user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

// Update user
#[debug_handler]
pub async fn update_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> ApiResult2<Json<crate::database::models::User>> {
    match users::update_user(
        user_id,
        request.username,
        request.email,
        request.is_active,
        request.profile,
    )
    .await
    {
        Ok(Some(user)) => Ok((StatusCode::OK, Json(user.sanitized()))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User"))),
        Err(e) => {
            eprintln!("Error updating user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

// Reset user password
#[debug_handler]
pub async fn reset_user_password(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ResetPasswordRequest>,
) -> ApiResult2<StatusCode> {
    // Hash the password with random salt
    let password_service = match password::hash_password(&request.new_password) {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Password hashing failed"),
            ));
        }
    };

    match users::reset_user_password_with_service(request.user_id, password_service).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("User"))),
        Err(e) => {
            eprintln!("Error resetting user password: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

// Toggle user active status
#[debug_handler]
pub async fn toggle_user_active(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> ApiResult2<Json<UserActiveStatusResponse>> {
    match users::toggle_user_active(user_id).await {
        Ok(is_active) => Ok((StatusCode::OK, Json(UserActiveStatusResponse { is_active }))),
        Err(e) => {
            eprintln!("Error toggling user active status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}

// Create a new user (admin only)
#[debug_handler]
pub async fn create_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<crate::database::models::CreateUserRequest>,
) -> ApiResult2<Json<crate::database::models::User>> {
    // Hash the password with random salt
    let password_service = match password::hash_password(&request.password) {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Password hashing failed"),
            ));
        }
    };

    match users::create_user_with_password_service(
        request.username.clone(),
        request.email.clone(),
        Some(password_service),
        None,
    )
    .await
    {
        Ok(user) => Ok((StatusCode::OK, Json(user.sanitized()))),
        Err(e) => {
            eprintln!("Error creating user: {}", e);
            if e.to_string().contains("duplicate key") {
                Err((
                    StatusCode::CONFLICT,
                    AppError::conflict("User already exists"),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Database error"),
                ))
            }
        }
    }
}

// Delete a user (admin only)
#[debug_handler]
pub async fn delete_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    match users::delete_user(user_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("User"))),
        Err(e) => {
            eprintln!("Error deleting user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Database error"),
            ))
        }
    }
}
