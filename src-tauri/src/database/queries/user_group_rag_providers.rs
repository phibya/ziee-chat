use uuid::Uuid;

use crate::database::queries::{rag_providers::get_rag_provider_by_id, user_groups::get_user_group_by_id};
use crate::database::{
    get_database_pool,
    models::{
        AssignRAGProviderToGroupRequest, RAGProvider, UserGroupRAGProvider,
        UserGroupRAGProviderResponse,
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
        "INSERT INTO user_group_rag_providers (id, group_id, provider_id) 
         VALUES ($1, $2, $3) 
         RETURNING id, group_id, provider_id, assigned_at, updated_at",
    )
    .bind(relationship_id)
    .bind(request.group_id)
    .bind(request.provider_id)
    .fetch_one(pool)
    .await?;

    Ok(UserGroupRAGProviderResponse {
        id: relationship_row.id,
        group_id: relationship_row.group_id,
        provider_id: relationship_row.provider_id,
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

    // Get RAG provider IDs that the user can access and that allow user instance creation
    let provider_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT ugrp.provider_id 
         FROM user_group_rag_providers ugrp
         JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id
         JOIN user_groups ug ON ugm.group_id = ug.id
         JOIN rag_providers rp ON ugrp.provider_id = rp.id
         WHERE ugm.user_id = $1 
         AND ug.is_active = true
         AND rp.can_user_create_instance = true",
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

/// Check if a user can create instances with a specific RAG provider
pub async fn can_user_create_rag_instance(user_id: Uuid, provider_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if user belongs to any active group assigned to this provider and the provider allows user instance creation
    let has_permission: Option<(bool,)> = sqlx::query_as(
        "SELECT true 
         FROM user_group_memberships ugm
         JOIN user_groups ug ON ugm.group_id = ug.id
         JOIN user_group_rag_providers ugrp ON ug.id = ugrp.group_id
         JOIN rag_providers rp ON ugrp.provider_id = rp.id
         WHERE ugm.user_id = $1 
         AND ugrp.provider_id = $2
         AND ug.is_active = true
         AND rp.can_user_create_instance = true
         LIMIT 1",
    )
    .bind(user_id)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    Ok(has_permission.is_some())
}

/// Get RAG provider IDs assigned to a user group
pub async fn get_rag_provider_ids_for_group(group_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT provider_id FROM user_group_rag_providers WHERE group_id = $1"
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    Ok(provider_ids.into_iter().map(|(id,)| id).collect())
}
