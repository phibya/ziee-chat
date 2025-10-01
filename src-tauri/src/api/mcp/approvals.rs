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
    models::mcp_tool::{
        CreateConversationApprovalRequest, ToolApprovalResponse, ListConversationApprovalsQuery,
        UpdateToolApprovalRequest,
    },
    queries::{mcp_tool_approvals, mcp_servers},
};

// Response types
#[derive(Debug, Serialize, JsonSchema)]
pub struct ApprovalCheckResponse {
    pub approved: bool,
    pub source: Option<String>, // "global" or "conversation"
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SimpleResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteConversationApprovalQuery {
    pub server_id: Uuid,
    pub tool_name: String,
}

/// List approvals for specific conversation
#[debug_handler]
pub async fn list_conversation_approvals(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<ListConversationApprovalsQuery>,
) -> ApiResult<Json<Vec<ToolApprovalResponse>>> {
    let approvals = mcp_tool_approvals::list_conversation_tool_approvals(
        auth_user.user_id,
        conversation_id,
        query,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list conversation approvals: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    Ok((StatusCode::OK, Json(approvals)))
}

/// Create or update approval for specific conversation
#[debug_handler]
pub async fn create_conversation_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<CreateConversationApprovalRequest>,
) -> ApiResult<Json<ToolApprovalResponse>> {
    // Verify user has access to the server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, request.server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    // Update message content if approval_message_content_id is provided
    if let Some(content_id) = request.approval_message_content_id {
        use crate::database::queries;

        // Update the message content to set is_approved field
        sqlx::query!(
            r#"
            UPDATE message_contents
            SET content = jsonb_set(content, '{is_approved}', $2::jsonb, true),
                updated_at = NOW()
            WHERE id = $1
            "#,
            content_id,
            serde_json::json!(request.approved)
        )
        .execute(queries::get_database_pool().map_err(|e| {
            tracing::error!("Failed to get database pool: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update message content: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;
    }

    // Create the approval
    let approval = mcp_tool_approvals::create_conversation_tool_approval(
        auth_user.user_id,
        conversation_id,
        request,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create conversation approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    // Get server name for response
    let server = mcp_servers::get_mcp_server_by_id(approval.server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get server info: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Server not found")))?;

    let response = ToolApprovalResponse {
        id: approval.id,
        user_id: approval.user_id,
        conversation_id: approval.conversation_id,
        server_id: approval.server_id,
        server_name: server.display_name,
        tool_name: approval.tool_name,
        approved: approval.approved,
        auto_approve: approval.auto_approve,
        is_global: approval.is_global,
        approved_at: approval.approved_at,
        expires_at: approval.expires_at,
        is_expired: approval.expires_at.map_or(false, |exp| exp <= chrono::Utc::now()),
        notes: approval.notes,
        created_at: approval.created_at,
        updated_at: approval.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Delete conversation approval
#[debug_handler]
pub async fn delete_conversation_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<DeleteConversationApprovalQuery>,
) -> ApiResult<Json<SimpleResponse>> {
    // Verify user has access to the server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, query.server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    let deleted = mcp_tool_approvals::delete_conversation_tool_approval(
        auth_user.user_id,
        conversation_id,
        query.server_id,
        &query.tool_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete conversation approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    if deleted {
        Ok((StatusCode::OK, Json(SimpleResponse {
            success: true,
            message: "Approval deleted successfully".to_string(),
        })))
    } else {
        Err((StatusCode::NOT_FOUND, AppError::not_found("Approval not found")))
    }
}

/// Check if tool is approved for conversation
#[debug_handler]
pub async fn check_conversation_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<DeleteConversationApprovalQuery>, // Reuse for server_id and tool_name
) -> ApiResult<Json<ApprovalCheckResponse>> {
    // Verify user has access to the server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, query.server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    let approval_result = mcp_tool_approvals::check_tool_approval(
        auth_user.user_id,
        conversation_id,
        query.server_id,
        &query.tool_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to check tool approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?;

    let response = if let Some((approved, source)) = approval_result {
        ApprovalCheckResponse {
            approved,
            source: Some(source),
            expires_at: None, // TODO: Add expiration info if needed
        }
    } else {
        ApprovalCheckResponse {
            approved: false,
            source: None,
            expires_at: None,
        }
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update an existing tool approval
#[debug_handler]
pub async fn update_tool_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(approval_id): Path<Uuid>,
    Json(request): Json<UpdateToolApprovalRequest>,
) -> ApiResult<Json<ToolApprovalResponse>> {
    let updated_approval = mcp_tool_approvals::update_tool_approval(
        auth_user.user_id,
        approval_id,
        request,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to update tool approval: {}", e);
        match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, AppError::not_found("Approval not found")),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error")),
        }
    })?;

    // Get server name for response
    let server = mcp_servers::get_mcp_server_by_id(updated_approval.server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get server info: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Server not found")))?;

    let response = ToolApprovalResponse {
        id: updated_approval.id,
        user_id: updated_approval.user_id,
        conversation_id: updated_approval.conversation_id,
        server_id: updated_approval.server_id,
        server_name: server.display_name,
        tool_name: updated_approval.tool_name,
        approved: updated_approval.approved,
        auto_approve: updated_approval.auto_approve,
        is_global: updated_approval.is_global,
        approved_at: updated_approval.approved_at,
        expires_at: updated_approval.expires_at,
        is_expired: updated_approval.expires_at.map_or(false, |exp| exp <= chrono::Utc::now()),
        notes: updated_approval.notes,
        created_at: updated_approval.created_at,
        updated_at: updated_approval.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get global tool approval for specific server/tool
#[debug_handler]
pub async fn get_global_tool_approval(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((server_id, tool_name)): Path<(Uuid, String)>,
) -> ApiResult<Json<ToolApprovalResponse>> {
    // Verify user has access to the server
    let can_access = mcp_servers::can_user_access_server(auth_user.user_id, server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check server access: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    if !can_access {
        return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to MCP server")));
    }

    let approval = mcp_tool_approvals::get_global_tool_approval(
        auth_user.user_id,
        server_id,
        &tool_name,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get global tool approval: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
    })?
    .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Global approval not found")))?;

    // Get server name for response
    let server = mcp_servers::get_mcp_server_by_id(server_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get server info: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?
        .ok_or((StatusCode::NOT_FOUND, AppError::not_found("Server not found")))?;

    let response = ToolApprovalResponse {
        id: approval.id,
        user_id: approval.user_id,
        conversation_id: approval.conversation_id,
        server_id: approval.server_id,
        server_name: server.display_name,
        tool_name: approval.tool_name,
        approved: approval.approved,
        auto_approve: approval.auto_approve,
        is_global: approval.is_global,
        approved_at: approval.approved_at,
        expires_at: approval.expires_at,
        is_expired: approval.expires_at.map_or(false, |exp| exp <= chrono::Utc::now()),
        notes: approval.notes,
        created_at: approval.created_at,
        updated_at: approval.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Clean expired tool approvals (admin maintenance function)
#[debug_handler]
pub async fn clean_expired_approvals(
    Extension(_auth_user): Extension<AuthenticatedUser>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Add admin permission check
    let cleaned_count = mcp_tool_approvals::clean_expired_tool_approvals()
        .await
        .map_err(|e| {
            tracing::error!("Failed to clean expired approvals: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, AppError::internal_error("Database error"))
        })?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": format!("Cleaned {} expired approvals", cleaned_count),
        "cleaned_count": cleaned_count
    }))))
}