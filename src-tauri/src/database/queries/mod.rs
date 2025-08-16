pub mod api_proxy_server_models;
pub mod assistants;
pub mod branches;
pub mod chat;
pub mod configuration;
pub mod download_instances;
pub mod files;
pub mod models;
pub mod projects;
pub mod providers;
pub mod rag_providers;
pub mod rag_repositories;
pub mod repositories;
pub mod user_group_providers;
pub mod user_groups;
pub mod user_settings;
pub mod users;

use crate::database::DATABASE_POOL;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub(crate) fn get_database_pool() -> Result<Arc<Pool<Postgres>>, sqlx::Error> {
    DATABASE_POOL
        .get()
        .ok_or_else(|| sqlx::Error::PoolClosed)
        .cloned()
}
