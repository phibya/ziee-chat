use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{Type, Decode, Encode, Postgres};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MCPTransportType {
    Stdio,
    Http,
    Sse,
}

impl<'r> Decode<'r, Postgres> for MCPTransportType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        match s {
            "stdio" => Ok(MCPTransportType::Stdio),
            "http" => Ok(MCPTransportType::Http),
            "sse" => Ok(MCPTransportType::Sse),
            _ => Err(format!("Unknown transport type: {}", s).into()),
        }
    }
}

impl<'q> Encode<'q, Postgres> for MCPTransportType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            MCPTransportType::Stdio => "stdio",
            MCPTransportType::Http => "http",
            MCPTransportType::Sse => "sse",
        };
        <&str as Encode<Postgres>>::encode_by_ref(&s, buf)
    }
}

impl Type<Postgres> for MCPTransportType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MCPServerStatus {
    Stopped,
    Starting,
    Running,
    Error,
    Restarting,
}

impl<'r> Decode<'r, Postgres> for MCPServerStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        match s {
            "stopped" => Ok(MCPServerStatus::Stopped),
            "starting" => Ok(MCPServerStatus::Starting),
            "running" => Ok(MCPServerStatus::Running),
            "error" => Ok(MCPServerStatus::Error),
            "restarting" => Ok(MCPServerStatus::Restarting),
            _ => Err(format!("Unknown server status: {}", s).into()),
        }
    }
}

impl<'q> Encode<'q, Postgres> for MCPServerStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            MCPServerStatus::Stopped => "stopped",
            MCPServerStatus::Starting => "starting",
            MCPServerStatus::Running => "running",
            MCPServerStatus::Error => "error",
            MCPServerStatus::Restarting => "restarting",
        };
        <&str as Encode<Postgres>>::encode_by_ref(&s, buf)
    }
}

impl Type<Postgres> for MCPServerStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }
}

impl std::fmt::Display for MCPServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCPServerStatus::Stopped => write!(f, "stopped"),
            MCPServerStatus::Starting => write!(f, "starting"),
            MCPServerStatus::Running => write!(f, "running"),
            MCPServerStatus::Error => write!(f, "error"),
            MCPServerStatus::Restarting => write!(f, "restarting"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MCPServer {
    pub id: Uuid,
    pub user_id: Option<Uuid>,

    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub is_system: bool,

    pub transport_type: MCPTransportType,

    // Connection config
    pub command: Option<String>,
    pub args: serde_json::Value,
    pub environment_variables: serde_json::Value,
    pub url: Option<String>,
    pub headers: serde_json::Value,
    pub timeout_seconds: Option<i32>,

    // Status
    pub status: MCPServerStatus,
    pub is_active: bool,
    pub last_health_check: Option<DateTime<Utc>>,
    pub restart_count: i32,
    pub last_restart_at: Option<DateTime<Utc>>,
    pub max_restart_attempts: Option<i32>,

    // Process info
    pub process_id: Option<i32>,
    pub port: Option<i32>,

    // Tools
    pub tools_discovered_at: Option<DateTime<Utc>>,
    pub tool_count: Option<i32>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateMCPServerRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub transport_type: MCPTransportType,

    // Connection config (based on transport type)
    pub command: Option<String>,
    pub args: Option<serde_json::Value>,
    pub environment_variables: Option<serde_json::Value>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub timeout_seconds: Option<i32>,

    pub max_restart_attempts: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSystemMCPServerRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub transport_type: MCPTransportType,

    // Connection config
    pub command: Option<String>,
    pub args: Option<serde_json::Value>,
    pub environment_variables: Option<serde_json::Value>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub timeout_seconds: Option<i32>,

    pub max_restart_attempts: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMCPServerRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,

    // Connection config updates
    pub command: Option<String>,
    pub args: Option<serde_json::Value>,
    pub environment_variables: Option<serde_json::Value>,
    pub url: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub timeout_seconds: Option<i32>,

    pub max_restart_attempts: Option<i32>,
}

