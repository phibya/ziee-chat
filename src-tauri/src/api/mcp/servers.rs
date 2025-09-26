use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    Extension, Json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError},
    middleware::AuthenticatedUser,
};
use crate::database::{
    models::mcp_server::{MCPServer, CreateMCPServerRequest, CreateSystemMCPServerRequest, UpdateMCPServerRequest},
    queries::{mcp_servers, user_group_mcp_servers},
};
use crate::mcp::{start_mcp_server, stop_mcp_server};

// Request/Response types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListServersQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListServersResponse {
    pub servers: Vec<MCPServer>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}


#[derive(Debug, Serialize, JsonSchema)]
pub struct ServerActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSystemServersQuery {
    pub enabled: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GroupAssignmentResponse {
    pub group_id: Uuid,
    pub server_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AssignServersRequest {
    pub server_ids: Vec<Uuid>,
}

// User MCP Server Operations

/// List user's accessible MCP servers
#[debug_handler]
pub async fn list_user_servers(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListServersQuery>,
) -> ApiResult<Json<ListServersResponse>> {
    let mut servers = match mcp_servers::list_user_accessible_mcp_servers(auth_user.user_id).await {
        Ok(servers) => servers,
        Err(e) => {
            tracing::error!("Failed to load user MCP servers: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };


    // Apply status filtering if provided
    if let Some(status_filter) = &query.status {
        servers = servers.into_iter()
            .filter(|s| s.status.to_string().to_lowercase() == status_filter.to_lowercase())
            .collect::<Vec<_>>();
    }

    // Apply pagination
    let total = servers.len();
    let page = query.page.unwrap_or(1).max(1) as usize;
    let per_page = query.per_page.unwrap_or(50).max(1).min(1000) as usize;

    let start = (page - 1) * per_page;
    let end = (start + per_page).min(servers.len());
    let paginated_servers = if start < servers.len() {
        servers[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok((StatusCode::OK, Json(ListServersResponse {
        servers: paginated_servers,
        total: total as i32,
        page: page as i32,
        per_page: per_page as i32,
    })))
}

/// Get a specific server by ID
#[debug_handler]
pub async fn get_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Json<MCPServer>> {
    // Check if user can access this server
    let can_access = match mcp_servers::can_user_access_server(auth_user.user_id, server_id).await {
        Ok(access) => access,
        Err(e) => {
            tracing::error!("Failed to check server access: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    match mcp_servers::get_mcp_server_by_id(server_id).await {
        Ok(Some(server)) => Ok((StatusCode::OK, Json(server))),
        Ok(None) => Err((StatusCode::NOT_FOUND, AppError::not_found("MCP Server"))),
        Err(e) => {
            tracing::error!("Failed to get server: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}

/// Create a new user MCP server
#[debug_handler]
pub async fn create_user_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateMCPServerRequest>,
) -> ApiResult<Json<MCPServer>> {
    tracing::info!("create_user_server called for user: {}", auth_user.user_id);
    tracing::info!("Request data: name={}, transport_type={:?}", request.name, request.transport_type);

    match mcp_servers::create_user_mcp_server(auth_user.user_id, request).await {
        Ok(server) => Ok((StatusCode::CREATED, Json(server))),
        Err(e) => {
            tracing::error!("Failed to create MCP server: {}", e);
            match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    Err((StatusCode::CONFLICT, AppError::conflict("Server with this name already exists")))
                }
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
            }
        }
    }
}

/// Update a user MCP server
#[debug_handler]
pub async fn update_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
    Json(request): Json<UpdateMCPServerRequest>,
) -> ApiResult<Json<MCPServer>> {
    // Check if user owns this server
    let server = match mcp_servers::get_mcp_server_by_id(server_id).await {
        Ok(Some(server)) => server,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("MCP Server"))),
        Err(e) => {
            tracing::error!("Failed to get server: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if server.user_id != Some(auth_user.user_id) || server.is_system {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Cannot update this server")));
    }

    match mcp_servers::update_mcp_server(server_id, request).await {
        Ok(updated_server) => Ok((StatusCode::OK, Json(updated_server))),
        Err(e) => {
            tracing::error!("Failed to update MCP server: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}

/// Delete a user MCP server
#[debug_handler]
pub async fn delete_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Check if user owns this server
    let server = match mcp_servers::get_mcp_server_by_id(server_id).await {
        Ok(Some(server)) => server,
        Ok(None) => return Err((StatusCode::NOT_FOUND, AppError::not_found("MCP Server"))),
        Err(e) => {
            tracing::error!("Failed to get server: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if server.user_id != Some(auth_user.user_id) || server.is_system {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Cannot delete this server")));
    }

    // Stop server if running
    if server.is_active {
        if let Err(e) = stop_mcp_server(&server_id).await {
            tracing::warn!("Failed to stop server before deletion: {}", e);
        }
    }

    // First remove all group assignments for this server
    if let Err(e) = user_group_mcp_servers::remove_all_server_assignments(server_id).await {
        tracing::warn!("Failed to remove server assignments during deletion: {}", e);
    }

    match mcp_servers::delete_mcp_server(server_id).await {
        Ok(()) => Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT)),
        Err(e) => {
            tracing::error!("Failed to delete MCP server: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}

/// Start an MCP server
#[debug_handler]
pub async fn start_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Json<ServerActionResponse>> {
    // Check if user can access this server
    let can_access = match mcp_servers::can_user_access_server(auth_user.user_id, server_id).await {
        Ok(access) => access,
        Err(e) => {
            tracing::error!("Failed to check server access: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    match start_mcp_server(&server_id).await {
        Ok(_result) => Ok((StatusCode::OK, Json(ServerActionResponse {
            success: true,
            message: "Server start initiated".to_string(),
        }))),
        Err(e) => {
            tracing::error!("Failed to start MCP server: {}", e);
            Ok((StatusCode::OK, Json(ServerActionResponse {
                success: false,
                message: format!("Failed to start server: {}", e),
            })))
        }
    }
}

/// Stop an MCP server
#[debug_handler]
pub async fn stop_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Json<ServerActionResponse>> {
    // Check if user can access this server
    let can_access = match mcp_servers::can_user_access_server(auth_user.user_id, server_id).await {
        Ok(access) => access,
        Err(e) => {
            tracing::error!("Failed to check server access: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    match stop_mcp_server(&server_id).await {
        Ok(_) => Ok((StatusCode::OK, Json(ServerActionResponse {
            success: true,
            message: "Server stopped".to_string(),
        }))),
        Err(e) => {
            tracing::error!("Failed to stop MCP server: {}", e);
            Ok((StatusCode::OK, Json(ServerActionResponse {
                success: false,
                message: format!("Failed to stop server: {}", e),
            })))
        }
    }
}

/// Restart an MCP server
#[debug_handler]
pub async fn restart_server(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Json<ServerActionResponse>> {
    // Check if user can access this server
    let can_access = match mcp_servers::can_user_access_server(auth_user.user_id, server_id).await {
        Ok(access) => access,
        Err(e) => {
            tracing::error!("Failed to check server access: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    // Stop then start
    if let Err(e) = stop_mcp_server(&server_id).await {
        tracing::warn!("Failed to stop server during restart: {}", e);
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    match start_mcp_server(&server_id).await {
        Ok(_result) => Ok((StatusCode::OK, Json(ServerActionResponse {
            success: true,
            message: "Server restarted".to_string(),
        }))),
        Err(e) => {
            tracing::error!("Failed to restart MCP server: {}", e);
            Ok((StatusCode::OK, Json(ServerActionResponse {
                success: false,
                message: format!("Failed to restart server: {}", e),
            })))
        }
    }
}


// Admin MCP Server Operations

/// List all system MCP servers (admin only)
#[debug_handler]
pub async fn list_system_servers(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListSystemServersQuery>,
) -> ApiResult<Json<ListServersResponse>> {
    let mut servers = match mcp_servers::list_system_mcp_servers().await {
        Ok(servers) => servers,
        Err(e) => {
            tracing::error!("Failed to load system MCP servers: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    // Apply enabled filtering if provided
    if let Some(enabled) = query.enabled {
        servers = servers.into_iter()
            .filter(|s| s.enabled == enabled)
            .collect::<Vec<_>>();
    }

    // Apply pagination
    let total = servers.len();
    let page = query.page.unwrap_or(1).max(1) as usize;
    let per_page = query.per_page.unwrap_or(50).max(1).min(1000) as usize;

    let start = (page - 1) * per_page;
    let end = (start + per_page).min(servers.len());
    let paginated_servers = if start < servers.len() {
        servers[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok((StatusCode::OK, Json(ListServersResponse {
        servers: paginated_servers,
        total: total as i32,
        page: page as i32,
        per_page: per_page as i32,
    })))
}

/// Create a new system MCP server (admin only)
#[debug_handler]
pub async fn create_system_server(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateSystemMCPServerRequest>,
) -> ApiResult<Json<MCPServer>> {
    match mcp_servers::create_system_mcp_server(request).await {
        Ok(server) => Ok((StatusCode::CREATED, Json(server))),
        Err(e) => {
            tracing::error!("Failed to create system MCP server: {}", e);
            match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    Err((StatusCode::CONFLICT, AppError::conflict("Server with this name already exists")))
                }
                _ => Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
            }
        }
    }
}

// Group Assignment Operations

/// Get MCP servers assigned to group (admin only)
#[debug_handler]
pub async fn get_group_servers(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Uuid>>> {
    let server_ids = match user_group_mcp_servers::get_group_mcp_servers(group_id).await {
        Ok(server_ids) => server_ids,
        Err(e) => {
            tracing::error!("Failed to load group MCP servers: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    Ok((StatusCode::OK, Json(server_ids)))
}

/// Assign MCP servers to group (admin only)
#[debug_handler]
pub async fn assign_servers_to_group(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AssignServersRequest>,
) -> ApiResult<Json<GroupAssignmentResponse>> {
    let _assignments = match user_group_mcp_servers::assign_multiple_servers_to_group(
        group_id,
        request.server_ids.clone(),
        auth_user.user_id,
    ).await {
        Ok(assignments) => assignments,
        Err(e) => {
            tracing::error!("Failed to assign servers to group: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    Ok((StatusCode::OK, Json(GroupAssignmentResponse {
        group_id,
        server_ids: request.server_ids,
    })))
}

/// Remove server from group (admin only)
#[debug_handler]
pub async fn remove_server_from_group(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path((group_id, server_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let removed = match user_group_mcp_servers::remove_server_from_group(group_id, server_id).await {
        Ok(removed) => removed,
        Err(e) => {
            tracing::error!("Failed to remove server from group: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    if removed {
        Ok((StatusCode::NO_CONTENT, StatusCode::NO_CONTENT))
    } else {
        Err((StatusCode::NOT_FOUND, AppError::not_found("Assignment not found")))
    }
}

/// List all group-server assignments (admin only)
#[debug_handler]
pub async fn list_all_group_assignments(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<crate::database::models::user_group_mcp_server::GroupServerAssignmentResponse>>> {
    match user_group_mcp_servers::list_all_group_server_assignments().await {
        Ok(assignments) => {
            Ok((StatusCode::OK, Json(assignments)))
        }
        Err(e) => {
            tracing::error!("Failed to list group assignments: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}
/// Get servers assigned to current user through group memberships
#[debug_handler]
pub async fn get_user_assigned_servers(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<Vec<Uuid>>> {
    match user_group_mcp_servers::get_user_assigned_servers(auth_user.user_id).await {
        Ok(server_ids) => {
            Ok((StatusCode::OK, Json(server_ids)))
        }
        Err(e) => {
            tracing::error!("Failed to get user assigned servers: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}

/// Get groups that have access to a specific server (admin only)
#[debug_handler]
pub async fn get_server_access_groups(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Uuid>>> {
    match user_group_mcp_servers::get_groups_with_server_access(server_id).await {
        Ok(group_ids) => {
            Ok((StatusCode::OK, Json(group_ids)))
        }
        Err(e) => {
            tracing::error!("Failed to get server access groups: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")))
        }
    }
}
