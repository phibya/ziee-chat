pub mod assistants;
pub mod config;
pub mod downloads;
pub mod groups;
pub mod models;
pub mod providers;
pub mod repositories;
pub mod users;

use axum::Router;

pub fn admin_routes() -> Router {
    Router::new()
        .merge(users::admin_user_routes())
        .merge(groups::admin_group_routes())
        .merge(config::admin_config_routes())
        .merge(providers::admin_provider_routes())
        .merge(models::admin_model_routes())
        .merge(repositories::admin_repository_routes())
        .merge(assistants::admin_assistant_routes())
        .merge(downloads::admin_download_routes())
}