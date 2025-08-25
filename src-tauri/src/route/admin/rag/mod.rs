use aide::axum::ApiRouter;

pub mod instances;
pub mod providers; 
pub mod repositories;

pub fn admin_rag_routes() -> ApiRouter {
    ApiRouter::new()
        .merge(instances::admin_rag_instance_routes())
        .merge(providers::admin_rag_provider_routes())
        .merge(repositories::admin_rag_repository_routes())
}