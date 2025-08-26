use uuid::Uuid;

use crate::database::queries::{providers::get_provider_by_id, user_groups::get_user_group_by_id};
use crate::database::{
    get_database_pool,
    models::{
        AssignProviderToGroupRequest, Provider, UserGroup, UserGroupProvider,
        UserGroupProviderResponse,
    },
};

/// Assign a provider to a user group
pub async fn assign_provider_to_group(
    request: AssignProviderToGroupRequest,
) -> Result<UserGroupProviderResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First validate that both the provider and group exist before inserting
    let provider = get_provider_by_id(request.provider_id)
        .await?
        .ok_or_else(|| {
            eprintln!("Model provider not found: {}", request.provider_id);
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
        "SELECT id FROM user_group_providers WHERE group_id = $1 AND provider_id = $2",
    )
    .bind(request.group_id)
    .bind(request.provider_id)
    .fetch_optional(pool)
    .await?;

    if existing_relationship.is_some() {
        eprintln!(
            "Relationship already exists between group {} and provider {}",
            request.group_id, request.provider_id
        );
        return Err(sqlx::Error::RowNotFound); // Use a simpler error
    }

    let relationship_id = Uuid::new_v4();
    let relationship_row: UserGroupProvider = sqlx::query_as(
        "INSERT INTO user_group_providers (id, group_id, provider_id) 
         VALUES ($1, $2, $3) 
         RETURNING id, group_id, provider_id, assigned_at",
    )
    .bind(relationship_id)
    .bind(request.group_id)
    .bind(request.provider_id)
    .fetch_one(pool)
    .await?;

    Ok(UserGroupProviderResponse {
        id: relationship_row.id,
        group_id: relationship_row.group_id,
        provider_id: relationship_row.provider_id,
        assigned_at: relationship_row.assigned_at,
        provider,
        group,
    })
}

/// Remove a provider from a user group
pub async fn remove_provider_from_group(
    group_id: Uuid,
    provider_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query(
        "DELETE FROM user_group_providers 
         WHERE group_id = $1 AND provider_id = $2",
    )
    .bind(group_id)
    .bind(provider_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get provider IDs assigned to a user group
pub async fn get_provider_ids_for_group(group_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT provider_id FROM user_group_providers WHERE group_id = $1")
            .bind(group_id)
            .fetch_all(pool)
            .await?;

    Ok(provider_ids.into_iter().map(|(id,)| id).collect())
}

/// Get all user groups that have access to a model provider
pub async fn get_groups_for_provider(provider_id: Uuid) -> Result<Vec<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let group_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT group_id FROM user_group_providers WHERE provider_id = $1")
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

/// Get all model providers available to a user based on their group memberships
/// Users with config::providers::read permission get access to all providers
pub async fn get_providers_for_user(user_id: Uuid) -> Result<Vec<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if the user has config::providers::read permission
    let has_read_permission = check_user_providers_read_permission(user_id).await?;

    if has_read_permission {
        // User has read permission, return all model providers (enabled and disabled)
        return get_all_providers().await;
    }

    // User doesn't have read permission, return only providers assigned to their groups
    let provider_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT ugmp.provider_id 
         FROM user_group_providers ugmp
         JOIN user_group_memberships ugm ON ugmp.group_id = ugm.group_id
         JOIN user_groups ug ON ugm.group_id = ug.id
         WHERE ugm.user_id = $1 AND ug.is_active = true",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut providers = Vec::new();
    for (provider_id,) in provider_ids {
        if let Some(provider) = get_provider_by_id(provider_id).await? {
            providers.push(provider);
        }
    }

    Ok(providers)
}

/// Check if a user has config::providers::read permission
async fn check_user_providers_read_permission(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if user belongs to any active group with config::providers::read permission
    let has_permission: Option<(bool,)> = sqlx::query_as(
        "SELECT true 
         FROM user_group_memberships ugm
         JOIN user_groups ug ON ugm.group_id = ug.id
         WHERE ugm.user_id = $1 
         AND ug.is_active = true 
         AND (
             ug.permissions @> $2::jsonb OR 
             ug.permissions @> $3::jsonb OR 
             ug.permissions @> $4::jsonb
         )
         LIMIT 1",
    )
    .bind(user_id)
    .bind(serde_json::json!(["config::providers::read"]))
    .bind(serde_json::json!(["config::providers::*"]))
    .bind(serde_json::json!(["*"]))
    .fetch_optional(pool)
    .await?;

    Ok(has_permission.is_some())
}

/// Get all model providers (for admin users)
async fn get_all_providers() -> Result<Vec<Provider>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT id FROM providers ORDER BY built_in DESC, created_at ASC")
            .fetch_all(pool)
            .await?;

    let mut providers = Vec::new();
    for (provider_id,) in provider_ids {
        if let Some(provider) = get_provider_by_id(provider_id).await? {
            providers.push(provider);
        }
    }

    Ok(providers)
}
