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
    permissions::{check_permission, Permission},
};
use crate::database::{
    models::mcp_tool::{MCPToolWithServer, MCPToolWithApproval, SetToolGlobalApprovalRequest},
    queries::{mcp_tools, mcp_servers, mcp_tool_approvals},
};
use crate::ai::mcp::tool_discovery::{discover_and_cache_tools_http, should_rediscover_tools};


// Request/Response types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListToolsQuery {
    pub server_id: Option<Uuid>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListToolsResponse {
    pub tools: Vec<MCPToolWithServer>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}



/// List user's accessible tools
#[debug_handler]
pub async fn list_user_tools(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListToolsQuery>,
) -> ApiResult<Json<ListToolsResponse>> {
    let mut tools = match mcp_tools::get_user_accessible_tools(auth_user.user_id).await {
        Ok(tools) => tools,
        Err(e) => {
            tracing::error!("Failed to load user tools: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")));
        }
    };

    // Apply server filter if provided
    if let Some(server_id) = query.server_id {
        tools = tools.into_iter()
            .filter(|t| t.server_id == server_id)
            .collect();
    }

    // Apply search filter if provided
    if let Some(search_term) = &query.search {
        let search_lower = search_term.to_lowercase();
        tools = tools.into_iter()
            .filter(|t| {
                t.tool_name.to_lowercase().contains(&search_lower) ||
                t.tool_description.as_ref().map_or(false, |d| d.to_lowercase().contains(&search_lower)) ||
                t.server_display_name.to_lowercase().contains(&search_lower)
            })
            .collect();
    }

    // Apply pagination
    let total = tools.len();
    let page = query.page.unwrap_or(1).max(1) as usize;
    let per_page = query.per_page.unwrap_or(50).max(1).min(1000) as usize;

    let start = (page - 1) * per_page;
    let end = (start + per_page).min(tools.len());
    let paginated_tools = if start < tools.len() {
        tools[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok((StatusCode::OK, Json(ListToolsResponse {
        tools: paginated_tools,
        total: total as i32,
        page: page as i32,
        per_page: per_page as i32,
    })))
}

/// Get tools for a specific server (with auto-discovery and caching)
pub async fn get_server_tools(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<MCPToolWithApproval>>, StatusCode> {
    // Get server details first to check if it's a system server
    let server = match mcp_servers::get_mcp_server_by_id(server_id).await {
        Ok(Some(server)) => server,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get server: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check permissions based on server type
    if server.is_system {
        // For system servers, check if user has admin MCP read permission
        if !check_permission(&auth_user.user, Permission::McpAdminServersRead.as_str()) {
            return Err(StatusCode::FORBIDDEN);
        }
    } else {
        // For user servers, check if user can access this server
        let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check server access: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !can_access {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check if we should discover/rediscover tools (cache older than 10 minutes)
    let should_discover = should_rediscover_tools(server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check if tools should be rediscovered: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if should_discover {
        // Attempt discovery, but don't fail if it doesn't work
        if let Err(e) = discover_and_cache_tools_http(server_id).await {
            tracing::warn!("Tool discovery failed for server {}: {}", server_id, e);

            // Check if we have any cached tools to fall back to
            let cached_tools = mcp_tools::get_cached_tools_for_server(server_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get cached tools: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            if cached_tools.is_empty() {
                // No cached tools available, return service unavailable
                tracing::error!("No cached tools available and discovery failed for server {}", server_id);
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
            // Continue with cached tools
        }
    }

    // Get tools from cache
    let tools = mcp_tools::get_cached_tools_for_server(server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get server tools: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get approval status for each tool for this user
    let mut tools_with_approval = Vec::new();

    for tool in tools {
        // Check global approval for this tool (since we're not in conversation context)
        let global_approval = mcp_tool_approvals::get_global_tool_approval(
            auth_user.user_id,
            server_id,
            &tool.tool_name,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get global tool approval: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let (is_auto_approved, approval_source, approval_expires_at) = if let Some(approval) = global_approval {
            // Check if the approval is still valid (not expired)
            let is_expired = approval.expires_at.map_or(false, |exp| exp <= chrono::Utc::now());

            if approval.approved && approval.auto_approve && !is_expired {
                (true, Some("global".to_string()), approval.expires_at)
            } else {
                (false, None, None)
            }
        } else {
            (false, None, None)
        };

        tools_with_approval.push(MCPToolWithApproval {
            id: tool.id,
            server_id: tool.server_id,
            tool_name: tool.tool_name,
            tool_description: tool.tool_description,
            input_schema: tool.input_schema,
            discovered_at: tool.discovered_at,
            last_used_at: tool.last_used_at,
            usage_count: tool.usage_count,
            is_auto_approved,
            approval_source,
            approval_expires_at,
        });
    }

    Ok(Json(tools_with_approval))
}

/// Find tool by name for user (with conflict resolution)
#[debug_handler]
pub async fn find_tool_by_name(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(tool_name): Path<String>,
    Query(query): Query<FindToolQuery>,
) -> ApiResult<Json<Option<MCPToolWithServer>>> {
    let tool = mcp_tools::find_tool_by_name_for_user(
        auth_user.user_id,
        &tool_name,
        query.server_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to find tool: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    Ok((StatusCode::OK, Json(tool)))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindToolQuery {
    pub server_id: Option<Uuid>,
}


/// Set global auto-approve for a tool
#[debug_handler]
pub async fn set_tool_global_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((server_id, tool_name)): Path<(Uuid, String)>,
    Json(request): Json<SetToolGlobalApprovalRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check if user can access this server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    // Set global approval
    let approval = mcp_tool_approvals::set_global_tool_approval(
        auth_user.user_id,
        server_id,
        &tool_name,
        request.auto_approve,
        request.expires_at,
        request.notes,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to set global tool approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": if request.auto_approve { "Global auto-approve enabled" } else { "Global auto-approve disabled" },
        "approval": approval
    }))))
}

/// Remove global auto-approve for a tool
#[debug_handler]
pub async fn remove_tool_global_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((server_id, tool_name)): Path<(Uuid, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check if user can access this server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    // Remove global approval
    let deleted = mcp_tool_approvals::delete_global_tool_approval(
        auth_user.user_id,
        server_id,
        &tool_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to remove global tool approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    if deleted {
        Ok((StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "Global auto-approve removed"
        }))))
    } else {
        Err((StatusCode::NOT_FOUND, AppError::not_found("Global approval not found")))
    }
}


/// Get tool usage statistics (admin only)
pub async fn get_tool_statistics(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<(String, String, i32, i64)>>, StatusCode> {
    // TODO: Implement proper admin permission checking
    // For now, allow all authenticated users to access admin functions
    // This should be restricted using proper middleware like other admin routes

    let stats = mcp_tools::get_tool_statistics()
        .await
        .map_err(|e| {
            eprintln!("Failed to get tool statistics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(stats))
}