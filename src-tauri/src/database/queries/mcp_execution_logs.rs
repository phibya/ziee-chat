use uuid::Uuid;

use crate::database::{
    get_database_pool,
    models::{MCPExecutionLog, MCPExecutionStatus},
};

/// Create a new execution log entry
pub async fn create_execution_log(
    user_id: Uuid,
    server_id: Uuid,
    thread_id: Option<Uuid>,
    tool_name: String,
    tool_parameters: Option<serde_json::Value>,
    request_id: Option<Uuid>,
) -> Result<Uuid, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let execution_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO mcp_execution_logs (
            id, user_id, server_id, thread_id, tool_name,
            tool_parameters, status, request_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        execution_id,
        user_id,
        server_id,
        thread_id,
        tool_name,
        tool_parameters,
        MCPExecutionStatus::Pending as MCPExecutionStatus,
        request_id
    )
    .execute(pool)
    .await?;

    Ok(execution_id)
}

/// Update execution log with completion data
pub async fn complete_execution_log(
    execution_id: Uuid,
    status: MCPExecutionStatus,
    execution_result: Option<serde_json::Value>,
    error_message: Option<String>,
    error_code: Option<String>,
    duration_ms: Option<i32>,
) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    sqlx::query!(
        r#"
        UPDATE mcp_execution_logs SET
            status = $2,
            execution_result = $3,
            error_message = $4,
            error_code = $5,
            duration_ms = $6,
            completed_at = NOW()
        WHERE id = $1
        "#,
        execution_id,
        status as MCPExecutionStatus,
        execution_result,
        error_message,
        error_code,
        duration_ms
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get execution log by ID
pub async fn get_execution_log(execution_id: Uuid) -> Result<Option<MCPExecutionLog>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let log = sqlx::query_as!(
        MCPExecutionLog,
        r#"
        SELECT
            id, user_id, server_id, thread_id, tool_name,
            tool_parameters, execution_result,
            status as "status: MCPExecutionStatus",
            started_at, completed_at, duration_ms,
            error_message, error_code, request_id, correlation_id
        FROM mcp_execution_logs
        WHERE id = $1
        "#,
        execution_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(log)
}

/// List execution logs for a user
pub async fn list_user_execution_logs(
    user_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<MCPExecutionLog>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let logs = sqlx::query_as!(
        MCPExecutionLog,
        r#"
        SELECT
            id, user_id, server_id, thread_id, tool_name,
            tool_parameters, execution_result,
            status as "status: MCPExecutionStatus",
            started_at, completed_at, duration_ms,
            error_message, error_code, request_id, correlation_id
        FROM mcp_execution_logs
        WHERE user_id = $1
        ORDER BY started_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

/// List execution logs for a specific thread
pub async fn list_thread_execution_logs(thread_id: Uuid) -> Result<Vec<MCPExecutionLog>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let logs = sqlx::query_as!(
        MCPExecutionLog,
        r#"
        SELECT
            id, user_id, server_id, thread_id, tool_name,
            tool_parameters, execution_result,
            status as "status: MCPExecutionStatus",
            started_at, completed_at, duration_ms,
            error_message, error_code, request_id, correlation_id
        FROM mcp_execution_logs
        WHERE thread_id = $1
        ORDER BY started_at ASC
        "#,
        thread_id
    )
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

/// List all execution logs (admin view)
pub async fn list_all_execution_logs(
    limit: Option<i64>,
    offset: Option<i64>,
    status_filter: Option<MCPExecutionStatus>,
) -> Result<Vec<MCPExecutionLog>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let logs = match status_filter {
        Some(status) => {
            sqlx::query_as!(
                MCPExecutionLog,
                r#"
                SELECT
                    id, user_id, server_id, thread_id, tool_name,
                    tool_parameters, execution_result,
                    status as "status: MCPExecutionStatus",
                    started_at, completed_at, duration_ms,
                    error_message, error_code, request_id, correlation_id
                FROM mcp_execution_logs
                WHERE status = $1
                ORDER BY started_at DESC
                LIMIT $2 OFFSET $3
                "#,
                status as MCPExecutionStatus,
                limit,
                offset
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                MCPExecutionLog,
                r#"
                SELECT
                    id, user_id, server_id, thread_id, tool_name,
                    tool_parameters, execution_result,
                    status as "status: MCPExecutionStatus",
                    started_at, completed_at, duration_ms,
                    error_message, error_code, request_id, correlation_id
                FROM mcp_execution_logs
                ORDER BY started_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(logs)
}

/// Get execution statistics for admin dashboard
pub async fn get_execution_statistics() -> Result<serde_json::Value, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let stats = sqlx::query!(
        r#"
        SELECT
            status::TEXT,
            COUNT(*) as count,
            AVG(duration_ms)::FLOAT as avg_duration_ms,
            MAX(duration_ms) as max_duration_ms,
            MIN(duration_ms) as min_duration_ms
        FROM mcp_execution_logs
        WHERE started_at >= NOW() - INTERVAL '24 hours'
        GROUP BY status
        "#
    )
    .fetch_all(pool)
    .await?;

    let total_executions = sqlx::query!(
        r#"
        SELECT COUNT(*) as total FROM mcp_execution_logs
        WHERE started_at >= NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(pool)
    .await?;

    let top_tools = sqlx::query!(
        r#"
        SELECT
            tool_name,
            COUNT(*) as usage_count,
            AVG(duration_ms)::FLOAT as avg_duration_ms
        FROM mcp_execution_logs
        WHERE started_at >= NOW() - INTERVAL '24 hours'
        GROUP BY tool_name
        ORDER BY usage_count DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(serde_json::json!({
        "total_executions_24h": total_executions.total,
        "status_breakdown": stats.into_iter().map(|row| {
            serde_json::json!({
                "status": row.status,
                "count": row.count,
                "avg_duration_ms": row.avg_duration_ms,
                "max_duration_ms": row.max_duration_ms,
                "min_duration_ms": row.min_duration_ms
            })
        }).collect::<Vec<_>>(),
        "top_tools": top_tools.into_iter().map(|row| {
            serde_json::json!({
                "tool_name": row.tool_name,
                "usage_count": row.usage_count,
                "avg_duration_ms": row.avg_duration_ms
            })
        }).collect::<Vec<_>>()
    }))
}

/// Clean up old execution logs (retention policy)
pub async fn cleanup_old_execution_logs(retention_days: i32) -> Result<u64, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query!(
        "DELETE FROM mcp_execution_logs WHERE started_at < NOW() - INTERVAL '1 day' * $1",
        retention_days as f64
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}