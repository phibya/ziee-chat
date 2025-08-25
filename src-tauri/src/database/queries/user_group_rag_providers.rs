use uuid::Uuid;

use crate::database::queries::{rag_providers::get_rag_provider_by_id, user_groups::get_user_group_by_id};
use crate::database::{
    get_database_pool,
    models::{
        AssignRAGProviderToGroupRequest, RAGProvider, UserGroup, UserGroupRAGProvider,
        UserGroupRAGProviderResponse, UpdateGroupRAGProviderRequest,
    },
};

/// Assign a RAG provider to a user group with creation permissions
pub async fn assign_rag_provider_to_group(
    request: AssignRAGProviderToGroupRequest,
) -> Result<UserGroupRAGProviderResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First validate that both the RAG provider and group exist before inserting
    let provider = get_rag_provider_by_id(request.provider_id)
        .await?
        .ok_or_else(|| {
            eprintln!("RAG provider not found: {}", request.provider_id);
            sqlx::Error::RowNotFound
        })?;

    let group = get_user_group_by_id(request.group_id)
        .await?
        .ok_or_else(|| {
            eprintln!("User group not found: {}", request.group_id);
            sqlx::Error::RowNotFound
        })?;

    // Check if the relationship already exists
    let existing_relationship: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM user_group_rag_providers WHERE group_id = $1 AND provider_id = $2",
    )
    .bind(request.group_id)
    .bind(request.provider_id)
    .fetch_optional(pool)
    .await?;

    if existing_relationship.is_some() {
        eprintln!(
            "Relationship already exists between group {} and RAG provider {}",
            request.group_id, request.provider_id
        );
        return Err(sqlx::Error::RowNotFound); // Use a simpler error
    }

    let relationship_id = Uuid::new_v4();
    let relationship_row: UserGroupRAGProvider = sqlx::query_as(
        "INSERT INTO user_group_rag_providers (id, group_id, provider_id, can_create_instance) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, group_id, provider_id, can_create_instance, assigned_at, updated_at",
    )
    .bind(relationship_id)
    .bind(request.group_id)
    .bind(request.provider_id)
    .bind(request.can_create_instance)
    .fetch_one(pool)
    .await?;

    Ok(UserGroupRAGProviderResponse {
        id: relationship_row.id,
        group_id: relationship_row.group_id,
        provider_id: relationship_row.provider_id,
        can_create_instance: relationship_row.can_create_instance,
        assigned_at: relationship_row.assigned_at,
        updated_at: relationship_row.updated_at,
        provider,
        group,
    })
}

/// Update RAG provider permissions for a user group
pub async fn update_group_rag_provider_permissions(
    group_id: Uuid,
    provider_id: Uuid,
    request: UpdateGroupRAGProviderRequest,
) -> Result<UserGroupRAGProviderResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Update the relationship
    let relationship_row: UserGroupRAGProvider = sqlx::query_as(
        "UPDATE user_group_rag_providers 
         SET can_create_instance = COALESCE($3, can_create_instance)
         WHERE group_id = $1 AND provider_id = $2
         RETURNING id, group_id, provider_id, can_create_instance, assigned_at, updated_at",
    )
    .bind(group_id)
    .bind(provider_id)
    .bind(request.can_create_instance)
    .fetch_one(pool)
    .await?;

    // Get the provider and group details
    let provider = get_rag_provider_by_id(provider_id)
        .await?
        .ok_or_else(|| {
            eprintln!("RAG provider not found: {}", provider_id);
            sqlx::Error::RowNotFound
        })?;

    let group = get_user_group_by_id(group_id)
        .await?
        .ok_or_else(|| {
            eprintln!("User group not found: {}", group_id);
            sqlx::Error::RowNotFound
        })?;

    Ok(UserGroupRAGProviderResponse {
        id: relationship_row.id,
        group_id: relationship_row.group_id,
        provider_id: relationship_row.provider_id,
        can_create_instance: relationship_row.can_create_instance,
        assigned_at: relationship_row.assigned_at,
        updated_at: relationship_row.updated_at,
        provider,
        group,
    })
}

/// Remove a RAG provider from a user group
pub async fn remove_rag_provider_from_group(
    group_id: Uuid,
    provider_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query(
        "DELETE FROM user_group_rag_providers 
         WHERE group_id = $1 AND provider_id = $2",
    )
    .bind(group_id)
    .bind(provider_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get all RAG providers available to a user for creating instances
pub async fn get_creatable_rag_providers_for_user(user_id: Uuid) -> Result<Vec<RAGProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get RAG provider IDs that the user can create instances for
    let provider_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT ugrp.provider_id 
         FROM user_group_rag_providers ugrp
         JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id
         JOIN user_groups ug ON ugm.group_id = ug.id
         WHERE ugm.user_id = $1 
         AND ug.is_active = true 
         AND ugrp.can_create_instance = true",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut providers = Vec::new();
    for (provider_id,) in provider_ids {
        if let Some(provider) = get_rag_provider_by_id(provider_id).await? {
            providers.push(provider);
        }
    }

    Ok(providers)
}

/// Get all RAG providers assigned to a user group
#[allow(dead_code)] // For future admin interface functionality
pub async fn get_rag_providers_for_group(group_id: Uuid) -> Result<Vec<RAGProvider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT provider_id FROM user_group_rag_providers WHERE group_id = $1")
            .bind(group_id)
            .fetch_all(pool)
            .await?;

    let mut providers = Vec::new();
    for (provider_id,) in provider_ids {
        if let Some(provider) = get_rag_provider_by_id(provider_id).await? {
            providers.push(provider);
        }
    }

    Ok(providers)
}

/// Get all user groups that have access to a RAG provider
#[allow(dead_code)] // For future admin interface functionality
pub async fn get_groups_for_rag_provider(provider_id: Uuid) -> Result<Vec<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let group_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT group_id FROM user_group_rag_providers WHERE provider_id = $1")
            .bind(provider_id)
            .fetch_all(pool)
            .await?;

    let mut groups = Vec::new();
    for (group_id,) in group_ids {
        if let Some(group) = get_user_group_by_id(group_id).await? {
            groups.push(group);
        }
    }

    Ok(groups)
}

/// Check if a user can create instances with a specific RAG provider
pub async fn can_user_create_rag_instance(user_id: Uuid, provider_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if user belongs to any active group with can_create_instance=true for this provider
    let has_permission: Option<(bool,)> = sqlx::query_as(
        "SELECT true 
         FROM user_group_memberships ugm
         JOIN user_groups ug ON ugm.group_id = ug.id
         JOIN user_group_rag_providers ugrp ON ug.id = ugrp.group_id
         WHERE ugm.user_id = $1 
         AND ugrp.provider_id = $2
         AND ug.is_active = true 
         AND ugrp.can_create_instance = true
         LIMIT 1",
    )
    .bind(user_id)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    Ok(has_permission.is_some())
}

/// List all relationships between user groups and RAG providers
pub async fn list_user_group_rag_provider_relationships(
) -> Result<Vec<UserGroupRAGProviderResponse>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let relationships: Vec<UserGroupRAGProvider> = sqlx::query_as(
        "SELECT id, group_id, provider_id, can_create_instance, assigned_at, updated_at
         FROM user_group_rag_providers 
         ORDER BY assigned_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut responses = Vec::new();
    for relationship in relationships {
        if let (Some(provider), Some(group)) = (
            get_rag_provider_by_id(relationship.provider_id).await?,
            get_user_group_by_id(relationship.group_id).await?,
        ) {
            responses.push(UserGroupRAGProviderResponse {
                id: relationship.id,
                group_id: relationship.group_id,
                provider_id: relationship.provider_id,
                can_create_instance: relationship.can_create_instance,
                assigned_at: relationship.assigned_at,
                updated_at: relationship.updated_at,
                provider,
                group,
            });
        }
    }

    Ok(responses)
}