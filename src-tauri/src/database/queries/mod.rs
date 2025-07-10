pub mod assistants;
pub mod branches;
pub mod chat;
pub mod configuration;
pub mod model_providers;
pub mod user_groups;
pub mod user_group_model_providers;
pub mod user_settings;
pub mod users;

use crate::database::DATABASE_POOL;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub(crate) fn get_database_pool() -> Result<Arc<Pool<Postgres>>, sqlx::Error> {
    DATABASE_POOL.get().ok_or_else(|| sqlx::Error::PoolClosed).cloned()
}
