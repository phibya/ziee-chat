use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{MCPTool, MCPToolWithServer},
};

/// Cache discovered tools for a server
pub async fn cache_discovered_tools(
    server_id: Uuid,
    tools: Vec<(String, Option<String>, serde_json::Value)>, // (name, description, schema)
) -> Result<Vec<MCPTool>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Start a transaction to clear old tools and insert new ones
    let mut tx = pool.begin().await?;

    // Clear existing tools for this server
    sqlx::query!("DELETE FROM mcp_tools_cache WHERE server_id = $1", server_id)
        .execute(&mut *tx)
        .await?;

    let mut cached_tools = Vec::new();

    // Insert new tools
    for (tool_name, tool_description, input_schema) in tools {
        let tool = sqlx::query_as!(
            MCPTool,
            r#"
            INSERT INTO mcp_tools_cache (
                server_id, tool_name, tool_description, input_schema
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id, server_id, tool_name, tool_description, input_schema,
                discovered_at, last_used_at, usage_count
            "#,
            server_id,
            tool_name,
            tool_description,
            input_schema
        )
        .fetch_one(&mut *tx)
        .await?;

        cached_tools.push(tool);
    }

    tx.commit().await?;
    Ok(cached_tools)
}

/// Get all cached tools for a server
pub async fn get_cached_tools_for_server(server_id: Uuid) -> Result<Vec<MCPTool>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let tools = sqlx::query_as!(
        MCPTool,
        r#"
        SELECT
            id, server_id, tool_name, tool_description, input_schema,
            discovered_at, last_used_at, usage_count
        FROM mcp_tools_cache
        WHERE server_id = $1
        ORDER BY tool_name ASC
        "#,
        server_id
    )
    .fetch_all(pool)
    .await?;

    Ok(tools)
}

/// Get user accessible tools (from user servers + accessible system servers)
pub async fn get_user_accessible_tools(user_id: Uuid) -> Result<Vec<MCPToolWithServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let tools = sqlx::query_as!(
        MCPToolWithServer,
        r#"
        SELECT
            t.id, t.server_id, t.tool_name, t.tool_description, t.input_schema,
            t.discovered_at, t.last_used_at, t.usage_count,
            s.name as "server_name!",
            s.display_name as "server_display_name!",
            s.is_system as "is_system!",
            s.transport_type::TEXT as "transport_type!",
            ga.auto_approve as "global_auto_approve?: bool",
            ga.expires_at as "global_approval_expires_at?",
            ga.notes as "global_approval_notes?"
        FROM mcp_tools_cache t
        INNER JOIN mcp_servers s ON t.server_id = s.id
        LEFT JOIN mcp_tool_approvals ga ON (
            t.server_id = ga.server_id
            AND t.tool_name = ga.tool_name
            AND ga.user_id = $1
            AND ga.is_global = true
            AND ga.approved = true
            AND (ga.expires_at IS NULL OR ga.expires_at > NOW())
        )
        WHERE (
            -- User's own servers
            s.user_id = $1
            -- OR accessible system servers through group membership
            OR (
                s.is_system = true
                AND EXISTS (
                    SELECT 1
                    FROM user_group_mcp_servers ugms
                    INNER JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
                    WHERE ugms.server_id = s.id AND ugm.user_id = $1
                )
            )
        )
        ORDER BY s.display_name ASC, t.tool_name ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(tools)
}

/// Find tool by name for a user (with server disambiguation if needed)
pub async fn find_tool_by_name_for_user(
    user_id: Uuid,
    tool_name: &str,
    server_id: Option<Uuid>,
) -> Result<Option<MCPToolWithServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let tool = match server_id {
        Some(sid) => {
            // Specific server requested
            sqlx::query_as!(
                MCPToolWithServer,
                r#"
                SELECT
                    t.id, t.server_id, t.tool_name, t.tool_description, t.input_schema,
                    t.discovered_at, t.last_used_at, t.usage_count,
                    s.name as "server_name!", s.display_name as "server_display_name!",
                    s.is_system, s.transport_type::TEXT as "transport_type!",
                    NULL::BOOLEAN as "global_auto_approve",
                    NULL::TIMESTAMP WITH TIME ZONE as "global_approval_expires_at",
                    NULL::TEXT as "global_approval_notes"
                FROM mcp_tools_cache t
                JOIN mcp_servers s ON t.server_id = s.id
                LEFT JOIN user_group_mcp_servers ugms ON s.id = ugms.server_id
                LEFT JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
                WHERE t.tool_name = $1 AND t.server_id = $2 AND (
                    -- User's own server
                    s.user_id = $3
                    -- OR accessible system server through group membership
                    OR (s.is_system = true AND ugm.user_id = $3)
                )
                "#,
                tool_name,
                sid,
                user_id
            )
            .fetch_optional(pool)
            .await?
        }
        None => {
            // Find first available tool with this name
            sqlx::query_as!(
                MCPToolWithServer,
                r#"
                SELECT
                    t.id, t.server_id, t.tool_name, t.tool_description, t.input_schema,
                    t.discovered_at, t.last_used_at, t.usage_count,
                    s.name as "server_name!", s.display_name as "server_display_name!",
                    s.is_system, s.transport_type::TEXT as "transport_type!",
                    NULL::BOOLEAN as "global_auto_approve",
                    NULL::TIMESTAMP WITH TIME ZONE as "global_approval_expires_at",
                    NULL::TEXT as "global_approval_notes"
                FROM mcp_tools_cache t
                JOIN mcp_servers s ON t.server_id = s.id
                LEFT JOIN user_group_mcp_servers ugms ON s.id = ugms.server_id
                LEFT JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
                WHERE t.tool_name = $1 AND (
                    -- User's own server
                    s.user_id = $2
                    -- OR accessible system server through group membership
                    OR (s.is_system = true AND ugm.user_id = $2)
                )
                ORDER BY s.is_system ASC -- Prefer user servers over system servers
                LIMIT 1
                "#,
                tool_name,
                user_id
            )
            .fetch_optional(pool)
            .await?
        }
    };

    Ok(tool)
}

/// Update tool usage statistics
pub async fn update_tool_usage(server_id: Uuid, tool_name: &str) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_tools_cache SET
            usage_count = usage_count + 1,
            last_used_at = NOW()
        WHERE server_id = $1 AND tool_name = $2
        "#,
        server_id,
        tool_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Clear tools cache for a server (when server is removed or restarted)
pub async fn clear_tools_cache_for_server(server_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!("DELETE FROM mcp_tools_cache WHERE server_id = $1", server_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get a specific tool by server_id and tool name
pub async fn get_tool_by_server_and_name(
    server_id: Uuid,
    tool_name: &str,
) -> Result<Option<MCPTool>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let tool = sqlx::query_as!(
        MCPTool,
        r#"
        SELECT
            id, server_id, tool_name, tool_description, input_schema,
            discovered_at, last_used_at, usage_count
        FROM mcp_tools_cache
        WHERE server_id = $1 AND tool_name = $2
        "#,
        server_id,
        tool_name
    )
    .fetch_optional(pool)
    .await?;

    Ok(tool)
}

/// Get tool statistics for admin dashboard
pub async fn get_tool_statistics() -> Result<Vec<(String, String, i32, i64)>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let stats = sqlx::query!(
        r#"
        SELECT
            t.tool_name,
            s.display_name as server_name,
            t.usage_count,
            COUNT(*) as instances
        FROM mcp_tools_cache t
        JOIN mcp_servers s ON t.server_id = s.id
        GROUP BY t.tool_name, s.display_name, t.usage_count
        ORDER BY t.usage_count DESC, t.tool_name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(stats
        .into_iter()
        .map(|row| {
            (
                row.tool_name,
                row.server_name,
                row.usage_count,
                row.instances.unwrap_or(0),
            )
        })
        .collect())
}