use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult2, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        AssignProviderToGroupRequest, AssignUserToGroupRequest, CreateUserGroupRequest,
        UpdateUserGroupRequest, UserGroupProviderResponse,
    },
    queries::{user_group_providers, user_groups},
};
use crate::types::PaginationQuery;


// Create user group
#[debug_handler]
pub async fn create_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUserGroupRequest>,
) -> ApiResult2<Json<crate::database::models::UserGroup>> {
    match user_groups::create_user_group(request.name, request.description, request.permissions)
        .await
    {
        Ok(group) => {
            // If provider_ids are provided, assign them to the group
            if let Some(provider_ids) = request.provider_ids {
                for provider_id in provider_ids {
                    let assign_request = AssignProviderToGroupRequest {
                        group_id: group.id,
                        provider_id,
                    };
                    if let Err(e) =
                        user_group_providers::assign_provider_to_group(assign_request).await
                    {
                        eprintln!("Error assigning model provider to group: {}", e);
                        // Continue with other providers even if one fails
                    }
                }
            }

            // Return the updated group with model provider IDs
            match user_groups::get_user_group_by_id(group.id).await {
                Ok(Some(updated_group)) => Ok((StatusCode::OK, Json(updated_group))),
                Ok(None) => Err((
                    StatusCode::NOT_FOUND,
                    AppError::not_found("User group")
                )),
                Err(e) => {
                    eprintln!("Error getting updated user group: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Failed to get updated user group")
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating user group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to create user group")
            ))
        }
    }
}

// Get user group by ID
#[debug_handler]
pub async fn get_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> ApiResult2<Json<crate::database::models::UserGroup>> {
    match user_groups::get_user_group_by_id(group_id).await {
        Ok(Some(group)) => Ok((StatusCode::OK, Json(group))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("User group")
        )),
        Err(e) => {
            eprintln!("Error getting user group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user group")
            ))
        }
    }
}

// List user groups with pagination
#[debug_handler]
pub async fn list_user_groups(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult2<Json<crate::database::models::UserGroupListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::list_user_groups(page, per_page).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error listing user groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to list user groups")
            ))
        }
    }
}

// Update user group
#[debug_handler]
pub async fn update_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateUserGroupRequest>,
) -> ApiResult2<Json<crate::database::models::UserGroup>> {
    // Handle model provider assignments if provided
    if let Some(provider_ids) = &request.provider_ids {
        // First, get current assignments
        let current_providers = user_group_providers::get_provider_ids_for_group(group_id)
            .await
            .unwrap_or_default();

        // Remove providers that are no longer in the list
        for current_provider in &current_providers {
            if !provider_ids.contains(current_provider) {
                if let Err(e) =
                    user_group_providers::remove_provider_from_group(group_id, *current_provider)
                        .await
                {
                    eprintln!("Error removing model provider from group: {}", e);
                }
            }
        }

        // Add new providers
        for provider_id in provider_ids {
            if !current_providers.contains(provider_id) {
                let assign_request = AssignProviderToGroupRequest {
                    group_id,
                    provider_id: *provider_id,
                };
                if let Err(e) = user_group_providers::assign_provider_to_group(assign_request).await
                {
                    eprintln!("Error assigning model provider to group: {}", e);
                }
            }
        }
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
        Ok(Some(group)) => Ok((StatusCode::OK, Json(group))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("User group")
        )),
        Err(e) => {
            eprintln!("Error updating user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => Err((
                    StatusCode::FORBIDDEN,
                    AppError::forbidden("Access denied")
                )),
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to update user group")
                )),
            }
        }
    }
}

// Delete user group
#[debug_handler]
pub async fn delete_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> ApiResult2<StatusCode> {
    match user_groups::delete_user_group(group_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("User group")
        )),
        Err(e) => {
            eprintln!("Error deleting user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => Err((
                    StatusCode::FORBIDDEN,
                    AppError::forbidden("Access denied")
                )),
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to delete user group")
                )),
            }
        }
    }
}

// Assign user to group
#[debug_handler]
pub async fn assign_user_to_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignUserToGroupRequest>,
) -> ApiResult2<StatusCode> {
    match user_groups::assign_user_to_group(request.user_id, request.group_id, None).await {
        Ok(()) => Ok((StatusCode::OK, StatusCode::OK)),
        Err(e) => {
            eprintln!("Error assigning user to group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to assign user to group")
            ))
        }
    }
}

// Remove user from group
#[debug_handler]
pub async fn remove_user_from_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((user_id, group_id)): Path<(Uuid, Uuid)>,
) -> ApiResult2<StatusCode> {
    match user_groups::remove_user_from_group(user_id, group_id).await {
        Ok(true) => Ok((StatusCode::OK, StatusCode::OK)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("User group membership")
        )),
        Err(e) => {
            eprintln!("Error removing user from group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to remove user from group")
            ))
        }
    }
}

// Get group members
#[debug_handler]
pub async fn get_group_members(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult2<Json<crate::database::models::UserListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::get_group_members(group_id, page, per_page).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error getting group members: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get group members")
            ))
        }
    }
}

// Get model providers for a group
#[debug_handler]
pub async fn get_group_providers(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> ApiResult2<Json<Vec<crate::database::models::Provider>>> {
    match user_group_providers::get_providers_for_group(group_id).await {
        Ok(providers) => Ok((StatusCode::OK, Json(providers))),
        Err(e) => {
            eprintln!("Error getting model providers for group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get model providers for group")
            ))
        }
    }
}

// Assign model provider to group
#[debug_handler]
pub async fn assign_provider_to_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignProviderToGroupRequest>,
) -> ApiResult2<Json<UserGroupProviderResponse>> {
    match user_group_providers::assign_provider_to_group(request).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error assigning model provider to group: {}", e);
            match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    Err((
                        StatusCode::CONFLICT,
                        AppError::conflict("Provider already assigned to group")
                    ))
                }
                sqlx::Error::RowNotFound => Err((
                    StatusCode::NOT_FOUND,
                    AppError::not_found("Provider or group")
                )),
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to assign model provider to group")
                )),
            }
        }
    }
}

// Remove model provider from group
#[debug_handler]
pub async fn remove_provider_from_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((group_id, provider_id)): Path<(Uuid, Uuid)>,
) -> ApiResult2<StatusCode> {
    match user_group_providers::remove_provider_from_group(group_id, provider_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("Provider assignment")
        )),
        Err(e) => {
            eprintln!("Error removing model provider from group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to remove model provider from group")
            ))
        }
    }
}

// List all user group model provider relationships
#[debug_handler]
pub async fn list_user_group_provider_relationships(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult2<Json<Vec<UserGroupProviderResponse>>> {
    match user_group_providers::list_user_group_provider_relationships().await {
        Ok(relationships) => Ok((StatusCode::OK, Json(relationships))),
        Err(e) => {
            eprintln!(
                "Error listing user group model provider relationships: {}",
                e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to list user group provider relationships")
            ))
        }
    }
}
