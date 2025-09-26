use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{
        CreateMCPServerRequest, CreateSystemMCPServerRequest, MCPServer, MCPServerStatus,
        MCPTransportType, UpdateMCPServerRequest,
    },
};

/// Create a new user MCP server
pub async fn create_user_mcp_server(
    user_id: Uuid,
    request: CreateMCPServerRequest,
) -> Result<MCPServer, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server = sqlx::query_as!(
        MCPServer,
        r#"
        INSERT INTO mcp_servers (
            user_id, name, display_name, description,
            transport_type, command, args, environment_variables,
            url, headers, timeout_seconds, max_restart_attempts, enabled, is_system
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, false)
        RETURNING
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        "#,
        user_id,
        request.name,
        request.display_name,
        request.description,
        request.transport_type as MCPTransportType,
        request.command,
        request.args.unwrap_or(serde_json::json!([])),
        request.environment_variables.unwrap_or(serde_json::json!({})),
        request.url,
        request.headers.unwrap_or(serde_json::json!({})),
        request.timeout_seconds,
        request.max_restart_attempts,
        request.enabled.unwrap_or(true)
    )
    .fetch_one(pool)
    .await?;

    Ok(server)
}

/// Create a new system MCP server (admin only)
pub async fn create_system_mcp_server(
    request: CreateSystemMCPServerRequest,
) -> Result<MCPServer, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server = sqlx::query_as!(
        MCPServer,
        r#"
        INSERT INTO mcp_servers (
            name, display_name, description,
            transport_type, command, args, environment_variables,
            url, headers, timeout_seconds, max_restart_attempts, enabled, is_system
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, true)
        RETURNING
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        "#,
        request.name,
        request.display_name,
        request.description,
        request.transport_type as MCPTransportType,
        request.command,
        request.args.unwrap_or(serde_json::json!([])),
        request.environment_variables.unwrap_or(serde_json::json!({})),
        request.url,
        request.headers.unwrap_or(serde_json::json!({})),
        request.timeout_seconds,
        request.max_restart_attempts,
        request.enabled.unwrap_or(true)
    )
    .fetch_one(pool)
    .await?;

    Ok(server)
}

/// Get MCP server by ID
pub async fn get_mcp_server_by_id(server_id: Uuid) -> Result<Option<MCPServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server = sqlx::query_as!(
        MCPServer,
        r#"
        SELECT
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        FROM mcp_servers
        WHERE id = $1
        "#,
        server_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(server)
}

/// List user's accessible MCP servers (user servers + accessible system servers)
pub async fn list_user_accessible_mcp_servers(user_id: Uuid) -> Result<Vec<MCPServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let servers = sqlx::query_as!(
        MCPServer,
        r#"
        SELECT DISTINCT
            s.id, s.user_id, s.name, s.display_name, s.description,
            s.enabled, s.is_system,
            s.transport_type as "transport_type: MCPTransportType",
            s.command, s.args, s.environment_variables, s.url, s.headers, s.timeout_seconds,
            s.status as "status: MCPServerStatus",
            s.is_active, s.last_health_check, s.restart_count, s.last_restart_at,
            s.max_restart_attempts, s.process_id, s.port,
            s.tools_discovered_at, s.tool_count,
            s.created_at, s.updated_at
        FROM mcp_servers s
        LEFT JOIN user_group_mcp_servers ugms ON s.id = ugms.server_id
        LEFT JOIN user_group_memberships ugm ON ugms.group_id = ugm.group_id
        WHERE
            -- User's own servers
            s.user_id = $1
            -- OR accessible system servers through group membership
            OR (s.is_system = true AND ugm.user_id = $1)
        ORDER BY s.is_system ASC, s.display_name ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(servers)
}

/// List all system MCP servers (admin only)
pub async fn list_system_mcp_servers() -> Result<Vec<MCPServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let servers = sqlx::query_as!(
        MCPServer,
        r#"
        SELECT
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        FROM mcp_servers
        WHERE is_system = true
        ORDER BY display_name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(servers)
}

/// Update MCP server
pub async fn update_mcp_server(
    server_id: Uuid,
    request: UpdateMCPServerRequest,
) -> Result<MCPServer, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let server = sqlx::query_as!(
        MCPServer,
        r#"
        UPDATE mcp_servers SET
            display_name = COALESCE($2, display_name),
            description = COALESCE($3, description),
            enabled = COALESCE($4, enabled),
            command = COALESCE($5, command),
            args = COALESCE($6, args),
            environment_variables = COALESCE($7, environment_variables),
            url = COALESCE($8, url),
            headers = COALESCE($9, headers),
            timeout_seconds = COALESCE($10, timeout_seconds),
            max_restart_attempts = COALESCE($11, max_restart_attempts),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        "#,
        server_id,
        request.display_name,
        request.description,
        request.enabled,
        request.command,
        request.args,
        request.environment_variables,
        request.url,
        request.headers,
        request.timeout_seconds,
        request.max_restart_attempts
    )
    .fetch_one(pool)
    .await?;

    Ok(server)
}

/// Delete MCP server
pub async fn delete_mcp_server(server_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!("DELETE FROM mcp_servers WHERE id = $1", server_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Update server status (for process management)
pub async fn update_server_status(
    server_id: Uuid,
    status: MCPServerStatus,
    is_active: bool,
    process_id: Option<i32>,
    port: Option<i32>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_servers SET
            status = $2,
            is_active = $3,
            process_id = $4,
            port = $5,
            last_health_check = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        server_id,
        status as MCPServerStatus,
        is_active,
        process_id,
        port
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update server restart count
pub async fn update_server_restart_count(
    server_id: Uuid,
    restart_count: i32,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_servers SET
            restart_count = $2,
            last_restart_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        server_id,
        restart_count
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update tool discovery timestamp and count
pub async fn update_tools_discovered(
    server_id: Uuid,
    tool_count: i32,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_servers SET
            tools_discovered_at = NOW(),
            tool_count = $2,
            updated_at = NOW()
        WHERE id = $1
        "#,
        server_id,
        tool_count
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if user can access server (owns it or has group access)
pub async fn can_user_access_server(user_id: Uuid, server_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check if user owns the server directly
    let server = sqlx::query!(
        "SELECT user_id, is_system FROM mcp_servers WHERE id = $1",
        server_id
    )
    .fetch_optional(pool)
    .await?;

    match server {
        Some(server) => {
            // If user owns the server directly, they have access
            if server.user_id == Some(user_id) {
                return Ok(true);
            }

            // If it's a system server, check group access
            if server.is_system {
                use crate::database::queries::user_group_mcp_servers;
                return user_group_mcp_servers::user_has_server_access(user_id, server_id).await;
            }

            Ok(false)
        }
        None => Ok(false), // Server doesn't exist
    }
}

/// Get all enabled MCP servers (for auto-start and health monitoring)
pub async fn get_all_enabled_mcp_servers() -> Result<Vec<MCPServer>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let servers = sqlx::query_as!(
        MCPServer,
        r#"
        SELECT
            id, user_id, name, display_name, description,
            enabled, is_system,
            transport_type as "transport_type: MCPTransportType",
            command, args, environment_variables, url, headers, timeout_seconds,
            status as "status: MCPServerStatus",
            is_active, last_health_check, restart_count, last_restart_at,
            max_restart_attempts, process_id, port,
            tools_discovered_at, tool_count,
            created_at, updated_at
        FROM mcp_servers
        WHERE enabled = true
        ORDER BY is_system DESC, display_name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(servers)
}

/// Update server runtime information (process ID, port, status)
pub async fn update_mcp_server_runtime_info(
    server_id: &Uuid,
    process_id: Option<i32>,
    port: Option<i32>,
    status: String,
    is_active: bool,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_servers SET
            process_id = $2,
            port = $3,
            status = $4,
            is_active = $5,
            last_health_check = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        server_id,
        process_id,
        port,
        status,
        is_active
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get server runtime information (process ID, port, status)
pub async fn get_mcp_server_runtime_info(
    server_id: &Uuid,
) -> Result<Option<(Option<i32>, Option<i32>, String)>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        r#"
        SELECT process_id, port, status
        FROM mcp_servers
        WHERE id = $1
        "#,
        server_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| (row.process_id, row.port, row.status)))
}