use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPTool {
    pub id: Uuid,
    pub server_id: Uuid,
    pub tool_name: String,
    pub tool_description: Option<String>,
    pub input_schema: serde_json::Value,
    pub discovered_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i32,
}

// Tool with server information for frontend
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPToolWithServer {
    // Tool info
    pub id: Uuid,
    pub server_id: Uuid,
    pub tool_name: String,
    pub tool_description: Option<String>,
    pub input_schema: serde_json::Value,
    pub discovered_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i32,

    // Server info
    pub server_name: String,
    pub server_display_name: String,
    pub is_system: bool,
    pub transport_type: String,

    // Global approval info (optional, loaded when requested)
    pub global_auto_approve: Option<bool>,
    pub global_approval_expires_at: Option<DateTime<Utc>>,
    pub global_approval_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum MCPExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPToolApproval {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>, // NULL for global approvals
    pub server_id: Uuid,
    pub tool_name: String,
    pub approved: bool,
    pub auto_approve: bool,
    pub is_global: bool, // true = global approval, false = conversation-specific
    pub approved_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPExecutionLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub server_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub tool_name: String,
    pub tool_parameters: Option<serde_json::Value>,
    pub execution_result: Option<serde_json::Value>,
    pub status: MCPExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub request_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

// Request/Response types for tool execution
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecuteToolRequest {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub server_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub auto_approve: Option<bool>,
}

// Request/Response types for tool approvals
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetToolGlobalApprovalRequest {
    pub auto_approve: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateConversationApprovalRequest {
    pub server_id: Uuid,
    pub tool_name: String,
    pub approved: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateToolApprovalRequest {
    pub approved: Option<bool>,
    pub auto_approve: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToolApprovalResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub server_id: Uuid,
    pub server_name: String, // Joined from mcp_servers
    pub tool_name: String,
    pub approved: bool,
    pub auto_approve: bool,
    pub is_global: bool,
    pub approved_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_expired: bool, // Computed field
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for ToolApprovalResponse {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            conversation_id: row.try_get("conversation_id")?,
            server_id: row.try_get("server_id")?,
            server_name: row.try_get("server_name")?,
            tool_name: row.try_get("tool_name")?,
            approved: row.try_get("approved")?,
            auto_approve: row.try_get("auto_approve")?,
            is_global: row.try_get("is_global")?,
            approved_at: row.try_get("approved_at")?,
            expires_at: row.try_get("expires_at")?,
            is_expired: row.try_get::<Option<DateTime<Utc>>, _>("expires_at")?.map_or(false, |exp| exp <= Utc::now()),
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListConversationApprovalsQuery {
    pub server_id: Option<Uuid>,
    pub tool_name: Option<String>,
    pub approved: Option<bool>,
    pub include_expired: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToolExecutionResponse {
    pub execution_id: Uuid,
    pub status: MCPExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
}