use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::api::errors::{ApiResult, AppError};
use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        AssignProviderToGroupRequest, AssignRAGProviderToGroupRequest, AssignUserToGroupRequest,
        CreateUserGroupRequest, UpdateUserGroupRequest,
    },
    queries::{user_group_providers, user_group_rag_providers, user_groups},
};
use crate::api::types::PaginationQuery;

// Create user group
#[debug_handler]
pub async fn create_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUserGroupRequest>,
) -> ApiResult<Json<crate::database::models::UserGroup>> {
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

            // If rag_provider_ids are provided, assign them to the group
            if let Some(rag_provider_ids) = request.rag_provider_ids {
                for provider_id in rag_provider_ids {
                    let assign_request = AssignRAGProviderToGroupRequest {
                        group_id: group.id,
                        provider_id,
                    };
                    if let Err(e) =
                        user_group_rag_providers::assign_rag_provider_to_group(assign_request).await
                    {
                        eprintln!("Error assigning RAG provider to group: {}", e);
                        // Continue with other providers even if one fails
                    }
                }
            }

            // Return the updated group with model provider IDs
            match user_groups::get_user_group_by_id(group.id).await {
                Ok(Some(updated_group)) => Ok((StatusCode::OK, Json(updated_group))),
                Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User group"))),
                Err(e) => {
                    eprintln!("Error getting updated user group: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::internal_error("Failed to get updated user group"),
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating user group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to create user group"),
            ))
        }
    }
}

// Get user group by ID
#[debug_handler]
pub async fn get_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> ApiResult<Json<crate::database::models::UserGroup>> {
    match user_groups::get_user_group_by_id(group_id).await {
        Ok(Some(group)) => Ok((StatusCode::OK, Json(group))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User group"))),
        Err(e) => {
            eprintln!("Error getting user group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get user group"),
            ))
        }
    }
}

// List user groups with pagination
#[debug_handler]
pub async fn list_user_groups(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> ApiResult<Json<crate::database::models::UserGroupListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::list_user_groups(page, per_page).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error listing user groups: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to list user groups"),
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
) -> ApiResult<Json<crate::database::models::UserGroup>> {
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

    // Handle RAG provider assignments if provided
    if let Some(rag_provider_ids) = &request.rag_provider_ids {
        // First, get current assignments
        let current_rag_providers =
            user_group_rag_providers::get_rag_provider_ids_for_group(group_id)
                .await
                .unwrap_or_default();

        // Remove RAG providers that are no longer in the list
        for current_provider in &current_rag_providers {
            if !rag_provider_ids.contains(current_provider) {
                if let Err(e) = user_group_rag_providers::remove_rag_provider_from_group(
                    group_id,
                    *current_provider,
                )
                .await
                {
                    eprintln!("Error removing RAG provider from group: {}", e);
                }
            }
        }

        // Add new RAG providers
        for provider_id in rag_provider_ids {
            if !current_rag_providers.contains(provider_id) {
                let assign_request = AssignRAGProviderToGroupRequest {
                    group_id,
                    provider_id: *provider_id,
                };
                if let Err(e) =
                    user_group_rag_providers::assign_rag_provider_to_group(assign_request).await
                {
                    eprintln!("Error assigning RAG provider to group: {}", e);
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
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("User group"))),
        Err(e) => {
            eprintln!("Error updating user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => {
                    Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")))
                }
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to update user group"),
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
) -> ApiResult<StatusCode> {
    match user_groups::delete_user_group(group_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((StatusCode::NOT_FOUND, AppError::not_found("User group"))),
        Err(e) => {
            eprintln!("Error deleting user group: {}", e);
            match e {
                sqlx::Error::RowNotFound => {
                    Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")))
                }
                _ => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::internal_error("Failed to delete user group"),
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
) -> ApiResult<StatusCode> {
    match user_groups::assign_user_to_group(request.user_id, request.group_id, None).await {
        Ok(()) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Err(e) => {
            eprintln!("Error assigning user to group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to assign user to group"),
            ))
        }
    }
}

// Remove user from group
#[debug_handler]
pub async fn remove_user_from_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((user_id, group_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    match user_groups::remove_user_from_group(user_id, group_id).await {
        Ok(true) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            AppError::not_found("User group membership"),
        )),
        Err(e) => {
            eprintln!("Error removing user from group: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to remove user from group"),
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
) -> ApiResult<Json<crate::database::models::UserListResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    match user_groups::get_group_members(group_id, page, per_page).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            eprintln!("Error getting group members: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::internal_error("Failed to get group members"),
            ))
        }
    }
}
