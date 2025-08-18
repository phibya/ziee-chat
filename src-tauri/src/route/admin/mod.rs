pub mod api_proxy_server;
pub mod assistants;
pub mod config;
pub mod downloads;
pub mod engines;
pub mod groups;
pub mod hardware;
pub mod models;
pub mod providers;
pub mod rag_providers;
pub mod rag_repositories;
pub mod repositories;
pub mod users;

use aide::axum::ApiRouter;

pub fn admin_routes() -> ApiRouter {
    ApiRouter::new().nest(
        "/admin",
        ApiRouter::new()
            .merge(users::admin_user_routes())
            .merge(groups::admin_group_routes())
            .merge(config::admin_config_routes())
            .merge(providers::admin_provider_routes())
            .merge(models::admin_model_routes())
            .merge(repositories::admin_repository_routes())
            .merge(rag_providers::admin_rag_provider_routes())
            .merge(rag_repositories::admin_rag_repository_routes())
            .merge(assistants::admin_assistant_routes())
            .merge(downloads::admin_download_routes())
            .merge(engines::admin_engine_routes())
            .merge(hardware::hardware_routes())
            .merge(api_proxy_server::admin_api_proxy_server_routes()),
    )
}
