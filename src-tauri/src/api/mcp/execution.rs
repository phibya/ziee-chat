use axum::{
    debug_handler,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::api::{
    errors::{ApiResult, AppError},
    middleware::AuthenticatedUser,
};
use crate::database::{
    models::mcp_tool::{ExecuteToolRequest, MCPExecutionLog, MCPExecutionStatus, ToolExecutionResponse},
    queries::{mcp_execution_logs, mcp_servers, mcp_tools, mcp_tool_approvals},
};

// Request/Response types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListExecutionLogsQuery {
    pub server_id: Option<Uuid>,
    pub thread_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListExecutionLogsResponse {
    pub logs: Vec<MCPExecutionLog>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CancelExecutionRequest {
    pub reason: Option<String>,
}

/// Execute a tool
pub async fn execute_tool(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<ExecuteToolRequest>,
) -> Result<Json<ToolExecutionResponse>, StatusCode> {
    // Find the tool for this user
    let tool = mcp_tools::find_tool_by_name_for_user(
        auth_user.user_id,
        &request.tool_name,
        request.server_id,
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to find tool: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check if user can access the server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, tool.server_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to check server access: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !can_access {
        return Err(StatusCode::FORBIDDEN);
    }

    // Check tool approvals (global + conversation-specific)
    if let Some(conversation_id) = request.conversation_id {
        // Check if tool is approved for this conversation/thread
        let approval_check = mcp_tool_approvals::check_tool_approval(
            auth_user.user_id,
            conversation_id,
            tool.server_id,
            &tool.tool_name,
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to check tool approval: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // If not approved and doesn't require special approval, check if tool typically needs approval
        let requires_approval = should_require_approval(&tool.tool_name);
        let is_approved = approval_check.as_ref().map_or(false, |(approved, _)| *approved);

        if requires_approval && !is_approved && !request.auto_approve.unwrap_or(false) {
            // Tool requires approval but is not approved
            return Ok(Json(ToolExecutionResponse {
                execution_id: Uuid::new_v4(), // Placeholder ID
                status: MCPExecutionStatus::Failed,
                result: None,
                error_message: Some("Tool execution requires approval. Use conversation approvals or set global auto-approve.".to_string()),
                duration_ms: Some(0),
            }));
        }

        // Log approval status for debugging
        if let Some((approved, source)) = &approval_check {
            tracing::info!(
                "Tool {} execution approved via {}: {}",
                tool.tool_name, source, approved
            );
        }
    } else {
        // No conversation_id provided, check if tool generally requires approval
        let requires_approval = should_require_approval(&tool.tool_name);
        if requires_approval && !request.auto_approve.unwrap_or(false) {
            // For tools without conversation context, require explicit auto_approve flag
            return Ok(Json(ToolExecutionResponse {
                execution_id: Uuid::new_v4(), // Placeholder ID
                status: MCPExecutionStatus::Failed,
                result: None,
                error_message: Some("Tool execution requires approval. Provide conversation_id for conversation-scoped approval or use auto_approve flag.".to_string()),
                duration_ms: Some(0),
            }));
        }
    }

    // Create execution log
    let execution_id = mcp_execution_logs::create_execution_log(
        auth_user.user_id,
        tool.server_id,
        request.conversation_id,
        request.tool_name.clone(),
        Some(request.parameters.clone()),
        None, // request_id - could be generated if needed
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to create execution log: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Implement actual tool execution
    // This would involve:
    // 1. Connecting to the MCP server
    // 2. Sending the tool execution request
    // 3. Handling the response (success/failure)
    // 4. Updating the execution log with results
    // 5. Updating tool usage statistics

    // For now, return a placeholder response
    let response = ToolExecutionResponse {
        execution_id,
        status: MCPExecutionStatus::Failed,
        result: None,
        error_message: Some("Tool execution not yet implemented".to_string()),
        duration_ms: Some(0),
    };

    // Update execution log with failure
    let _ = mcp_execution_logs::complete_execution_log(
        execution_id,
        MCPExecutionStatus::Failed,
        None,
        Some("Tool execution not yet implemented".to_string()),
        Some("NOT_IMPLEMENTED".to_string()),
        Some(0),
    ).await;

    // Update tool usage statistics
    let _ = mcp_tools::update_tool_usage(tool.server_id, &tool.tool_name).await;

    Ok(Json(response))
}

/// Get execution log by ID
pub async fn get_execution_log(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<MCPExecutionLog>, StatusCode> {
    let log = mcp_execution_logs::get_execution_log(execution_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to get execution log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if user owns this execution log
    if log.user_id != auth_user.user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(log))
}

/// List user's execution logs
#[debug_handler]
pub async fn list_user_execution_logs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListExecutionLogsQuery>,
) -> ApiResult<Json<ListExecutionLogsResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
    let limit = Some(per_page as i64);
    let offset = Some(((page - 1) * per_page) as i64);

    let logs = mcp_execution_logs::list_user_execution_logs(
        auth_user.user_id,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list execution logs: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    // Apply additional filters
    let mut filtered_logs = logs;

    if let Some(server_id) = query.server_id {
        filtered_logs = filtered_logs.into_iter()
            .filter(|log| log.server_id == server_id)
            .collect();
    }

    if let Some(thread_id) = query.thread_id {
        filtered_logs = filtered_logs.into_iter()
            .filter(|log| log.thread_id == Some(thread_id))
            .collect();
    }

    if let Some(status_filter) = &query.status {
        let status_lower = status_filter.to_lowercase();
        filtered_logs = filtered_logs.into_iter()
            .filter(|log| {
                match &log.status {
                    MCPExecutionStatus::Pending => "pending" == status_lower,
                    MCPExecutionStatus::Running => "running" == status_lower,
                    MCPExecutionStatus::Completed => "completed" == status_lower,
                    MCPExecutionStatus::Failed => "failed" == status_lower,
                    MCPExecutionStatus::Cancelled => "cancelled" == status_lower,
                    MCPExecutionStatus::Timeout => "timeout" == status_lower,
                }
            })
            .collect();
    }

    let total = filtered_logs.len() as i32;

    Ok((StatusCode::OK, Json(ListExecutionLogsResponse {
        logs: filtered_logs,
        total,
        page,
        per_page,
    })))
}

/// List execution logs for a specific thread
pub async fn list_thread_execution_logs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<MCPExecutionLog>>, StatusCode> {
    let logs = mcp_execution_logs::list_thread_execution_logs(thread_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to list thread execution logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Filter to only logs owned by this user
    let user_logs = logs.into_iter()
        .filter(|log| log.user_id == auth_user.user_id)
        .collect();

    Ok(Json(user_logs))
}

/// Cancel a tool execution
#[debug_handler]
pub async fn cancel_execution(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(execution_id): Path<Uuid>,
    Json(request): Json<CancelExecutionRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get the execution log
    let log = mcp_execution_logs::get_execution_log(execution_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get execution log: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Execution log not found")))?;

    // Check if user owns this execution
    if log.user_id != auth_user.user_id {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied")));
    }

    // Check if execution can be cancelled
    match log.status {
        MCPExecutionStatus::Pending | MCPExecutionStatus::Running => {
            // TODO: Implement actual cancellation logic
            // This would involve sending a cancellation request to the MCP server

            // Update execution log to cancelled
            let cancel_reason = request.reason.unwrap_or_else(|| "Cancelled by user".to_string());
            mcp_execution_logs::complete_execution_log(
                execution_id,
                MCPExecutionStatus::Cancelled,
                None,
                Some(cancel_reason),
                Some("USER_CANCELLED".to_string()),
                None,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to update execution log: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
            })?;

            Ok((StatusCode::OK, Json(serde_json::json!({"message": "Execution cancelled successfully"}))))
        }
        _ => {
            // Execution already completed, failed, cancelled, or timed out
            Err((StatusCode::CONFLICT, AppError::conflict("Execution cannot be cancelled")))
        }
    }
}

/// Get execution statistics (admin only)
pub async fn get_execution_statistics(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement proper admin permission checking
    // For now, allow all authenticated users to access admin functions
    // This should be restricted using proper middleware like other admin routes

    let stats = mcp_execution_logs::get_execution_statistics()
        .await
        .map_err(|e| {
            eprintln!("Failed to get execution statistics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(stats))
}

/// List all execution logs (admin only)
#[debug_handler]
pub async fn list_all_execution_logs(
    Extension(_auth_user): Extension<AuthenticatedUser>,
    Query(query): Query<ListExecutionLogsQuery>,
) -> ApiResult<Json<ListExecutionLogsResponse>> {
    // TODO: Implement proper admin permission checking
    // For now, allow all authenticated users to access admin functions
    // This should be restricted using proper middleware like other admin routes

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
    let limit = Some(per_page as i64);
    let offset = Some(((page - 1) * per_page) as i64);

    // Parse status filter
    let status_filter = query.status.as_ref().and_then(|s| {
        match s.to_lowercase().as_str() {
            "pending" => Some(MCPExecutionStatus::Pending),
            "running" => Some(MCPExecutionStatus::Running),
            "completed" => Some(MCPExecutionStatus::Completed),
            "failed" => Some(MCPExecutionStatus::Failed),
            "cancelled" => Some(MCPExecutionStatus::Cancelled),
            "timeout" => Some(MCPExecutionStatus::Timeout),
            _ => None,
        }
    });

    let logs = mcp_execution_logs::list_all_execution_logs(
        limit,
        offset,
        status_filter,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list all execution logs: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    let total = logs.len() as i32;

    Ok((StatusCode::OK, Json(ListExecutionLogsResponse {
        logs,
        total,
        page,
        per_page,
    })))
}

/// Determine if a tool requires approval before execution
/// This is a placeholder implementation - in a full system this would check
/// tool configuration, user permissions, or security policies
fn should_require_approval(tool_name: &str) -> bool {
    // For now, consider potentially dangerous tools that might need approval
    match tool_name.to_lowercase().as_str() {
        // File system operations
        "delete_file" | "rm" | "remove" => true,
        "write_file" | "create_file" => true,
        // Network operations
        "http_request" | "fetch" | "curl" => true,
        // Shell/system operations
        "shell_execute" | "exec" | "run_command" => true,
        // Database operations
        "execute_sql" | "db_query" | "database_write" => true,
        // By default, don't require approval for read-only or safe operations
        _ => false,
    }
}