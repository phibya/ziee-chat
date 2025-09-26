use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{GroupServerAssignment, GroupServerAssignmentResponse, UserGroupMCPServer},
};

/// Assign a system MCP server to a user group
pub async fn assign_server_to_group(
    server_id: Uuid,
    group_id: Uuid,
    assigned_by: Uuid,
) -> Result<UserGroupMCPServer, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assignment = sqlx::query_as!(
        UserGroupMCPServer,
        r#"
        INSERT INTO user_group_mcp_servers (server_id, group_id, assigned_by)
        VALUES ($1, $2, $3)
        ON CONFLICT (group_id, server_id) DO UPDATE SET
            assigned_at = NOW(),
            assigned_by = $3
        RETURNING id, group_id, server_id, assigned_at, assigned_by
        "#,
        server_id,
        group_id,
        assigned_by
    )
    .fetch_one(pool)
    .await?;

    Ok(assignment)
}

/// Unassign a system MCP server from a user group
pub async fn unassign_server_from_group(
    server_id: Uuid,
    group_id: Uuid,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        "DELETE FROM user_group_mcp_servers WHERE server_id = $1 AND group_id = $2",
        server_id,
        group_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// List all server assignments for a group
pub async fn list_group_server_assignments(
    group_id: Uuid,
) -> Result<Vec<GroupServerAssignment>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let assignments = sqlx::query_as!(
        GroupServerAssignment,
        r#"
        SELECT
            ugms.server_id,
            s.name as server_name,
            s.display_name as server_display_name,
            ugms.assigned_at,
            u.username as assigned_by_name
        FROM user_group_mcp_servers ugms
        JOIN mcp_servers s ON ugms.server_id = s.id
        JOIN users u ON ugms.assigned_by = u.id
        WHERE ugms.group_id = $1 AND s.is_system = true
        ORDER BY s.display_name ASC
        "#,
        group_id
    )
    .fetch_all(pool)
    .await?;

    Ok(assignments)
}

/// List all groups with their server assignments (admin view)
pub async fn list_all_group_server_assignments() -> Result<Vec<GroupServerAssignmentResponse>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First get all groups
    let groups = sqlx::query!(
        r#"
        SELECT id, name FROM user_groups
        ORDER BY name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut responses = Vec::new();

    for group in groups {
        let assignments = list_group_server_assignments(group.id).await?;

        responses.push(GroupServerAssignmentResponse {
            group_id: group.id,
            group_name: group.name,
            server_assignments: assignments,
        });
    }

    Ok(responses)
}

/// Get servers assigned to a specific user through their group memberships
pub async fn get_user_assigned_servers(user_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server_ids = sqlx::query!(
        r#"
        SELECT DISTINCT ugms.server_id
        FROM user_group_mcp_servers ugms
        JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
        WHERE ugm.user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(server_ids.into_iter().map(|row| row.server_id).collect())
}

/// Check if a user has access to a system server through group assignment
pub async fn user_has_server_access(user_id: Uuid, server_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_group_mcp_servers ugms
        JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
        WHERE ugm.user_id = $1 AND ugms.server_id = $2
        "#,
        user_id,
        server_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.count.unwrap_or(0) > 0)
}

/// Get groups that have access to a specific server
pub async fn get_groups_with_server_access(server_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let group_ids = sqlx::query!(
        "SELECT group_id FROM user_group_mcp_servers WHERE server_id = $1",
        server_id
    )
    .fetch_all(pool)
    .await?;

    Ok(group_ids.into_iter().map(|row| row.group_id).collect())
}

/// Remove all assignments for a server (when server is deleted)
pub async fn remove_all_server_assignments(server_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        "DELETE FROM user_group_mcp_servers WHERE server_id = $1",
        server_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove all assignments for a group (when group is deleted)
pub async fn remove_all_group_assignments(group_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        "DELETE FROM user_group_mcp_servers WHERE group_id = $1",
        group_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get MCP servers assigned to group (simple list of server IDs)
pub async fn get_group_mcp_servers(group_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server_ids = sqlx::query!(
        "SELECT server_id FROM user_group_mcp_servers WHERE group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    Ok(server_ids.into_iter().map(|row| row.server_id).collect())
}

/// Assign multiple MCP servers to group (batch operation)
pub async fn assign_multiple_servers_to_group(
    group_id: Uuid,
    server_ids: Vec<Uuid>,
    assigned_by: Uuid,
) -> Result<Vec<UserGroupMCPServer>, sqlx::Error> {
    let mut assignments = Vec::new();

    for server_id in server_ids {
        let assignment = assign_server_to_group(server_id, group_id, assigned_by).await?;
        assignments.push(assignment);
    }

    Ok(assignments)
}

/// Remove server from group (wrapper that returns bool for API compatibility)
pub async fn remove_server_from_group(group_id: Uuid, server_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if the assignment exists
    let exists = sqlx::query!(
        "SELECT 1 as exists FROM user_group_mcp_servers WHERE group_id = $1 AND server_id = $2",
        group_id,
        server_id
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if exists {
        // Use the unassign function to actually remove it
        unassign_server_from_group(server_id, group_id).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}