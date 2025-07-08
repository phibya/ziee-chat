use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::{AuthenticatedUser, check_permission};
use crate::database::{
    models::{CreateUserGroupRequest, UpdateUserGroupRequest, AssignUserToGroupRequest},
    queries::user_groups,
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

// Create user group
pub async fn create_user_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUserGroupRequest>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::create_user_group(
        request.name,
        request.description,
        request.permissions,
    )
    .await
    {
        Ok(group) => Ok(Json(group)),
        Err(e) => {
            eprintln!("Error creating user group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get user group by ID
pub async fn get_user_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::get_user_group_by_id(group_id).await {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting user group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// List user groups with pagination
pub async fn list_user_groups(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<crate::database::models::UserGroupListResponse>, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::list_user_groups(page, per_page).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error listing user groups: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Update user group
pub async fn update_user_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateUserGroupRequest>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::update_user_group(
        group_id,
        request.name,
        request.description,
        request.permissions,
        request.is_active,
    )
    .await
    {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => Err(StatusCode::FORBIDDEN),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// Delete user group
pub async fn delete_user_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::delete_user_group(group_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => Err(StatusCode::FORBIDDEN),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// Assign user to group
pub async fn assign_user_to_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignUserToGroupRequest>,
) -> Result<StatusCode, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::assign_user_to_group(request.user_id, request.group_id, None).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Error assigning user to group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Remove user from group
pub async fn remove_user_from_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((user_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    match user_groups::remove_user_from_group(user_id, group_id).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error removing user from group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get group members
pub async fn get_group_members(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<crate::database::models::UserListResponse>, StatusCode> {
    // Check group_management permission
    if !check_permission(&auth_user.user, "group_management") {
        return Err(StatusCode::FORBIDDEN);
    }

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::get_group_members(group_id, page, per_page).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error getting group members: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}