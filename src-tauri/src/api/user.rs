use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{ResetPasswordRequest, UpdateUserRequest},
    queries::users,
};
use crate::utils::password;

#[derive(Deserialize)]
pub struct UserHello {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

pub async fn greet(Json(payload): Json<UserHello>) -> (StatusCode, String) {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string());
    }
    // Return a greeting message
    (
        StatusCode::OK,
        serde_json::to_string(&format!("Hello, {}!", name))
            .unwrap_or_else(|_| "\"Hello, World!\"".to_string()),
    )
}

// List users with pagination
pub async fn list_users(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<crate::database::models::UserListResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match users::list_users(page, per_page).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error listing users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get user by ID
pub async fn get_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::database::models::User>, StatusCode> {
    match users::get_user_by_id(user_id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Update user
pub async fn update_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<crate::database::models::User>, StatusCode> {
    match users::update_user(
        user_id,
        request.username,
        request.email,
        request.is_active,
        request.profile,
    )
    .await
    {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Reset user password
pub async fn reset_user_password(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    // Hash the password with random salt
    let password_service = match password::hash_password(&request.new_password) {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match users::reset_user_password_with_service(request.user_id, password_service).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error resetting user password: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Toggle user active status
pub async fn toggle_user_active(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match users::toggle_user_active(user_id).await {
        Ok(is_active) => Ok(Json(serde_json::json!({ "is_active": is_active }))),
        Err(e) => {
            eprintln!("Error toggling user active status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create a new user (admin only)
pub async fn create_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<crate::database::models::CreateUserRequest>,
) -> Result<Json<crate::database::models::User>, StatusCode> {
    // Hash the password with random salt
    let password_service = match password::hash_password(&request.password) {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            eprintln!("Error creating user: {}", e);
            if e.to_string().contains("duplicate key") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

// Delete a user (admin only)
pub async fn delete_user(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match users::delete_user(user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
