use axum::{
    debug_handler,
    extract::Path,
    response::{sse::{Event, KeepAlive}, Sse},
    http::StatusCode,
    Extension,
};
use futures_util::Stream;
use std::convert::Infallible;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;
use schemars::JsonSchema;

use crate::api::{
    errors::{ApiResult, AppError},
    middleware::AuthenticatedUser,
    permissions::{check_permission, Permission},
};
use crate::database::queries::{mcp_servers};
use crate::ai::mcp::logging::{MCPLogger, MCPLogEntry, LogWatcherManager};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MCPLogConnectionInfo {
    pub server_id: Uuid,
    pub server_name: String,
    pub connected_at: DateTime<Utc>,
    pub initial_log_count: usize,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MCPInitialLogsComplete {
    pub server_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MCPLogError {
    pub error: String,
    pub server_id: Uuid,
}

// SSE event types for MCP server log streaming
crate::sse_event_enum! {
    #[derive(Debug, Clone, Serialize, JsonSchema)]
    pub enum SSEMCPLogEvent {
        LogEntry(MCPLogEntry),
        Connected(MCPLogConnectionInfo),
        InitialLogsComplete(MCPInitialLogsComplete),
        Error(MCPLogError),
    }
}

#[debug_handler]
pub async fn stream_server_logs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(server_id): Path<Uuid>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // 1. Get server info first to check if it's a system server
    let server = mcp_servers::get_mcp_server_by_id(server_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, AppError::database_error(err)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, AppError::not_found("Server not found")))?;

    // 2. Check access based on server type
    if server.is_system {
        // For system servers, check if user has admin MCP read permission
        if !check_permission(&auth_user.user, Permission::McpAdminServersRead.as_str()) {
            return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to system server logs")));
        }
    } else {
        // For user servers, check if user owns the server
        if server.user_id != Some(auth_user.user_id) {
            return Err((StatusCode::FORBIDDEN, AppError::forbidden("Access denied to this server")));
        }
    }

    // 3. Create log receiver for this server (starts file watching if needed)
    let mut log_receiver = LogWatcherManager::subscribe_to_server_logs(server_id);

    // 4. Get recent logs from today (last 50 lines)
    let logger = MCPLogger::new(server_id);
    let recent_logs = logger.get_recent_logs(50)
        .unwrap_or_default();

    // 5. Create event stream
    let stream = async_stream::stream! {
        // Send initial connection event
        let connection_info = MCPLogConnectionInfo {
            server_id,
            server_name: server.display_name.clone(),
            connected_at: Utc::now(),
            initial_log_count: recent_logs.len(),
        };
        yield Ok(SSEMCPLogEvent::Connected(connection_info).into());

        // Send recent logs first (last 50 lines from today)
        for log_entry in recent_logs {
            yield Ok(SSEMCPLogEvent::LogEntry(log_entry).into());
        }

        // Send marker to indicate initial logs are done
        yield Ok(SSEMCPLogEvent::InitialLogsComplete(MCPInitialLogsComplete {
            server_id,
            timestamp: Utc::now(),
        }).into());

        // Stream new log entries in real-time
        loop {
            match log_receiver.recv().await {
                Ok(log_entry) => {
                    yield Ok(SSEMCPLogEvent::LogEntry(log_entry).into());
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Channel closed, stop streaming
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Channel lagged, send error event and continue
                    let error_event = SSEMCPLogEvent::Error(MCPLogError {
                        error: format!("Log stream lagged behind by {} messages", n),
                        server_id,
                    });
                    yield Ok(error_event.into());
                }
            }
        }

        // Clean up subscription when stream ends
        LogWatcherManager::unsubscribe_from_server_logs(server_id);
    };

    Ok((StatusCode::OK, Sse::new(stream).keep_alive(KeepAlive::default())))
}