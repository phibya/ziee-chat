pub mod api_proxy_server_models;
pub mod assistants;
pub mod auth_providers;
pub mod auth_sessions;
pub mod branches;
pub mod chat;
pub mod configuration;
pub mod download_instances;
pub mod files;
pub mod mcp_execution_logs;
pub mod mcp_servers;
pub mod mcp_tool_approvals;
pub mod mcp_tools;
pub mod models;
pub mod projects;
pub mod providers;
pub mod repositories;
pub mod user_auth_links;
pub mod user_group_mcp_servers;
pub mod user_group_providers;
pub mod user_groups;
pub mod user_settings;
pub mod users;

// Alias for compatibility with user provisioning
pub mod groups {
    pub use super::user_groups::get_by_name;
}

use crate::database::DATABASE_POOL;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub(crate) fn get_database_pool() -> Result<Arc<Pool<Postgres>>, sqlx::Error> {
    DATABASE_POOL
        .get()
        .ok_or_else(|| sqlx::Error::PoolClosed)
        .cloned()
}
