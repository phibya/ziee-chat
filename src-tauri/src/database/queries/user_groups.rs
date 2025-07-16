use crate::database::get_database_pool;
use crate::database::models::*;
use crate::database::queries::user_group_providers::get_provider_ids_for_group;
use sqlx::Row;
use uuid::Uuid;

// User Group CRUD operations
pub async fn create_user_group(
    name: String,
    description: Option<String>,
    permissions: serde_json::Value,
) -> Result<UserGroup, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query(
        r#"
        INSERT INTO user_groups (name, description, permissions)
        VALUES ($1, $2, $3)
        RETURNING id, name, description, permissions, is_protected, is_active, created_at, updated_at
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&permissions)
    .fetch_one(&*pool)
    .await?;

    let group_id: Uuid = row.get("id");
    let provider_ids = get_provider_ids_for_group(group_id)
        .await
        .unwrap_or_default();

    Ok(UserGroup {
        id: group_id,
        name: row.get("name"),
        description: row.get("description"),
        permissions: row.get("permissions"),
        provider_ids,
        is_protected: row.get("is_protected"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_user_group_by_id(group_id: Uuid) -> Result<Option<UserGroup>, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query("SELECT * FROM user_groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(&*pool)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let group_id: Uuid = row.get("id");
    let provider_ids = get_provider_ids_for_group(group_id)
        .await
        .unwrap_or_default();

    Ok(Some(UserGroup {
        id: group_id,
        name: row.get("name"),
        description: row.get("description"),
        permissions: row.get("permissions"),
        provider_ids,
        is_protected: row.get("is_protected"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn list_user_groups(
    page: i32,
    per_page: i32,
) -> Result<UserGroupListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let offset = (page - 1) * per_page;

    // Get total count
    let total_row = sqlx::query("SELECT COUNT(*) as count FROM user_groups")
        .fetch_one(&*pool)
        .await?;
    let total: i64 = total_row.get("count");

    // Get groups
    let rows = sqlx::query("SELECT * FROM user_groups ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(per_page)
        .bind(offset)
        .fetch_all(&*pool)
        .await?;

    let mut groups = Vec::new();
    for row in rows {
        let group_id: Uuid = row.get("id");
        let provider_ids = get_provider_ids_for_group(group_id)
            .await
            .unwrap_or_default();

        groups.push(UserGroup {
            id: group_id,
            name: row.get("name"),
            description: row.get("description"),
            permissions: row.get("permissions"),
            provider_ids,
            is_protected: row.get("is_protected"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
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
    permissions: Option<serde_json::Value>,
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

    let mut query = String::from("UPDATE user_groups SET");
    let mut updates = Vec::new();
    let mut param_index = 1;

    if name.is_some() {
        updates.push(format!(" name = ${}", param_index));
        param_index += 1;
    }

    if description.is_some() {
        updates.push(format!(" description = ${}", param_index));
        param_index += 1;
    }

    if permissions.is_some() {
        updates.push(format!(" permissions = ${}", param_index));
        param_index += 1;
    }

    if is_active.is_some() {
        updates.push(format!(" is_active = ${}", param_index));
        param_index += 1;
    }

    // If no updates are provided, return the existing group
    if updates.is_empty() {
        return get_user_group_by_id(group_id).await;
    }

    // Always update the updated_at field
    updates.push(" updated_at = CURRENT_TIMESTAMP".to_string());

    query.push_str(&updates.join(","));
    query.push_str(&format!(" WHERE id = ${} RETURNING *", param_index));

    let mut sql_query = sqlx::query(&query);

    if let Some(name) = name {
        sql_query = sql_query.bind(name);
    }
    if let Some(description) = description {
        sql_query = sql_query.bind(description);
    }
    if let Some(permissions) = permissions {
        sql_query = sql_query.bind(permissions);
    }
    if let Some(is_active) = is_active {
        sql_query = sql_query.bind(is_active);
    }

    sql_query = sql_query.bind(group_id);

    let row = sql_query.fetch_optional(&*pool).await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let group_id: Uuid = row.get("id");
    let provider_ids = get_provider_ids_for_group(group_id)
        .await
        .unwrap_or_default();

    Ok(Some(UserGroup {
        id: group_id,
        name: row.get("name"),
        description: row.get("description"),
        permissions: row.get("permissions"),
        provider_ids,
        is_protected: row.get("is_protected"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
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

    let result = sqlx::query("DELETE FROM user_groups WHERE id = $1")
        .bind(group_id)
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

    sqlx::query(
        "INSERT INTO user_group_memberships (user_id, group_id, assigned_by) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(group_id)
    .bind(assigned_by)
    .execute(&*pool)
    .await?;

    Ok(())
}

pub async fn remove_user_from_group(user_id: Uuid, group_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;

    // Check if user is protected
    let user_protected: Option<(bool,)> =
        sqlx::query_as("SELECT is_protected FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&*pool)
            .await?;

    if let Some((is_protected,)) = user_protected {
        if is_protected {
            // Protected users cannot be removed from groups
            return Err(sqlx::Error::RowNotFound);
        }
    }

    let result =
        sqlx::query("DELETE FROM user_group_memberships WHERE user_id = $1 AND group_id = $2")
            .bind(user_id)
            .bind(group_id)
            .execute(&*pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_user_groups(user_id: Uuid) -> Result<Vec<UserGroupDb>, sqlx::Error> {
    let pool = get_database_pool()?;

    let rows = sqlx::query(
        r#"
        SELECT ug.* FROM user_groups ug
        JOIN user_group_memberships ugm ON ug.id = ugm.group_id
        WHERE ugm.user_id = $1 AND ug.is_active = TRUE
        ORDER BY ug.name
        "#,
    )
    .bind(user_id)
    .fetch_all(&*pool)
    .await?;

    let groups = rows
        .into_iter()
        .map(|row| UserGroupDb {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            permissions: row.get("permissions"),
            is_protected: row.get("is_protected"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(groups)
}

// Helper function to get admin group ID
pub async fn get_admin_group_id() -> Result<Option<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query("SELECT id FROM user_groups WHERE name = 'admin'")
        .fetch_optional(&*pool)
        .await?;

    Ok(row.map(|r| r.get("id")))
}

// Helper function to get user group ID
pub async fn get_user_group_id() -> Result<Option<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;

    let row = sqlx::query("SELECT id FROM user_groups WHERE name = 'user'")
        .fetch_optional(&*pool)
        .await?;

    Ok(row.map(|r| r.get("id")))
}

// Function to assign user to admin group (for root/admin users)
pub async fn assign_user_to_admin_group(user_id: Uuid) -> Result<(), sqlx::Error> {
    if let Some(admin_group_id) = get_admin_group_id().await? {
        // Check if user is already in admin group
        let pool = get_database_pool()?;
        let existing = sqlx::query(
            "SELECT id FROM user_group_memberships WHERE user_id = $1 AND group_id = $2",
        )
        .bind(user_id)
        .bind(admin_group_id)
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
        let existing = sqlx::query(
            "SELECT id FROM user_group_memberships WHERE user_id = $1 AND group_id = $2",
        )
        .bind(user_id)
        .bind(user_group_id)
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
    let total_row =
        sqlx::query("SELECT COUNT(*) as count FROM user_group_memberships WHERE group_id = $1")
            .bind(group_id)
            .fetch_one(&*pool)
            .await?;
    let total: i64 = total_row.get("count");

    // Get user IDs
    let rows = sqlx::query(
        "SELECT user_id FROM user_group_memberships WHERE group_id = $1 ORDER BY assigned_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(group_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&*pool)
    .await?;

    let mut users = Vec::new();
    for row in rows {
        let user_id: Uuid = row.get("user_id");
        if let Some(user) = crate::database::queries::users::get_user_by_id(user_id).await? {
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
