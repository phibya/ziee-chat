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
    models::mcp_tool::{MCPTool, MCPToolWithServer, SetToolGlobalApprovalRequest},
    queries::{mcp_tools, mcp_servers, mcp_tool_approvals},
};

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

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToolDiscoveryResponse {
    pub success: bool,
    pub message: String,
    pub tools_discovered: i32,
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

/// Get tools for a specific server
pub async fn get_server_tools(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<MCPTool>>, StatusCode> {
    // Check if user can access this server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to check server access: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }

    let tools = mcp_tools::get_cached_tools_for_server(server_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to get server tools: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(tools))
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

/// Trigger tool discovery for a server
pub async fn discover_server_tools(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> Result<Json<ToolDiscoveryResponse>, StatusCode> {
    // Check if user can access this server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to check server access: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }

    // TODO: Implement actual tool discovery from MCP server
    // This would involve:
    // 1. Connecting to the MCP server
    // 2. Sending a "tools/list" request
    // 3. Parsing the response and caching the tools
    // 4. Updating the server's tool count and discovery timestamp

    // For now, simulate tool discovery by clearing and re-discovering tools
    let _ = mcp_tools::clear_tools_cache_for_server(server_id).await;

    // Simulate discovering some tools (in real implementation, this would come from the MCP server)
    let mock_tools = vec![
        ("example_tool".to_string(), Some("An example tool".to_string()), serde_json::json!({"type": "object"}))
    ];

    let tools_count = mock_tools.len();

    // Cache the discovered tools
    if let Err(e) = mcp_tools::cache_discovered_tools(server_id, mock_tools).await {
        eprintln!("Failed to cache discovered tools: {}", e);
        return Ok(Json(ToolDiscoveryResponse {
            success: false,
            message: format!("Failed to cache tools: {}", e),
            tools_discovered: 0,
        }));
    }

    // Update server's tools discovered count
    let _ = mcp_servers::update_tools_discovered(server_id, tools_count as i32).await;

    Ok(Json(ToolDiscoveryResponse {
        success: true,
        message: format!("Discovered {} tools", tools_count),
        tools_discovered: tools_count as i32,
    }))
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