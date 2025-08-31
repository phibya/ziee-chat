use crate::database::get_database_pool;
use crate::database::models::*;
use crate::database::queries::user_group_providers::get_provider_ids_for_group;
use crate::database::queries::user_group_rag_providers::get_rag_provider_ids_for_group;
use uuid::Uuid;

// User Group CRUD operations
pub async fn create_user_group(
    name: String,
    description: Option<String>,
    permissions: Vec<String>,
) -> Result<UserGroup, sqlx::Error> {
    let pool = get_database_pool()?;

    let permissions_json =
        serde_json::to_value(&permissions).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

    let mut group = sqlx::query_as!(
        UserGroup,
        r#"
        INSERT INTO user_groups (name, description, permissions)
        VALUES ($1, $2, $3)
        RETURNING id, name, description,
        permissions as "permissions: Vec<String>",
        '[]'::jsonb as "provider_ids!: Vec<Uuid>",
        '[]'::jsonb as "rag_provider_ids!: Vec<Uuid>",
        is_protected, is_active, created_at, updated_at
        "#,
        name,
        description,
        permissions_json
    )
    .fetch_one(&*pool)
    .await?;

    let provider_ids = get_provider_ids_for_group(group.id)
        .await
        .unwrap_or_default();
    let rag_provider_ids = get_rag_provider_ids_for_group(group.id)
        .await
        .unwrap_or_default();

    group.provider_ids = provider_ids;
    group.rag_provider_ids = rag_provider_ids;

    Ok(group)
}

pub async fn get_user_group_by_id(group_id: Uuid) -> Result<Option<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;

    let mut group = sqlx::query_as!(
        UserGroup,
        r#"SELECT id, name, description, 
        permissions as "permissions: Vec<String>", 
        '[]'::jsonb as "provider_ids!: Vec<Uuid>",
        '[]'::jsonb as "rag_provider_ids!: Vec<Uuid>",
        is_protected, is_active, created_at, updated_at 
        FROM user_groups WHERE id = $1"#,
        group_id
    )
    .fetch_optional(&*pool)
    .await?;

    if let Some(ref mut group) = group {
        let provider_ids = get_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        let rag_provider_ids = get_rag_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        group.provider_ids = provider_ids;
        group.rag_provider_ids = rag_provider_ids;
    }

    Ok(group)
}

pub async fn list_user_groups(
    page: i32,
    per_page: i32,
) -> Result<UserGroupListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM user_groups"
    )
    .fetch_one(&*pool)
    .await?;
    let total: i64 = total_row.count.unwrap_or(0);

    // Get groups
    let mut groups = sqlx::query_as!(
        UserGroup,
        r#"SELECT id, name, description, 
        permissions as "permissions: Vec<String>", 
        '[]'::jsonb as "provider_ids!: Vec<Uuid>",
        '[]'::jsonb as "rag_provider_ids!: Vec<Uuid>",
        is_protected, is_active, created_at, updated_at 
        FROM user_groups ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        per_page as i64,
        offset as i64
    )
    .fetch_all(&*pool)
    .await?;

    // Load provider_ids and rag_provider_ids for each group
    for group in &mut groups {
        let provider_ids = get_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        let rag_provider_ids = get_rag_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        group.provider_ids = provider_ids;
        group.rag_provider_ids = rag_provider_ids;
    }

    Ok(UserGroupListResponse {
        groups,
        total,
        page,
        per_page,
    })
}

pub async fn update_user_group(
    group_id: Uuid,
    name: Option<String>,
    description: Option<String>,
    permissions: Option<Vec<String>>,
    is_active: Option<bool>,
) -> Result<Option<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if this is a protected group and apply restrictions
    let existing_group = get_user_group_by_id(group_id).await?;
    if let Some(group) = &existing_group {
        if group.is_protected {
            // For protected groups, only allow editing description
            // Name, permissions, and is_active cannot be changed
            if name.is_some() || permissions.is_some() || is_active.is_some() {
                return Err(sqlx::Error::RowNotFound);
            }
        }
    }

    // If no updates are provided, return the existing group
    if name.is_none() && description.is_none() && permissions.is_none() && is_active.is_none() {
        return get_user_group_by_id(group_id).await;
    }

    // Update individual fields with separate queries
    if let Some(name) = name.clone() {
        sqlx::query!(
            "UPDATE user_groups SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            name,
            group_id
        )
        .execute(&*pool)
        .await?;
    }

    if let Some(description) = description.clone() {
        sqlx::query!(
            "UPDATE user_groups SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            description,
            group_id
        )
        .execute(&*pool)
        .await?;
    }

    if let Some(permissions) = permissions.clone() {
        let permissions_json =
            serde_json::to_value(&permissions).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
        sqlx::query!(
            "UPDATE user_groups SET permissions = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            permissions_json,
            group_id
        )
        .execute(&*pool)
        .await?;
    }

    if let Some(is_active) = is_active {
        sqlx::query!(
            "UPDATE user_groups SET is_active = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            is_active,
            group_id
        )
        .execute(&*pool)
        .await?;
    }

    let mut group = get_user_group_by_id(group_id).await?;

    if let Some(ref mut group) = group {
        let provider_ids = get_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        let rag_provider_ids = get_rag_provider_ids_for_group(group.id)
            .await
            .unwrap_or_default();
        group.provider_ids = provider_ids;
        group.rag_provider_ids = rag_provider_ids;
    }

    Ok(group)
}

pub async fn delete_user_group(group_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if this is a protected group (admin or user)
    let existing_group = get_user_group_by_id(group_id).await?;
    if let Some(group) = existing_group {
        if group.is_protected {
            return Err(sqlx::Error::RowNotFound);
        }
    }

    let result = sqlx::query!(
        "DELETE FROM user_groups WHERE id = $1",
        group_id
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// User Group Membership operations
pub async fn assign_user_to_group(
    user_id: Uuid,
    group_id: Uuid,
    assigned_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;

    sqlx::query!(
        "INSERT INTO user_group_memberships (user_id, group_id, assigned_by) VALUES ($1, $2, $3)",
        user_id,
        group_id,
        assigned_by
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

pub async fn remove_user_from_group(user_id: Uuid, group_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if user is protected
    let user_protected = sqlx::query!(
        "SELECT is_protected FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&*pool)
    .await?;

    if let Some(row) = user_protected {
        if row.is_protected {
            // Protected users cannot be removed from groups
            return Err(sqlx::Error::RowNotFound);
        }
    }

    let result = sqlx::query!(
        "DELETE FROM user_group_memberships WHERE user_id = $1 AND group_id = $2",
        user_id,
        group_id
    )
    .execute(&*pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_user_groups(user_id: Uuid) -> Result<Vec<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;

    let groups = sqlx::query_as!(
        UserGroup,
        r#"
        SELECT ug.id, ug.name, ug.description, 
        ug.permissions as "permissions: Vec<String>", 
        '[]'::jsonb as "provider_ids!: Vec<Uuid>",
        '[]'::jsonb as "rag_provider_ids!: Vec<Uuid>",
        ug.is_protected, ug.is_active, ug.created_at, ug.updated_at 
        FROM user_groups ug
        JOIN user_group_memberships ugm ON ug.id = ugm.group_id
        WHERE ugm.user_id = $1 AND ug.is_active = TRUE
        ORDER BY ug.name
        "#,
        user_id
    )
    .fetch_all(&*pool)
    .await?;

    // Note: provider_ids are left empty as they are loaded separately when needed

    Ok(groups)
}

// Helper function to get admin group ID
pub async fn get_admin_group_id() -> Result<Option<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query!(
        "SELECT id FROM user_groups WHERE name = 'admin'"
    )
    .fetch_optional(&*pool)
    .await?;

    Ok(row.map(|r| r.id))
}

// Helper function to get user group ID
pub async fn get_user_group_id() -> Result<Option<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query!(
        "SELECT id FROM user_groups WHERE name = 'user'"
    )
    .fetch_optional(&*pool)
    .await?;

    Ok(row.map(|r| r.id))
}

// Function to assign user to admin group (for root/admin users)
pub async fn assign_user_to_admin_group(user_id: Uuid) -> Result<(), sqlx::Error> {
    if let Some(admin_group_id) = get_admin_group_id().await? {
        // Check if user is already in admin group
        let pool = get_database_pool()?;
        let existing = sqlx::query!(
            "SELECT id FROM user_group_memberships WHERE user_id = $1 AND group_id = $2",
            user_id,
            admin_group_id
        )
        .fetch_optional(&*pool)
        .await?;

        if existing.is_none() {
            assign_user_to_group(user_id, admin_group_id, None).await?;
        }
    }
    Ok(())
}

// Function to assign user to default user group
pub async fn assign_user_to_default_group(user_id: Uuid) -> Result<(), sqlx::Error> {
    if let Some(user_group_id) = get_user_group_id().await? {
        // Check if user is already in user group
        let pool = get_database_pool()?;
        let existing = sqlx::query!(
            "SELECT id FROM user_group_memberships WHERE user_id = $1 AND group_id = $2",
            user_id,
            user_group_id
        )
        .fetch_optional(&*pool)
        .await?;

        if existing.is_none() {
            assign_user_to_group(user_id, user_group_id, None).await?;
        }
    }
    Ok(())
}

pub async fn get_group_members(
    group_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<UserListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM user_group_memberships WHERE group_id = $1",
        group_id
    )
    .fetch_one(&*pool)
    .await?;
    let total: i64 = total_row.count.unwrap_or(0);

    // Get user IDs
    let rows = sqlx::query!(
        "SELECT user_id FROM user_group_memberships WHERE group_id = $1 ORDER BY assigned_at DESC LIMIT $2 OFFSET $3",
        group_id,
        per_page as i64,
        offset as i64
    )
    .fetch_all(&*pool)
    .await?;

    let mut users = Vec::new();
    for row in rows {
        if let Some(user) = crate::database::queries::users::get_user_by_id(row.user_id).await? {
            users.push(user);
        }
    }

    Ok(UserListResponse {
        users,
        total,
        page,
        per_page,
    })
}
