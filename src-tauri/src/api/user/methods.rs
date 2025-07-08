use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{
    models::{UpdateUserRequest, ResetPasswordRequest},
    queries::users,
};

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
    Json(request): Json<ResetPasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    // Hash the password
    let password_hash = match bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Error hashing password: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match users::reset_user_password(request.user_id, password_hash).await {
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
