use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::AuthenticatedUser;
use crate::database::{
    models::{
        AssignProviderToGroupRequest, AssignUserToGroupRequest, CreateUserGroupRequest,
        UpdateUserGroupRequest, UserGroupProviderResponse,
    },
    queries::{user_group_model_providers, user_groups},
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<i32>,
    per_page: Option<i32>,
}

// Create user group
pub async fn create_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUserGroupRequest>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
    match user_groups::create_user_group(request.name, request.description, request.permissions)
        .await
    {
        Ok(group) => {
            // If model_provider_ids are provided, assign them to the group
            if let Some(provider_ids) = request.provider_ids {
                for provider_id in provider_ids {
                    let assign_request = AssignProviderToGroupRequest {
                        group_id: group.id,
                        provider_id,
                    };
                    if let Err(e) =
                        user_group_model_providers::assign_provider_to_group(assign_request)
                            .await
                    {
                        eprintln!("Error assigning model provider to group: {}", e);
                        // Continue with other providers even if one fails
                    }
                }
            }

            // Return the updated group with model provider IDs
            match user_groups::get_user_group_by_id(group.id).await {
                Ok(Some(updated_group)) => Ok(Json(updated_group)),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(e) => {
                    eprintln!("Error getting updated user group: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating user group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get user group by ID
pub async fn get_user_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<crate::database::models::UserGroupListResponse>, StatusCode> {
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateUserGroupRequest>,
) -> Result<Json<crate::database::models::UserGroup>, StatusCode> {
    // Handle model provider assignments if provided
    if let Some(provider_ids) = &request.provider_ids {
        // First, get current assignments
        let current_providers =
            user_group_model_providers::get_provider_ids_for_group(group_id)
                .await
                .unwrap_or_default();

        // Remove providers that are no longer in the list
        for current_provider in &current_providers {
            if !provider_ids.contains(current_provider) {
                if let Err(e) = user_group_model_providers::remove_provider_from_group(
                    group_id,
                    *current_provider,
                )
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
                if let Err(e) =
                    user_group_model_providers::assign_provider_to_group(assign_request).await
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignUserToGroupRequest>,
) -> Result<StatusCode, StatusCode> {
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((user_id, group_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
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
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<crate::database::models::UserListResponse>, StatusCode> {
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

// Get model providers for a group
pub async fn get_group_providers(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<crate::database::models::Provider>>, StatusCode> {
    match user_group_model_providers::get_providers_for_group(group_id).await {
        Ok(providers) => Ok(Json(providers)),
        Err(e) => {
            eprintln!("Error getting model providers for group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Assign model provider to group
pub async fn assign_provider_to_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignProviderToGroupRequest>,
) -> Result<Json<UserGroupProviderResponse>, StatusCode> {
    match user_group_model_providers::assign_provider_to_group(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            eprintln!("Error assigning model provider to group: {}", e);
            match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    Err(StatusCode::CONFLICT)
                }
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// Remove model provider from group
pub async fn remove_provider_from_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((group_id, provider_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match user_group_model_providers::remove_provider_from_group(group_id, provider_id).await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error removing model provider from group: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// List all user group model provider relationships
pub async fn list_user_group_model_provider_relationships(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<UserGroupProviderResponse>>, StatusCode> {
    match user_group_model_providers::list_user_group_model_provider_relationships().await {
        Ok(relationships) => Ok(Json(relationships)),
        Err(e) => {
            eprintln!(
                "Error listing user group model provider relationships: {}",
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
