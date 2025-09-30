use chrono::Utc;
use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::mcp_tool::{
        MCPToolApproval, ToolApprovalResponse, CreateConversationApprovalRequest,
        SetToolGlobalApprovalRequest, ListConversationApprovalsQuery, UpdateToolApprovalRequest,
    },
};

/// Create or update a global tool approval (auto_approve setting)
pub async fn create_global_tool_approval(
    user_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
    request: SetToolGlobalApprovalRequest,
) -> Result<MCPToolApproval, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let approved_at = if request.auto_approve {
        Some(Utc::now())
    } else {
        None
    };

    // Use a separate query approach since the unique constraint is deferrable
    // First try to update existing record
    let existing = sqlx::query_as!(
        MCPToolApproval,
        r#"
        UPDATE mcp_tool_approvals SET
            approved = $4,
            auto_approve = $4,
            approved_at = $5,
            expires_at = $6,
            notes = $7,
            updated_at = NOW()
        WHERE user_id = $1 AND server_id = $2 AND tool_name = $3 AND is_global = true
        RETURNING *
        "#,
        user_id,
        server_id,
        tool_name,
        request.auto_approve,
        approved_at,
        request.expires_at,
        request.notes
    )
    .fetch_optional(pool)
    .await?;

    if let Some(approval) = existing {
        return Ok(approval);
    }

    // If no existing record, insert new one
    sqlx::query_as!(
        MCPToolApproval,
        r#"
        INSERT INTO mcp_tool_approvals (
            user_id, conversation_id, server_id, tool_name,
            approved, auto_approve, is_global, approved_at, expires_at, notes
        )
        VALUES ($1, NULL, $2, $3, $4, $4, true, $5, $6, $7)
        RETURNING *
        "#,
        user_id,
        server_id,
        tool_name,
        request.auto_approve, // approved = auto_approve for global approvals
        approved_at,
        request.expires_at,
        request.notes
    )
    .fetch_one(pool)
    .await
}

/// Remove global tool approval
pub async fn delete_global_tool_approval(
    user_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        r#"
        DELETE FROM mcp_tool_approvals
        WHERE user_id = $1 AND server_id = $2 AND tool_name = $3 AND is_global = true
        "#,
        user_id,
        server_id,
        tool_name
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Create or update a conversation-specific tool approval
pub async fn create_conversation_tool_approval(
    user_id: Uuid,
    conversation_id: Uuid,
    request: CreateConversationApprovalRequest,
) -> Result<MCPToolApproval, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let approved_at = if request.approved {
        Some(Utc::now())
    } else {
        None
    };

    // First try to update existing record (including expired ones)
    let existing = sqlx::query_as!(
        MCPToolApproval,
        r#"
        UPDATE mcp_tool_approvals SET
            approved = $5,
            approved_at = $6,
            expires_at = $7,
            notes = $8,
            updated_at = NOW()
        WHERE user_id = $1 AND conversation_id = $2 AND server_id = $3 AND tool_name = $4 AND is_global = false
        RETURNING *
        "#,
        user_id,
        conversation_id,
        request.server_id,
        request.tool_name,
        request.approved,
        approved_at,
        request.expires_at,
        request.notes
    )
    .fetch_optional(pool)
    .await?;

    if let Some(approval) = existing {
        return Ok(approval);
    }

    // If no existing record, insert new one
    sqlx::query_as!(
        MCPToolApproval,
        r#"
        INSERT INTO mcp_tool_approvals (
            user_id, conversation_id, server_id, tool_name,
            approved, auto_approve, is_global, approved_at, expires_at, notes
        )
        VALUES ($1, $2, $3, $4, $5, false, false, $6, $7, $8)
        RETURNING *
        "#,
        user_id,
        conversation_id,
        request.server_id,
        request.tool_name,
        request.approved,
        approved_at,
        request.expires_at,
        request.notes
    )
    .fetch_one(pool)
    .await
}

/// Check if tool is approved (checks both global and conversation-specific)
pub async fn check_tool_approval(
    user_id: Uuid,
    conversation_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
) -> Result<Option<(bool, String)>, sqlx::Error> { // (approved, source)
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First check for global auto_approve
    let global_result = sqlx::query!(
        r#"
        SELECT auto_approve, expires_at
        FROM mcp_tool_approvals
        WHERE user_id = $1 AND server_id = $2 AND tool_name = $3 AND is_global = true
          AND approved = true
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        user_id,
        server_id,
        tool_name
    )
    .fetch_optional(pool)
    .await?;

    if let Some(global) = global_result {
        if global.auto_approve {
            return Ok(Some((true, "global".to_string())));
        }
    }

    // Then check for conversation-specific approval
    let conversation_result = sqlx::query!(
        r#"
        SELECT approved
        FROM mcp_tool_approvals
        WHERE user_id = $1 AND conversation_id = $2 AND server_id = $3 AND tool_name = $4
          AND is_global = false AND approved = true
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        user_id,
        conversation_id,
        server_id,
        tool_name
    )
    .fetch_optional(pool)
    .await?;

    if let Some(_conv) = conversation_result {
        return Ok(Some((true, "conversation".to_string())));
    }

    Ok(None)
}

/// List conversation tool approvals
pub async fn list_conversation_tool_approvals(
    user_id: Uuid,
    conversation_id: Uuid,
    query: ListConversationApprovalsQuery,
) -> Result<Vec<ToolApprovalResponse>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let mut sql = r#"
        SELECT
            a.id, a.user_id, a.conversation_id, a.server_id, a.tool_name,
            a.approved, a.auto_approve, a.is_global, a.approved_at, a.expires_at, a.notes,
            a.created_at, a.updated_at,
            COALESCE(s.display_name, s.name) as server_name,
            CASE WHEN a.expires_at IS NOT NULL AND a.expires_at <= NOW()
                 THEN true ELSE false END as is_expired
        FROM mcp_tool_approvals a
        JOIN mcp_servers s ON a.server_id = s.id
        WHERE a.user_id = $1 AND a.conversation_id = $2 AND a.is_global = false
    "#.to_string();

    let mut param_count = 2;
    let mut conditions = Vec::new();

    if let Some(_server_id) = query.server_id {
        param_count += 1;
        conditions.push(format!("AND a.server_id = ${}", param_count));
    }

    if let Some(_approved) = query.approved {
        param_count += 1;
        conditions.push(format!("AND a.approved = ${}", param_count));
    }

    if let Some(_tool_name) = &query.tool_name {
        param_count += 1;
        conditions.push(format!("AND a.tool_name ILIKE ${}", param_count));
    }

    if !query.include_expired.unwrap_or(false) {
        conditions.push("AND (a.expires_at IS NULL OR a.expires_at > NOW())".to_string());
    }

    // Append conditions
    for condition in conditions {
        sql.push_str(&format!(" {}", condition));
    }

    sql.push_str(" ORDER BY a.created_at DESC");

    // Apply pagination
    let limit = query.per_page.unwrap_or(50).clamp(1, 100) as i64;
    let offset = ((query.page.unwrap_or(1) - 1) * query.per_page.unwrap_or(50)).max(0) as i64;
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    // Execute query with dynamic parameters
    let mut query_builder = sqlx::query_as::<_, ToolApprovalResponse>(&sql)
        .bind(user_id)
        .bind(conversation_id);

    if let Some(server_id) = query.server_id {
        query_builder = query_builder.bind(server_id);
    }

    if let Some(approved) = query.approved {
        query_builder = query_builder.bind(approved);
    }

    if let Some(tool_name) = &query.tool_name {
        query_builder = query_builder.bind(format!("%{}%", tool_name));
    }

    query_builder.fetch_all(pool).await
}

/// Delete conversation tool approval
pub async fn delete_conversation_tool_approval(
    user_id: Uuid,
    conversation_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        r#"
        DELETE FROM mcp_tool_approvals
        WHERE user_id = $1 AND conversation_id = $2 AND server_id = $3 AND tool_name = $4 AND is_global = false
        "#,
        user_id,
        conversation_id,
        server_id,
        tool_name
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Get global tool approval for a specific tool
pub async fn get_global_tool_approval(
    user_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
) -> Result<Option<MCPToolApproval>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query_as!(
        MCPToolApproval,
        r#"
        SELECT * FROM mcp_tool_approvals
        WHERE user_id = $1 AND server_id = $2 AND tool_name = $3 AND is_global = true
        "#,
        user_id,
        server_id,
        tool_name
    )
    .fetch_optional(pool)
    .await
}

/// Set global tool approval (alias for create_global_tool_approval for API compatibility)
pub async fn set_global_tool_approval(
    user_id: Uuid,
    server_id: Uuid,
    tool_name: &str,
    auto_approve: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
) -> Result<MCPToolApproval, sqlx::Error> {
    create_global_tool_approval(
        user_id,
        server_id,
        tool_name,
        SetToolGlobalApprovalRequest {
            auto_approve,
            expires_at,
            notes,
        }
    ).await
}

/// Update an existing tool approval
pub async fn update_tool_approval(
    user_id: Uuid,
    approval_id: Uuid,
    request: UpdateToolApprovalRequest,
) -> Result<MCPToolApproval, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let approved_at = if request.approved.unwrap_or(false) {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query_as!(
        MCPToolApproval,
        r#"
        UPDATE mcp_tool_approvals SET
            approved = COALESCE($2, approved),
            auto_approve = COALESCE($3, auto_approve),
            approved_at = CASE
                WHEN $2 = true THEN COALESCE($4, approved_at)
                WHEN $2 = false THEN NULL
                ELSE approved_at
            END,
            expires_at = COALESCE($5, expires_at),
            notes = COALESCE($6, notes),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $7
        RETURNING *
        "#,
        approval_id,
        request.approved,
        request.auto_approve,
        approved_at,
        request.expires_at,
        request.notes,
        user_id
    )
    .fetch_one(pool)
    .await
}

/// Clean expired tool approvals (maintenance function)
pub async fn clean_expired_tool_approvals() -> Result<i32, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        r#"
        DELETE FROM mcp_tool_approvals
        WHERE expires_at IS NOT NULL AND expires_at <= NOW()
        "#
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as i32)
}